use syn_expr::*;
use syn_act::*;
use syn_utils::*;
use type_sys::*;
use lexer::*;
use syn_reserr::*;

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
	pub body        : Vec<Act>,
	pub addr        : Cursor,
	pub can_be_clos : bool // if has names args or option args then can't be used as closure
}

pub fn parse_fn_full(lexer : &Lexer, curs : &Cursor) -> SynRes<SynFn> {
	
}

pub fn parse_lambda(lexer : &Lexer, curs : &Cursor) -> SynRes<SynFn> {
	let curs = lex!(lexer, curs, "fn");
	parse_list(lexer, &curs, &parse_arg, "(", ")");
}
