//use syn_expr::*;
//use syn_act::*;
use syn::utils::*;
use syn::type_sys::*;
use syn::reserr::*;
use syn::_fn::*;
use syn::compile_flags::*;

pub struct Class {
	pub addres    : Cursor,
	pub parent    : Option<Type>,
	pub singleton : bool,
	pub template  : Vec<String>,
	pub name      : String,
	pub priv_fn   : Vec<Method>,//SynFn>,
	pub pub_fn    : Vec<Method>,//SynFn>,
	pub priv_prop : Vec<Prop>,//Pair<String,Type>>,
	pub pub_prop  : Vec<Prop> //Pair<String,Type>>
}

pub struct Method {
	pub is_virt : bool,
	pub func    : SynFn,
	pub ftype   : Type // fill in type check
}

pub struct Prop {
	pub name   : String,
	pub ptype  : Type,
	pub addres : Cursor
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
			res.push(format!("{}PRIV {} : {:?}", tab, p.name, p.ptype));
		}
		for p in self.pub_prop.iter() {
			res.push(format!("{}PUBL {} : {:?}", tab, p.name, p.ptype));
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
		let mut ans = lex!(lexer, &curs);
		if ans.val == "}" {
			break;
			//syn_ok!(acc, ans.cursor);
		}
		// compile attribs
		let mut c_attribs = vec![];
		loop {
			if ans.val == "#" {
				let flag = parse_comp_flag(lexer, &curs)?;
				c_attribs.push(flag.val);
				curs = flag.cursor;
				ans = lex!(lexer, &curs);
			} else {
				break
			}
		}
		macro_rules! attr_to_fn {($fun:expr) => {
			for attr in c_attribs {
				if attr == CompFlag::NoExcept {
					$fun.no_except = true;
				}
			}
		};}

		// def fun order:      (pub|priv) [virtual] fn ...
		// def property order: (pub|priv) <Type> <name>
		// modif
		let start = curs.clone();
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
			let mut meth = try!(parse_fn_full(lexer, &curs));
			attr_to_fn!(meth.val);
			if is_pub {
				pub_fn.push(Method{is_virt : false, func : meth.val, ftype : Type::Unk});
			} else {
				priv_fn.push(Method{is_virt : false, func : meth.val, ftype : Type::Unk});
			}
			curs = meth.cursor;
		} else if sym.val == "virtual" {
			let mut meth = try!(parse_fn_full(lexer, &sym.cursor));
			attr_to_fn!(meth.val);
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
				pub_prop.push(Prop{name : fname.val, ptype : tp.val, addres : start});
			} else {
				priv_prop.push(Prop{name : fname.val, ptype : tp.val, addres : start});
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
