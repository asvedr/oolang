use syn_reserr::*;
use syn_utils::*;
use std::fmt;

#[derive(Clone,PartialEq)]
pub enum Type {
	Int,
	Real,
	Char,
	Str,
	Bool,
	Void,
	Arr(Box<Type>),
	Class(String,Option<Tmpl>),
	Fn(Vec<Type>, Box<Type>)
}

impl fmt::Debug for Type {
	fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Type::Int  => write!(f, "int"),
			Type::Real => write!(f, "real"),
			Type::Char => write!(f, "char"),
			Type::Str  => write!(f, "str"),
			Type::Bool => write!(f, "bool"),
			Type::Void => write!(f, "()"),
			Type::Arr(ref val) => write!(f, "[{:?}]", val),
			Type::Class(ref name, ref tmpl) =>
				match *tmpl {
					Some(ref val) => write!(f, "{:?}", val),
					_         => name.fmt(f)
				},
			Type::Fn(ref args, ref res) => write!(f, "Fn{:?}:{:?}", args, res)
		}
	}
}

pub type Tmpl = Vec<Type>;

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
			syn_ok!(Type::Fn(args.val, Box::new(res.val)), res.cursor);
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
			if ans.kind == LexTP::Id && (c >= 'A' && c <= 'Z') {
				let name = ans.val;
				match parse_tmpl(lexer, &ans.cursor) {
					Ok(ans) => {
						let rest = ans.cursor;
						syn_ok!(Type::Class(name, Some(ans.val)), rest);
					},
					_ => syn_ok!(Type::Class(name,None), ans.cursor)
				}
			} else {
				syn_throw!(format!("Bad class name '{}'", ans.val), curs);
			}
		}
	}
}

#[inline(always)]
pub fn parse_tmpl(lexer : &Lexer, curs : &Cursor) -> SynRes<Vec<Type>> {
	parse_list(lexer, &curs, &parse_type, "<", ">")
}
