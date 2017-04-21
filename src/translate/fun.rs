use bytecode::compile_fun::*;
use translate::code;

use std::fs::File;
use std::io;
use std::io::Write;

// TODO add send info about local vars and env to code::to_c


pub fn to_c(fun : &CFun, out : &mut File) -> io::Result<()> {
	write!(out, "Var {}(", fun.name)?;
	if fun.arg_cnt > 0 {
		write!(out, "Var arg0")?;
	}
	for i in 1 .. fun.arg_cnt  {
		write!(out, ", Var arg{}", i)?;
	}
	let mut finalizer = vec![];
	macro_rules! add_var {
	 	($prefix:expr, $count:expr) => {
	 		for i in 0 .. $count {
	 			finalizer.push(format!("DECLINK({}{})", $prefix, i));
	 		}
	 	}
	} 
	add_var!("arg", fun.arg_cnt);
	add_var!("var", fun.var_v);
	add_var!("stack", fun.stack_v);
	for i in 0 .. fun.out_cnt {
		finalizer.push(format!("DECLINK(env[{}])", i));
	}
	write!(out, ") {}", '{')?;
	code::to_c(&fun.body, &finalizer, out)?;
	write!(out, "{}", '}')
}