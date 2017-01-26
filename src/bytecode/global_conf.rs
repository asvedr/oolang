use type_check::tclass::*;
use bytecode::exc_keys::*;
use bytecode::registers::*;
use bytecode::cmd::*;
//use std::rc::Rc;
pub use std::cell::Ref;
use std::collections::HashMap;

pub struct GlobalConf {
	pub excepts : ExcKeys,
	pub classes : HashMap<String,RTClass>
}

impl GlobalConf {
	pub fn new(c : usize) -> GlobalConf {
		GlobalConf{
			excepts : ExcKeys::new(c),
			classes : HashMap::new()
		}
	}
	pub fn add_class(&mut self, class : RTClass) {
		let name = {
			let mut c = class.borrow_mut();
			// XXX cause of info about #NoExcept
			//c.prepare_to_translation();
			c.fname.clone()
		};
		self.classes.insert(name, class);
	}
	// 'cause of on translation can't get class out of table
	/*
		use .get(name).get_virt_i  - to get slot of virtual or check 'is it virtual'
		use .get(name).method2name - to get fname of regular method
		use .get(name).prop_i      - to get slot of prop
	*/
	#[inline(always)]
	pub fn get(&self, name : &String) -> Ref<TClass> {
		match self.classes.get(name) {
			Some(val) => val.borrow(),
			_ => panic!()
		}
	}
}

