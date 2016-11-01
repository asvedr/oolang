mod lexer;
#[macro_use]
mod syn_reserr;
mod type_sys;
#[macro_use]
mod syn_utils;
mod syn_expr;
mod syn_act;
mod syn_fn;
mod syn_class;
//use lexer::*;
//use std::io;
use std::io::Read;
use std::fs::File;
use syn_utils::Show;
//use std::result::Result;

use syn_reserr::*;

fn main() {
	let mut source = String::new();
	match File::open("source.code") {
		Ok(mut hnd) =>
			match hnd.read_to_string(&mut source) {
				Err(e) => {
					println!("read source err: {}", e);
					return;
				},
				_ => ()
			},
		Err(_)  => panic!("can't open file")
	};
	let lxr = Lexer::new(&*source);
	let curs = Cursor::new();
	match syn_class::parse_class(&lxr, &curs) {
		Ok(ans) => {
			for line in ans.val.show(0) {
				println!("{}", line);
			}
		},
		Err(vec) => {
			for e in vec {
				println!("ERR line: {} column: {}:", e.line + 1, e.column + 1);
				println!("{}", e.mess);
			}
		}
	}
}
