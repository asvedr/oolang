use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

// all exception keys are the numbers in C
// must make one NAME-CODE map for all modules in application
pub struct ExcKeys {
	map : HashMap<String,usize>,
	cnt : usize
}
pub type RExcKeys = Rc<RefCell<ExcKeys>>;

macro_rules! make_name{($pref:expr, $name:expr, $res:expr) => {{
	for i in $pref.iter() {
		$res = format!("{}{}_", $res, i);
	}
	$res = format!("{}{}_", $res, $name);
}};}

impl ExcKeys {
	#[inline(always)]
	pub fn get(&self, pref : &Vec<String>, name : &String) -> usize {
		let mut res = String::new();
		make_name!(pref, name, res);
		match self.map.get(&res) {
			Some(a) => *a,
			_ => panic!("bad exception key: {}", name)
		}
	}
	pub fn add(&mut self, pref : &Vec<String>, name : &String) {
		let mut res = String::new();
		make_name!(pref, name, res);
		self.map.insert(res, self.cnt);
		self.cnt += 1;
	}
	pub fn new(c : usize) -> RExcKeys {
		Rc::new(RefCell::new(ExcKeys {
			cnt : c,
			map : HashMap::new()
		}))
	}
}
