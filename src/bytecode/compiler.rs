use bytecode::global_conf::*;
use bytecode::state::*;
use bytecode::compile_fun::*;
use bytecode::exc_keys::*;
use preludelib::*;

pub struct Compiler {
	pub gc       : GlobalConf,
	pub dest_dir : String
}

impl Compiler {
	pub fn new(std : &Prelude, dest_dir : String) -> Compiler {
		let mut gc = GlobalConf::new(ExcKeys::new(0));
		//gc.fns     = std.pack.fns.clone();
		for (k,v) in std.cfns.iter() {
			gc.fns.insert(k.clone(), v.clone());
		}
		for tcls in std.pack.cls.values() {
			let name = tcls.borrow().fname.clone();
			gc.classes.insert(name, tcls.clone());
		}
		for e in std.pack.excepts.keys() {
			//let name = format!("{}_{}", std.full_name(), e);
			//gc.excepts.add(name);
			gc.excepts./*borrow_mut().*/add(&std.pack.name, e);
		}
		Compiler {
			gc       : gc,
			dest_dir : dest_dir
		}
	}
	//pub compile_mod(SynMod : &)
}
