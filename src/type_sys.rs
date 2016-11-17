use syn_reserr::*;
use syn_utils::*;
use std::fmt;

#[derive(Clone,PartialEq)]
pub enum Type {
	Unk, // UNKNOWN
	Int,
	Real,
	Char,
	Str,
	Bool,
	Void,
	Arr(Box<Type>),
	Class(Vec<String>,String,Option<Vec<Type>>),
	Fn(Option<Vec<String>>, Vec<Type>, Box<Type>),
}

#[macro_export]
macro_rules! type_fn {
	($t:expr, $args:expr, $res:expr) => {
		Type::Fn(Some($t), $args, Box::new($res))
	};
	($args:expr, $res:expr) => {
		Type::Fn(None, $args, Box::new($res))
	};
}

impl Type {
	pub fn is_prim(&self) -> bool {
		match *self {
			Type::Arr(_) => false,
			Type::Class(_,_,_) => false,
			Type::Fn(_,_,_) => false,
			_ => true
		}
	}
	pub fn is_unk(&self) -> bool {
		match *self {
			Type::Unk => true,
			_ => false
		}
	}
	pub fn is_class(&self) -> bool {
		match *self {
			Type::Class(_,_,_) => true,
			_ => false
		}
	}
	pub fn c_name(&self) -> &String {
		match *self {
			Type::Class(_,ref a,_) => a, 
			_ => panic!()
		}
	}
	pub fn c_prefix(&self) -> &Vec<String> {
		match *self {
			Type::Class(ref a,_,_) => a,
			_ => panic!()
		}
	}
	pub fn c_params(&self) -> Option<&Vec<Type>> {
		match *self {
			Type::Class(_,_,Some(ref v)) => Some(v),
			_ => None
		}
	}
	pub fn components(&self, res : &mut Vec<*const Type>) {
		match *self {
			Type::Arr(ref a) => res.push(&**a),
			Type::Class(_,_,Some(ref v)) => {
				for t in v.iter() {
					res.push(t)
				}
			},
			Type::Fn(_, ref args, ref res_t) => {
				res.push(&**res_t);
				for t in args.iter() {
					res.push(t);
				}
			},
			_ => ()
		}
	}
}

impl fmt::Debug for Type {
	fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Type::Unk  => write!(f, "(?)"),
			Type::Int  => write!(f, "int"),
			Type::Real => write!(f, "real"),
			Type::Char => write!(f, "char"),
			Type::Str  => write!(f, "str"),
			Type::Bool => write!(f, "bool"),
			Type::Void => write!(f, "()"),
			Type::Arr(ref val) => write!(f, "[{:?}]", val),
			Type::Class(ref pref, ref name, ref tmpl) => {
				if pref.len() == 0 {
					try!(write!(f, "_::"));
				} else {
					for a in pref.iter() {
						try!(write!(f, "{}::", a));
					}
				}
				match *tmpl {
					Some(ref val) => write!(f, "{:?}{:?}{:?}", pref, name, val),
					_             => write!(f, "{:?}{:?}", pref, name)
				}
			},
			Type::Fn(ref t, ref args, ref res) =>
				match *t {
					None => write!(f, "Fn{:?}:{:?}", args, res),
					Some(ref v) => write!(f, "Fn<{:?}>{:?}:{:?}", v, args, res)
				}
		}
	}
}

pub type Tmpl = Vec<String>;

pub fn parse_type(lexer : &Lexer, curs : &Cursor) -> SynRes<Type> {
	let ans = lex!(lexer, curs);
	match &*ans.val {
		"int"  => syn_ok!(Type::Int, ans.cursor),
		"real" => syn_ok!(Type::Real, ans.cursor),
		"char" => syn_ok!(Type::Char, ans.cursor),
		"str"  => syn_ok!(Type::Str, ans.cursor),
		"bool" => syn_ok!(Type::Bool, ans.cursor),
		"Fn"   => { // FUNC
			let args = try!(parse_list(lexer, &ans.cursor, &parse_type, "(", ")"));
			let curs = lex!(lexer, &args.cursor, ":");
			let res = try!(parse_type(lexer, &curs));
			syn_ok!(Type::Fn(None, args.val, Box::new(res.val)), res.cursor);
		},
		"["    => { // ARRAY
			let inner = try!(parse_type(lexer, &ans.cursor));
			let out = lex!(lexer, &inner.cursor, "]");
			let res = Type::Arr(Box::new(inner.val));
			syn_ok!(res, out);
		},
		"("    => { // VOID
			let rest = lex!(lexer, &ans.cursor, ")");
			syn_ok!(Type::Void, rest)
		},
		_      => { // CLASS
			// this may be 'Class' or 'Class<A,B...>
			let c = ans.val.chars().next().unwrap();
			// class name must start with uppercase
			let mut acc = vec![];
			let mut curs = curs.clone();
			macro_rules! class_fin {
				($pars:expr, $curs:expr) => {{
					let name = acc.pop().unwrap();
					syn_ok!(Type::Class(acc, name, $pars), $curs)
				}};
			}
			loop {
				let ans = lex!(lexer, &curs);
				if ans.kind == LexTP::Id && (c >= 'A' && c <= 'Z') {
					//let name = ans.val;
					acc.push(ans.val);
					match lexer.lex(&ans.cursor) {
						Err(_) => class_fin!(None, ans.cursor), //syn_ok!(Type::Class(acc,None), ans.cursor),
						Ok(sym) => {
							if sym.val == "::" {
								curs = sym.cursor;
							} else if sym.val == "<" {
								//let ans = try!(parse_tmpl(lexer, &ans.cursor));
								let ans : SynAns<Vec<Type>> = try!(parse_list(lexer, &ans.cursor, &parse_type, "<", ">"));
								//syn_ok!(Type::Class(acc, Some(ans.val)), ans.cursor);
								class_fin!(Some(ans.val), ans.cursor)
							} else {
								//syn_ok!(Type::Class(acc,None), ans.cursor)
								class_fin!(None, ans.cursor)
							}
						}
					}
				} else {
					syn_throw!(format!("Bad class name '{}'", ans.val), curs);
				}
			}
		}
	}
}

#[inline(always)]
pub fn parse_tmpl(lexer : &Lexer, curs : &Cursor) -> SynRes<Vec<String>> {
	fn parse_tp(lexer : &Lexer, curs : &Cursor) -> SynRes<String> {
		let sym = lex_type!(lexer, curs, LexTP::Id);
		let c = sym.val.chars().next().unwrap();
		if c >= 'A' && c <= 'Z' {
			syn_ok!(sym.val, sym.cursor);
		} else {
			syn_throw!("template name must start with high", curs);
		}
	}
	parse_list(lexer, &curs, &parse_tp, "<", ">")
}
