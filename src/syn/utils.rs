//#[macro_use]
use syn::reserr::*;
//use syn::lexer::*;

pub type Parser<A> = Fn(&Lexer,&Cursor) -> SynRes<A>;

pub fn parse_list<Item>(lexer : &Lexer, curs : &Cursor, item_parser : &Parser<Item>, start_signal : &str, stop_signal : &str) -> SynRes<Vec<Item>> {
	let mut curs : Cursor = lex!(lexer, curs, start_signal);
	let mut acc = vec![];
	let ans = lex!(lexer, &curs);
	if ans.val == stop_signal {
		syn_ok!(acc, ans.cursor);
	}
	loop {
		let ans = syn_try!(item_parser(lexer, &curs));
		acc.push(ans.val);
		curs = ans.cursor;
		let ans = lex!(lexer, &curs);
		if ans.val == stop_signal {
			syn_ok!(acc, ans.cursor);
		} else if ans.val == "," {
			curs = ans.cursor;
		} else {
			syn_throw!(format!("expected ',' or '{}'", stop_signal), curs)
		}
	}
}

#[derive(Clone)]
pub struct Pair<A,B> {
	pub a : A,
	pub b : B
}

#[macro_export]
macro_rules! pair_parser {
	($key_p:expr, $val_p:expr, $pair_sig:expr) => {
		move|lexer : &Lexer, curs : &Cursor| {
			let ans_a = syn_try!($key_p(lexer,curs));
			let curs  = lex!(lexer, &ans_a.cursor, $pair_sig);
			let ans_b = syn_try!($val_p(lexer, &curs));
			syn_ok!(Pair{a : ans_a.val, b : ans_b.val}, ans_b.cursor);
		};
	};
}

pub trait Show {
	fn show(&self, usize) -> Vec<String>;
}
