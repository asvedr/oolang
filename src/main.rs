#![allow(dead_code)]
#![allow(mutable_transmutes)]

extern crate getopts;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod syn;
#[macro_use]
mod type_check;
mod preludelib;
mod bytecode;
mod cmd_args;
mod translate;
//mod translate;
//use std::io;
use std::io::{Read, Write};
use std::fs::File;
use syn::*;
use type_check::checker::*;
use bytecode::compiler::*;
use bytecode::exc_keys::*;
use preludelib::*;

fn main() {
    let args = match cmd_args::parse() {
        Ok(a) => a,
        Err(a) => {
            println!("{}", a);
            return;
        }
    };
    let mut source = String::new();
    match File::open(&args.input[0].path) {
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
            let mod_name = vec!["main".to_string()];
            {
                let ch = Checker::new();
                match ch.check_mod(&mut m, &mod_name) {
                    Err(e) => {
                        println!("TCHECK ERR ON line: {} column: {}", e[0].line + 1, e[0].column + 1);
                        println!("{}", e[0].mess);
                        return;
                    },
                    _ => {
                        println!("syn in res.syn");
                        let _ = write_file("res.syn", &*m.print_to_string());
                    }
                }
            }
            let mut excs = ExcKeys::new(0);
            excs.register_mod(&m, &mod_name);
            let prelude = Prelude::new();
            let cmplr = Compiler::new(&prelude, excs, mod_name.clone(), "c_out".to_string());
            //let mod_name = vec!["main".to_string()];
            let cmod = cmplr.compile_mod(&m);
            excs = cmplr.destroy();
            println!("asm in res.asm");
            let _ = write_file("res.asm", &*cmod.print_to_string());
            //if m.funs.len() > 0 {
            //    let cfun = compile_fun::compile(&m.funs[0]);
            //    cfun.print()
            //}
            println!("c in res.c");
            let _ = translate::cmod_to_c(&cmod, "res");
        },
        Err(vec) => {
            for e in vec {
                println!("ERR line: {} column: {}:", e.line + 1, e.column + 1);
                println!("{}", e.mess);
            }
        }
    }
}

fn write_file(name : &str, text : &str) -> std::io::Result<()> {
    let mut file = File::create(name)?;
    write!(file, "{}", text)?;
    Ok(())
}

fn with_file_w<A>(name : &str, act : &Fn(&mut File) -> std::io::Result<A>) -> std::io::Result<A> {
    let mut file = File::create(name)?;
    act(&mut file)
}