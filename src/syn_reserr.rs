pub use lexer::*;

pub struct SynAns<A> {
	pub cursor : Cursor,
	pub val    : A,
}

#[derive(Debug)]
pub struct SynErr {
	pub line   : usize,
	pub column : usize,
	pub mess   : String
}

#[macro_export]
macro_rules! syn_addr {($m:expr, $l:expr, $c:expr) => {SynAddres{module : $m, line : $l, column : $c}}; }

pub type SynRes<A> = Result<SynAns<A>, Vec<SynErr>>;

#[macro_export]
macro_rules! syn_try {
	($val:expr) => {
		match $val {
			Ok(ans) => ans,
			Err(e) => return Err(e)
		}
	};
	($val:expr, $curs:expr, $mess:expr) => {
		match $val {
			Ok(ans) => ans,
			Err(trace) => {
				trace.push(SynErr{line : $curs.line, column : $curs.column, mess : $mess});
				return Err(trace);
			}
		}
	};
	($val:expr, $mess:expr) => {
		match $val {
			Ok(ans) => ans,
			Err(trace) => {
				trace.push(SynErr{line : 0, column : 0, mess : $mess});
				return Err(trace);
			}
		}
	}
}

#[macro_export]
macro_rules! syn_throw {
	($val:expr) => {return Err(vec![SynErr{line : 0, column : 0, mess : $val.to_string()}])};
	($val:expr, $curs:expr) => {
		return Err(vec![SynErr{line : $curs.line, column : $curs.column, mess : $val.to_string()}]);
	};
	($val:expr, $l:expr, $c:expr) => {
		return Err(vec![SynErr{line : $l, column : $c, mess : $val.to_string()}]);
	};
}

#[macro_export]
macro_rules! syn_ok {
	($val:expr, $curs:expr) => {return Ok(SynAns{val : $val, cursor : $curs})};
}

#[macro_export]
macro_rules! lex {
	($lexer:expr, $c:expr) => {
		match $lexer.lex($c) {
			Ok(ans) => ans,
			Err(e) => syn_throw!(e.data, e.line, e.column)
		}
	};
	($lexer:expr, $c:expr, $expect:expr) => {
		match $lexer.lex($c) {
			Ok(ans) =>
				if &*ans.val == $expect {
					ans.cursor
				} else {
					syn_throw!(format!("expected '{}', found '{}'", $expect, ans.val), $c)
				},
			Err(e) => syn_throw!(e.data, e.line, e.column)
		}
	};
}

#[macro_export]
macro_rules! lex_type {
	($lexer:expr, $c:expr, $kind:expr) => {
		match $lexer.lex($c) {
			Ok(ans) =>
				if ans.kind == $kind {
					ans
				} else {
					syn_throw!(format!("expected {:?}, found '{}':{:?}", $kind, ans.val, ans.kind), $c)
				},
			Err(e) => syn_throw!(e.data, e.line, e.column)
		}
	};
}
