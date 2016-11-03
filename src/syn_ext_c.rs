//use syn_utils::*;
use syn_reserr::*;
use type_sys::*;
use std::fmt;

pub type CType = String;

pub struct CFun {
	pub name   : String,
	pub cname  : String,
	pub ftype  : Type,
	pub addres : Cursor
}

impl fmt::Debug for CFun {
	fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
		write!(f, "CFUN {}:( {:?} ) = {}", self.name, self.ftype, self.cname)
	}
}

pub fn parse_c_type(lexer : &Lexer, curs : &Cursor) -> SynRes<CType> {
	let curs = lex!(lexer, curs, "type");
	let name = lex_type!(lexer, &curs, LexTP::Id);
	let c = name.val.chars().next().unwrap();
	if c >= 'A' && c <= 'Z' {
		syn_ok!(name.val, name.cursor);
	} else {
		syn_throw!("type name must start with high", curs);
	}
}

pub fn parse_c_fn(lexer : &Lexer, curs : &Cursor) -> SynRes<CFun> {
	let addr  = curs.clone();
	let curs  = lex!(lexer, curs, "fn");
	let name  = lex_type!(lexer, &curs, LexTP::Id);
	let curs  = lex!(lexer, &name.cursor, ":");
	let tp    = try!(parse_type(lexer, &curs));
	let curs  = lex!(lexer, &tp.cursor, "=");
	let cname = lex_type!(lexer, &curs, LexTP::Str);
	let cfun  = CFun{
		addres : addr,
		name   : name.val,
		cname  : cname.val,
		ftype  : tp.val
	};
	syn_ok!(cfun, cname.cursor);
}
