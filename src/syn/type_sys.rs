use syn::reserr::*;
use syn::utils::*;
//use syn::lexer::*;
use std::fmt;
use std::rc::Rc;

#[derive(Clone,PartialEq)]
pub enum Type {
	Unk, // UNKNOWN
	Int,
	Real,
	Char,
	Str,
	Bool,
	Void,
	Arr(Vec<RType>), // it easily to check then use box (Box<Type>),
	// Asc(Box<Type>,Box<Type>),
	//    prefix       name   template
	Class(Vec<String>,String,Option<Vec<RType>>),
	//  (template)?         args        result
	Fn(Option<Vec<String>>, Vec<RType>, RType),
}

pub type RType = Rc<Type>;

#[macro_export]
macro_rules! type_fn {
	($t:expr, $args:expr, $res:expr) => {
		Rc::new(Type::Fn(Some($t), $args, $res))
	};
	($args:expr, $res:expr) => {
		Rc::new(Type::Fn(None, $args, $res))
	};
}
#[macro_export]
macro_rules! type_c {
	($n:expr) => {Rc::new(Type::Class(vec![], $n, None))};
	($p:expr, $n:expr) => {Rc::new(Type::Class($p, $n, None))};
	($p:expr, $n:expr, $t:expr) => {Rc::new(Type::Class($p, $n, $t))}
}

macro_rules! check_is {($_self:expr, $t:ident) => {
	match *$_self {
		Type::$t => true,
		_ => false
	}
};}

impl Type {
	pub fn unk() -> RType { Rc::new(Type::Unk) }
	pub fn int() -> RType { Rc::new(Type::Int) }
	pub fn real() -> RType { Rc::new(Type::Real) }
	pub fn char() -> RType { Rc::new(Type::Char) }
	pub fn str() -> RType { Rc::new(Type::Str) }
	pub fn bool() -> RType { Rc::new(Type::Bool) }
	pub fn void() -> RType { Rc::new(Type::Void) }
	pub fn arr(a : RType) -> RType { Rc::new(Type::Arr(vec![a])) }
//	Class(Vec<String>,String,Option<Vec<RType>>),
	//  (template)?         args        result
//	Fn(Option<Vec<String>>, Vec<RType>, RType),

	pub fn is_int(&self)  -> bool {check_is!(self, Int)}
	pub fn is_real(&self) -> bool {check_is!(self, Real)}
	pub fn is_char(&self) -> bool {check_is!(self, Char)}
	pub fn is_str(&self)  -> bool {check_is!(self, Str)}
	pub fn is_bool(&self) -> bool {check_is!(self, Bool)}
	pub fn is_void(&self) -> bool {check_is!(self, Void)}
	pub fn is_unk(&self)  -> bool {check_is!(self, Unk)}
	pub fn is_asc(&self)  -> bool {
		match *self {
			Type::Class(ref p, ref n,_) => p.len() == 1 && p[0] == "%std" && n == "Asc",
			_ => false
		}
	}
	pub fn asc_key_val(&self) -> (RType, RType) {
		match *self {
			Type::Class(_,_,Some(ref params)) =>
				return (params[0].clone(), params[1].clone()),
			_ => panic!()
		}
	}
	pub fn is_arr(&self) -> bool {
		match *self {
			Type::Arr(_) => true,
			_ => false
		}
	}
	pub fn arr_item(&self) -> &RType {
		match *self {
			Type::Arr(ref i) => &i[0],
			_ => panic!()
		}
	}
	pub fn is_prim(&self) -> bool {
		match *self {
			Type::Arr(_) => false,
			Type::Class(_,_,_) => false,
			Type::Fn(_,_,_) => false,
			_ => true
		}
	}
	pub fn is_class(&self) -> bool {
		match *self {
			Type::Class(_,_,_) => true,
			_ => false
		}
	}
	pub fn class_name(&self) -> String {
		match *self {
			Type::Class(ref pref, ref name, _) => {
				let mut res = String::new();
				for p in pref.iter() {
					res = format!("{}{}_", res, p);
				}
				format!("{}{}", res, name)
			},
			Type::Arr(_) => "_std_vec".to_string(),
			Type::Str => "_std_str".to_string(),
			_ => panic!()
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
			Type::Arr(ref val) => write!(f, "[{:?}]", val[0]),
			Type::Class(ref pref, ref name, ref tmpl) => {
				if pref.len() == 0 {
					try!(write!(f, "_::"));
				} else {
					for a in pref.iter() {
						try!(write!(f, "{}::", a));
					}
				}
				match *tmpl {
					Some(ref val) => write!(f, "{}{:?}", name, val),
					_             => write!(f, "{}", name)
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

pub fn parse_type(lexer : &Lexer, curs : &Cursor) -> SynRes<RType> {
	let ans = lex!(lexer, curs);
	match &*ans.val {
		"int"  => syn_ok!(Rc::new(Type::Int), ans.cursor),
		"real" => syn_ok!(Rc::new(Type::Real), ans.cursor),
		"char" => syn_ok!(Rc::new(Type::Char), ans.cursor),
		"str"  => syn_ok!(Rc::new(Type::Str), ans.cursor),
		"bool" => syn_ok!(Rc::new(Type::Bool), ans.cursor),
		"Fn"   => { // FUNC
			let args = try!(parse_list(lexer, &ans.cursor, &parse_type, "(", ")"));
			let curs = lex!(lexer, &args.cursor, ":");
			let res = try!(parse_type(lexer, &curs));
			syn_ok!(Rc::new(Type::Fn(None, args.val, res.val)), res.cursor);
		},
		"["    => { // ARRAY
			let inner = try!(parse_type(lexer, &ans.cursor));
			let out = lex!(lexer, &inner.cursor, "]");
			let res = Type::Arr(vec![inner.val]);
			syn_ok!(Rc::new(res), out);
		},
		"("    => { // VOID
			let rest = lex!(lexer, &ans.cursor, ")");
			syn_ok!(Rc::new(Type::Void), rest)
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
					syn_ok!(Rc::new(Type::Class(acc, name, $pars)), $curs)
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
								let ans : SynAns<Vec<RType>> = try!(parse_list(lexer, &ans.cursor, &parse_type, "<", ">"));
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
