use type_check::tclass::*;
use bytecode::exc_keys::*;
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
			c.prepare_to_translation();
			c.fname.clone()
		};
		self.classes.insert(name, class);
	}
	// 'cause of on translation can't get class out of table
	#[inline(always)]
	pub fn get(&self, name : &String) -> Ref<TClass> {
		match self.classes.get(name) {
			Some(val) => val.borrow(),
			_ => panic!()
		}
	}
}

