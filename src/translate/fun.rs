use bytecode::compile_fun::*;
use bytecode::cmd::Cmd;
use translate::code;

use std::fs::File;
use std::io;
use std::io::Write;

// TODO add send info about local vars and env to code::to_c


pub fn to_c(fun : &CFun, out : &mut File) -> io::Result<()> {
	write!(out, "void {}(", fun.name)?;
	if fun.arg_cnt > 0 {
		write!(out, "Var arg0")?;
	}
	for i in 1 .. fun.arg_cnt  {
		write!(out, ", Var arg{}", i)?;
	}
	let mut finalizer = vec![];
	macro_rules! add_vars {
	 	($prefix:expr, $count:expr) => {
	 		for i in 0 .. $count {
	 			finalizer.push(format!("DECLINK({}{})", $prefix, i));
	 		}
	 	}
	}
	add_vars!("arg", fun.arg_cnt);
	add_vars!("v_var", fun.var_v);
	add_vars!("v_stack", fun.stack_v);
	for i in 0 .. fun.out_cnt {
		finalizer.push(format!("DECLINK(env[{}])", i));
	}
	write!(out, ") {}", "{\n")?;
	write!(out, "\t// INIT SECTION\n")?;
	// MACRO FOR DEFINE SEQUENCE OF LOCAL HOMOGENIC VARIABLES
	macro_rules! def_vars {
		($prefix:expr, $count:expr, $_type:expr, $init_val:expr) => {
			if $count > 0 {
				write!(out, "\t{} {}0={}", $_type, $prefix, $init_val)?;
				for i in 1 .. $count {
					write!(out, ", {}{}={}", $prefix, i, $init_val)?;
				}
				write!(out, ";\n")?;
			}
		}
	}
	// INIT LOCAL VARIABLES
	def_vars!("v_stack", fun.stack_v, "Var", "NULL");
	def_vars!("i_stack", fun.stack_i, "int", "0");
	def_vars!("r_stack", fun.stack_r, "double", "0");
	def_vars!("v_var", fun.var_v, "Var", "NULL");
	def_vars!("i_var", fun.var_i, "int", "0");
	def_vars!("r_var", fun.var_r, "double", "0");
	// DOES WE NEED LOCAL VAR FOR CLOSURES
	for cmd in fun.body.iter() {
		match *cmd {
			Cmd::MakeClos(_) => {
				// YES WE NEED
				write!(out, "\tClosure *closure;\n")?;
				break;
			},
			_ => ()
		}
	}
	write!(out, "\t// CODE SECTION\n")?;
	code::to_c(&fun.body, &finalizer, out)
	//write!(out, "{}", '}')
}