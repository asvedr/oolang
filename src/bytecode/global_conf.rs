use type_check::tclass::*;
use bytecode::exc_keys::*;
use std::rc::Rc;
use std::collections::HashMap;

pub struct GlobalConf {
	pub excepts : ExcKeys,
	pub classes : HashMap<String, Rc<TClass>>
}

impl GlobalConf {
	pub fn new(c : usize) -> GlobalConf {
		GlobalConf{
			excepts : ExcKeys::new(c),
			classes : HashMap::new()
		}
	}
}

