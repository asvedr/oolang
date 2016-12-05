//use syn_expr::*;
//use syn_act::*;
use syn::utils::*;
use syn::type_sys::*;
use syn::reserr::*;
use syn::_fn::*;

pub struct Class {
	pub addres    : Cursor,
	pub parent    : Option<Type>,
	pub singleton : bool,
	pub template  : Vec<String>,
	pub name      : String,
	pub priv_fn   : Vec<Method>,//SynFn>,
	pub pub_fn    : Vec<Method>,//SynFn>,
	pub priv_prop : Vec<Pair<String,Type>>,
	pub pub_prop  : Vec<Pair<String,Type>>
}

pub struct Method {
	pub is_virt : bool,
	pub func    : SynFn,
	pub ftype   : Type // fill in type check
}

impl Show for Class {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		let head = format!("{}CLASS sing:{} {:?} {} {:?}",tab,self.singleton,self.template,self.name,self.parent);
		let mut res = vec![head];
		tab.push(' ');
		for p in self.priv_prop.iter() {
			res.push(format!("{}PRIV {} : {:?}", tab, p.a, p.b));
		}
		for p in self.pub_prop.iter() {
			res.push(format!("{}PUBL {} : {:?}", tab, p.a, p.b));
		}
		for f in self.priv_fn.iter() {
			if f.is_virt {
				res.push(format!("{}PRIV VIRT", tab));
			} else {
				res.push(format!("{}PRIV", tab));
			}
			for line in f.func.show(layer + 1) {
				res.push(line);
			}
		}
		for f in self.pub_fn.iter() {
			if f.is_virt {
				res.push(format!("{}PUBL VIRT", tab));
			} else {
				res.push(format!("{}PUBL", tab));
			}
			for line in f.func.show(layer + 1) {
				res.push(line);
			}
		}
		res
	}
}

pub fn parse_class(lexer : &Lexer, curs : &Cursor) -> SynRes<Class> {
	let addres : Cursor = curs.clone();
	let mut curs = lex!(lexer, curs, "class");
	// singleton
	let singleton = {
		let sym = lex!(lexer, &curs);
		if sym.val == "single" {
			curs = sym.cursor;
			true
		} else {
			false
		}
	};
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
		// def fun order:      (pub|priv) [virtual] fn ...
		// def property order: (pub|priv) <Type> <name>
		// modif
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
		// data
		let sym = lex!(lexer, &curs);
		if sym.val == "fn" {
			let meth = try!(parse_fn_full(lexer, &curs));
			if is_pub {
				pub_fn.push(Method{is_virt : false, func : meth.val, ftype : Type::Unk});
			} else {
				priv_fn.push(Method{is_virt : false, func : meth.val, ftype : Type::Unk});
			}
			curs = meth.cursor;
		} else if sym.val == "virtual" {
			let meth = try!(parse_fn_full(lexer, &sym.cursor));
			if is_pub {
				pub_fn.push(Method{is_virt : true, func : meth.val, ftype : Type::Unk});
			} else {
				priv_fn.push(Method{is_virt : true, func : meth.val, ftype : Type::Unk});
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
			curs = ans.cursor;
			break;
		} else if ans.val == ";" {
			curs = ans.cursor;
		} else {
			syn_throw!(format!("expected ';' or '{}'", "{"), curs)
		}
	}

	let cls = Class {
		addres    : addres,
		singleton : singleton,
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
