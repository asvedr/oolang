use syn_expr::*;
use syn_act::*;
use syn_utils::*;
use type_sys::*;
//use lexer::*;
use syn_reserr::*;
//use std::fmt;

pub struct Arg {
	pub name  : String,
	pub tp    : Type,
	pub val   : Option<Expr>,
	pub named : bool
}

pub struct SynFn {
	pub name        : Option<String>,
	pub tmpl        : Option<Tmpl>,
	pub args        : Vec<Arg>,
	pub rettp       : Type,
	pub body        : Vec<Act<SynFn>>,
	pub addr        : Cursor,
	pub can_be_clos : bool // if has names args or option args then can't be used as closure
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
		let mut res = vec![format!("{}func {:?} allowclos:{} type:{:?}", tab, self.name, self.can_be_clos, self.rettp)];
		for arg in self.args.iter() {
			for line in arg.show(layer + 1) {
				res.push(line);
			}
		}
		res.push(format!("{}BODY", tab));
		for cmd in self.body.iter() {
			for line in cmd.show(layer + 1) {
				res.push(line);
			}
		}
		res
	}
}

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
		Some(tmpl.val)
	} else {
		curs = name.cursor;
		None
	};
	// args
	let parser = |l : &Lexer, c : &Cursor| {parse_arg(l, c, true)};
	let args = try!(parse_list(lexer, &curs, &parser, "(", ")"));
	let mut can_be_clos = true;
	for a in args.val.iter() {
		if a.named || match a.val {None => false, _ => true} {
			can_be_clos = false;
			break;
		}
	}
	// ret type
	curs = lex!(lexer, &args.cursor, ":");
	let tp = try!(parse_type(lexer, &curs));
	// body
	let body = try!(parse_act_list(lexer, &tp.cursor, &parse_fn_full));
	let res = SynFn {
		name        : Some(name.val),
		tmpl        : tmpl,
		args        : args.val,
		rettp       : tp.val,
		body        : body.val,
		addr        : orig,
		can_be_clos : can_be_clos
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
