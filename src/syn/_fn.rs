use syn::expr::*;
use syn::act::*;
use syn::utils::*;
use syn::type_sys::*;
//use lexer::*;
use syn::reserr::*;
use std::rc::Rc;
use std::collections::BTreeMap;

pub struct Arg {
	pub name  : String,
	pub tp    : RType,
	pub val   : Option<Expr>,
	pub named : bool
}

pub struct SynFn {
	pub name        : String,
	pub tmpl        : Tmpl,            // if fun has no tmpl then []
	pub args        : Vec<Arg>,
	pub rettp       : RType,           // return type
	pub body        : Vec<Act<SynFn>>, 
	pub addr        : Cursor,          // fun start addres
	pub can_be_clos : bool,            // if has names args or option args then can't be used as closure
	pub has_named   : bool,            // does fun has named args
	pub ftype       : RType,           // Fn(args) -> res
	// COMPILE TIME
	pub outers      : BTreeMap<String,RType>,
	pub no_except   : bool             // force optimization flag #noexcept
}

impl Show for Arg {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		let mut res = vec![format!("{}name:{} nflag:{} type:{:?}", tab, self.name, self.named, self.tp)];
		match self.val {
			None => res,
			Some(ref e) => {
				for line in e.show(layer + 1) {
					res.push(line)
				}
				res
			}
		}
	}
}

impl Show for SynFn {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		let name : &String = &self.name;
		let mut res = Vec::new();
		if self.no_except {
			res.push(format!("{}#NoExcept", tab));
		}
		if self.tmpl.len() > 0 {
			res.push(format!("{}func {} tmpl:{:?} allowclos:{} type:{:?}", tab, name, self.tmpl, self.can_be_clos, self.rettp));
		} else {
			res.push(format!("{}func {} allowclos:{} type:{:?}", tab, name, self.can_be_clos, self.rettp));
		}
		tab.push(' ');
		res.push(format!("{}OTRS: {:?}", tab, self.outers));
		res.push(format!("{}ARGS", tab));
		for arg in self.args.iter() {
			for line in arg.show(layer + 2) {
				res.push(line);
			}
		}
		res.push(format!("{}BODY", tab));
		for cmd in self.body.iter() {
			for line in cmd.show(layer + 2) {
				res.push(line);
			}
		}
		res
	}
}

/*
pub fn parse_lambda(lexer : &Lexer, curs : &Cursor) -> SynRes<SynFn> {
	// symbol 'fn'
	let orig = curs.clone();
	let mut curs = lex!(lexer, curs, "fn");
	// args
	let parser = |l : &Lexer, c : &Cursor| {parse_arg(l, c, false)};
	let args = try!(parse_list(lexer, &curs, &parser, "(", ")"));
	// ret type
	curs = lex!(lexer, &args.cursor, ":");
	let tp = try!(parse_type(lexer, &curs));
	// body
	let body = try!(parse_act_list(lexer, &tp.cursor, &parse_fn_full));
	let res = SynFn {
		name        : None,
		tmpl        : None,
		args        : args.val,
		rettp       : tp.val,
		body        : body.val,
		addr        : orig,
		can_be_clos : false
	};
	syn_ok!(res, body.cursor)	
}
*/

pub fn parse_fn_full(lexer : &Lexer, curs : &Cursor) -> SynRes<SynFn> {
	// symbol 'fn'
	let orig = curs.clone();
	let curs = lex!(lexer, curs, "fn");
	// name
	let name = lex_type!(lexer, &curs, LexTP::Id);
	// template
	let has_tmpl = match lexer.lex(&name.cursor) {
		Ok(ans) => ans.val == "<",
		_ => false
	};
	let mut curs;
	let tmpl = if has_tmpl {
		let tmpl = try!(parse_tmpl(lexer, &name.cursor));
		curs = tmpl.cursor;
		tmpl.val
	} else {
		curs = name.cursor;
		vec![]
	};
	// args
	let parser = |l : &Lexer, c : &Cursor| {parse_arg(l, c, true)};
	let args = try!(parse_list(lexer, &curs, &parser, "(", ")"));
	let mut can_be_clos = !has_tmpl;//true;
	let mut has_named = false;
	for a in args.val.iter() {
		if a.named || match a.val {None => false, _ => true} {
			can_be_clos = false;
			has_named = true;
			break;
		}
	}
	// ret type
	//curs = lex!(lexer, &args.cursor, ":");
	let ans = lex!(lexer, &args.cursor);
	let tp;
	if ans.val == ":" {
		let ans = try!(parse_type(lexer, &ans.cursor));
		curs = ans.cursor;
		tp = ans.val;
	} else if ans.val == "{" {
		tp = Type::void();
		curs = args.cursor.clone();
	} else {
		syn_throw!("expected ':' or '{'", args.cursor);
	}
	// body
	let body = try!(parse_act_list(lexer, /*&tp.cursor*/&curs, &parse_fn_full));
	// type
	let mut atypes = vec![];
	for a in args.val.iter() {
		atypes.push(a.tp.clone())
	}
	let ftype =
		if tmpl.len() == 0 {
			Rc::new(Type::Fn(None, atypes, tp./*val.*/clone()))
		} else {
			Rc::new(Type::Fn(Some(tmpl.clone()), atypes, tp./*val.*/clone()))
		};
	let res = SynFn {
		name        : name.val,
		tmpl        : tmpl,
		args        : args.val,
		rettp       : tp/*.val*/,
		body        : body.val,
		addr        : orig,
		can_be_clos : can_be_clos,
		has_named   : has_named,
		ftype       : ftype,
		outers      : BTreeMap::new(),//Vec::new(),
		no_except   : false
	};
	syn_ok!(res, body.cursor)
}

fn parse_arg(lexer : &Lexer, curs : &Cursor, all_features : bool) -> SynRes<Arg> {
	let sym = lex!(lexer, curs);
	// named feature
	let mut curs = curs.clone();
	let named = if all_features && sym.val == "~" {
		curs = sym.cursor;
		true
	} else {
		false
	};
	// arg name
	let name = lex_type!(lexer, &curs, LexTP::Id);
	curs = name.cursor;
	// default value
	let sym = lex!(lexer, &curs);
	let val = if all_features && sym.val == "=" {
		let val = try!(parse_expr(lexer, &sym.cursor));
		curs = val.cursor;
		Some(val.val)
	} else {
		None
	};
	// arg type
	curs = lex!(lexer, &curs, ":");
	let tp = try!(parse_type(lexer, &curs));
	curs = tp.cursor;
	let res = Arg {
		named : named,
		name  : name.val,
		val   : val,
		tp    : tp.val
	};
	syn_ok!(res, curs);
}
