use translate::fun;
use bytecode::compiler::*;

use std::fs::File;
use std::io::Write;
use std::io;

pub fn cmod_to_c(cmod : &CMod, fname : &str) -> io::Result<()> {
	let mut out = File::create(format!("{}.c", fname))?;
	write!(out, "#include \"{}.h\"\n\n", fname)?;

	for f in cmod.priv_fns.iter() {
		write!(out, "static ")?;
		fun::to_c(&f, &mut out)?;
	}
	for f in cmod.pub_fns.iter() {
		fun::to_c(&f, &mut out)?;
	}
	Ok(())
}