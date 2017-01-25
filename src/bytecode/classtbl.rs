use syn::type_sys::*;
use type_check::tclass::*;

pub struct ClassTbl {
	props    : HashMap<String, usize>, // exclude virtuals
	virtuals : HashMap<String, usize>,
//	meths    : HashMap<String, String>,
}

impl ClassTbl {
	pub fn create(tclass : &TClass) -> ClassTbl {
		let mut ind = 0;
		let mut props =
		let mut virts =
		
	}
	pub fn table_size(&self) -> usize {
		self.props.len() + self.virtuals.len()
	}
	#[inline(always)]
	pub fn property(&self, name : &String) -> Option<usize> {
		match self.props.get(name) {
			Some(l) => Some(*l),
			_ => None
		}
	}
	#[inline(always)]
	pub fn virtual(&self, name : &String) -> Option<usize> {
		match self.virtuals.get(name) {
			Some(l) => Some(*l),
			_ => None
		}
	}
}
