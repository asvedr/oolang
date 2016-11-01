//use syn_expr::*;
//use syn_act::*;
use syn_utils::*;
use type_sys::*;
use syn_reserr::*;
use syn_fn::*;

pub struct Class {
	pub addres    : Cursor,
	pub parent    : Option<Type>,
	pub template  : Vec<Type>,
	pub name      : String,
	pub priv_fn   : Vec<SynFn>,
	pub pub_fn    : Vec<SynFn>,
	pub priv_prop : Vec<Pair<String,Type>>,
	pub pub_prop  : Vec<Pair<String,Type>>
}

impl Show for Class {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		let head = format!("{}CLASS {:?} {} {:?}",tab,self.template,self.name,self.parent);
		let mut res = vec![head];
		tab.push(' ');
		for p in self.priv_prop.iter() {
			res.push(format!("{}PRIV {} : {:?}", tab, p.a, p.b));
		}
		for p in self.pub_prop.iter() {
			res.push(format!("{}PUBL {} : {:?}", tab, p.a, p.b));
		}
		for f in self.priv_fn.iter() {
			res.push(format!("{}PRIV", tab));
			for line in f.show(layer + 1) {
				res.push(line);
			}
		}
		for f in self.pub_fn.iter() {	
			res.push(format!("{}PUBL", tab));
			for line in f.show(layer + 1) {
				res.push(line);
			}
		}
		res
	}
}

pub fn parse_class(lexer : &Lexer, curs : &Cursor) -> SynRes<Class> {
	let addres : Cursor = curs.clone();
	let mut curs = lex!(lexer, curs, "class");
	// template
	let tmpl = {
		let sym = lex!(lexer, &curs);
		if sym.val == "<" {
			let ans = try!(parse_tmpl(lexer, &curs));
			curs = ans.cursor;
			ans.val
		} else {
			vec![]
		}
	};
	// name
	let cname = lex_type!(lexer, &curs, LexTP::Id);
	curs = cname.cursor;
	let cname = cname.val;
	// parent
	let parent = {
		let sym = lex!(lexer, &curs);
		if sym.val == ":" {
			let ans = try!(parse_type(lexer, &sym.cursor));
			curs = ans.cursor;
			Some(ans.val)
		} else {
			None
		}
	};

	curs = lex!(lexer, &curs, "{");

	// PROPS

	let mut priv_fn   = vec![];
	let mut pub_fn    = vec![];
	let mut priv_prop = vec![];
	let mut pub_prop  = vec![];

	loop {
		let ans = lex!(lexer, &curs);
		if ans.val == "}" {
			break;
			//syn_ok!(acc, ans.cursor);
		}
		let is_pub = {
			let sym = lex_type!(lexer, &curs, LexTP::Id);
			if sym.val == "pub" {
				curs = sym.cursor;
				true
			} else if sym.val == "priv" {
				curs = sym.cursor;
				false
			} else {
				syn_throw!(format!("expected 'pub' or 'priv', found '{}'", sym.val), curs)
			}
		};
		let sym = lex!(lexer, &curs);
		if sym.val == "fn" {
			let meth = try!(parse_fn_full(lexer, &curs));
			if is_pub {
				pub_fn.push(meth.val);
			} else {
				priv_fn.push(meth.val);
			}
			curs = meth.cursor;
		} else {
			let fname = lex_type!(lexer, &curs, LexTP::Id);
			curs = lex!(lexer, &fname.cursor, ":");
			let tp = try!(parse_type(lexer, &curs));
			if is_pub {
				pub_prop.push(Pair{a : fname.val, b : tp.val});
			} else {
				priv_prop.push(Pair{a : fname.val, b : tp.val});
			}
			curs = tp.cursor;
		}
		let ans = lex!(lexer, &curs);
		if ans.val == "}" {
			//syn_ok!(acc, ans.cursor);
			break;
		} else if ans.val == ";" {
			curs = ans.cursor;
		} else {
			syn_throw!(format!("expected ';' or '{}'", "{"), curs)
		}
	}

	let cls = Class {
		addres    : addres,
		parent    : parent,
		template  : tmpl,
		name      : cname,
		priv_fn   : priv_fn,
		pub_fn    : pub_fn,
		priv_prop : priv_prop,
		pub_prop  : pub_prop	
	};

	syn_ok!(cls, curs);
}
