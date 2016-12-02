#![allow(dead_code)]
#[macro_use]
mod syn;
#[macro_use]
mod type_check;
mod preludelib;
//use std::io;
use std::io::Read;
use std::fs::File;
use syn::*;
use type_check::checker::*;

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
	//let curs = Cursor::new();
	match parse_mod(&lxr) {
		Ok(mut m) => {
			/*for line in m.show(0) {
				println!("{}", line);
			}*/
			println!("CHECK");
			let ch = Checker::new();
			match ch.check_mod(&mut m) {
				Err(e) => {
					println!("TCHECK ERR ON line: {} column: {}", e[0].line + 1, e[0].column + 1);
					println!("{}", e[0].mess);
				},
				_ => {
					for line in m.show(0) {
						println!("{}", line);
					}
				}
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
