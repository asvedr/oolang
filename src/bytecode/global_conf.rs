use type_check::tclass::*;
use bytecode::exc_keys::*;
pub use std::cell::Ref;
use std::collections::HashMap;

// global values for module
pub struct GlobalConf {
	pub excepts : RExcKeys,
	pub classes : HashMap<String,RTClass>,
	pub fns     : HashMap<String,String> // map of full names
}

impl GlobalConf {
	pub fn new(exc : RExcKeys) -> GlobalConf {
		GlobalConf{
			excepts : exc,//ExcKeys::new(0),
			classes : HashMap::new(),
			fns     : HashMap::new()
		}
	}
	pub fn add_class(&mut self, class : RTClass) {
		let name = {
			let c = class.borrow_mut();
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
	pub fn get_class(&self, name : &String) -> Ref<TClass> {
		match self.classes.get(name) {
			Some(val) => val.borrow(),
			_ => panic!()
		}
	}
	#[inline(always)]
	pub fn get_fun(&self, name : &String) -> &String {
		match self.fns.get(name) {
			Some(val) => val,
			_ => panic!()
		}
	}
	#[inline(always)]
	pub fn get_exc(&self, pref : &Vec<String>, name : &String) -> usize {
		/*match self.excepts.borrow().map.get(name) {
			Some(n) => *n,
			_ => panic!()
		}*/
		self.excepts./*borrow().*/get(pref,name)
	}
	#[inline(always)]
	pub fn destroy(self) -> RExcKeys {
		self.excepts
	}
}

