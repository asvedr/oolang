mod lexer;
#[macro_use]
mod syn_reserr;
mod type_sys;
#[macro_use]
mod syn_utils;
mod syn_expr;
use lexer::*;
use std::io;
use std::io::Read;
use std::fs::File;
use syn_utils::Show;
//use std::result::Result;

fn main() {
	let mut source = String::new();
	match File::open("source.code") {
		Ok(mut hnd) => hnd.read_to_string(&mut source),
		Err(_)  => panic!("can't open file")
	};
	let lxr = Lexer::new(&*source);
	let curs = Cursor::new();
	match syn_expr::parse_expr(&lxr, &curs) {
		Ok(ans) => {
			for line in ans.val.show(0) {
				println!("{}", line);
			}
		},
		Err(e) => println!("ERR {:?}", e)
	}
}
