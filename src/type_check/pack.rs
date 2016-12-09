use syn::type_sys::*;
use syn::reserr::*;
use type_check::tclass::*;
use std::fmt::Write;
use std::collections::{HashMap, BTreeMap};

macro_rules! pack_of {
	($_self:expr, $pref:expr) => {{
		let mut cur : *const Pack = &*$_self;
		let mut fail = false;
		for name in $pref.iter() {// unsafe {
			match (*cur).packs.get(name) {
				Some(pack) => cur = *pack,
				None => {//panic!()
					fail = true;
					break;
				}
			}
		}//}
		if fail {None}
		else {Some(cur)}
	}};
}

macro_rules! get_obj {
	($_self:expr, $pref:expr, $name:expr, $map:ident, $out_map:ident) => {unsafe {
		match $pref {
			Some(pref) => {
				if pref[0] == "%mod" {
					match $_self.$map.get($name) {
						Some(ans) => Some(&*ans),
						_ => None
					}
				} else {
					let pack = pack_of!($_self, pref);
					match pack {
						Some(ptr) =>
							match (*ptr).$map.get($name) {
								Some(ans) => Some(&*ans),
								None => None
							},
						_ => None
					}
				}
			},
			None =>
				match $_self.$map.get($name) {
					Some(ans) => Some(&*ans),
					None =>
						match $_self.$out_map.get($name) {
							Some(pack) => Some(&*(**pack).$map.get($name).unwrap()),
							None => None
						}
				}
		}
	}};
}

macro_rules! find_import {
	($_self:expr, $name:expr, $map:ident, $out_map:ident) => {
		if $_self.$map.contains_key($name) {
			None
		} else {unsafe {
			match $_self.$out_map.get($name) {
				Some(pack) => Some((**pack).name.clone()),
				_ => panic!()
			}
		}}
	};
}

pub struct Pack {
	pub name    : Vec<String>,
	pub packs   : HashMap<String,*const Pack>,  // imports
	pub out_cls : HashMap<String,*const Pack>, // imports *
	pub out_fns : BTreeMap<String,*const Pack>, // imports *
	pub cls     : HashMap<String,TClass>,
	pub fns     : BTreeMap<String,Type>
}

impl Pack {
	pub fn new() -> Pack {
		Pack{
			name    : Vec::new(),
			packs   : HashMap::new(),
			out_cls : HashMap::new(),
			out_fns : BTreeMap::new(),
			cls     : HashMap::new(),
			fns     : BTreeMap::new()
		}
	}
	pub fn show(&self) -> String {
		let mut out = String::new();
		let _ = write!(out, "pack: {:?}\n", self.name);
		let _ = write!(out, "\nusing: [");
		for name in self.packs.keys() {
			let _ = write!(out, "{}, ", name);
		}
		let _ = write!(out, "]\nfns:\n");
		for name in self.fns.keys() {
			let _ = write!(out, "\t{} : {:?}\n", name, self.fns.get(name).unwrap());
		}
		return out;
	}
	pub fn get_cls(&self, pref : Option<&Vec<String>>, name : &String) -> Option<*const TClass> {
		get_obj!(self, pref, name, cls, out_cls)
	}
	pub fn get_fn(&self, pref : Option<&Vec<String>>, name : &String) -> Option<*const Type> {
		get_obj!(self, pref, name, fns, out_fns)
	}
	// changing arg
	pub fn open_pref(&self, pref : &mut Vec<String>) {
		let pack = unsafe {pack_of!(self, pref)};
		match pack {
			Some(ptr) => unsafe {*pref = (*ptr).name.clone()},
			_ => ()
		}
	}
	pub fn pack_of_fn(&self, name : &String) -> Option<Vec<String>> { // Some(pack) or None[it mean then fun is in self module]
		find_import!(self, name, fns, out_fns)
	}
	pub fn pack_of_cls(&self, name : &String) -> Option<Vec<String>> { // Some(pack) or None[it mean then class is in self module]
		find_import!(self, name, cls, out_cls)
	}
	pub fn check_class(&self, pref : &mut Vec<String>, name : &String, params : &Option<Vec<Type>>, pos : &Cursor) -> Result<(), Vec<SynErr>> {
		// .get_cls, .open_pref
		// GET OBJ
		let cls = if pref.len() == 0 {
			match self.get_cls(None, name) {
				None => {
					println!("3!");
					syn_throw!(format!("class {} not found", name), pos)
				},
				Some(cls) => cls
			}
		} else if pref[0] == "%tmpl" {
			// OUT OF MAIN BRANCH.
			// CHECK FOR ZERO PARAMS AND RETURN
			match *params {
				Some(_) => syn_throw!("template has more then 0 params", pos),
				_ => return Ok(())
			}
		} else {
			match self.get_cls(Some(pref), name) {
				None => {
					println!("4!");
					syn_throw!(format!("class {:?}{} not found", pref, name), pos)
				},
				Some(cls) => cls
			}
		};
		// CHECK COUNT
		let cnt1 = match *params {
			Some(ref v) => v.len(),
			_ => 0
		};
		let cnt2 = unsafe { (*cls).params.len() };
		if cnt1 != cnt2 {
			syn_throw!(format!("incorrect params count. Expect {}, found {}", cnt2, cnt1), pos)
		}
		// CHANGE PREF
		if pref.len() == 0 {
			match self.pack_of_cls(name) {
				Some(p) => *pref = p,
				_ => pref.push("%mod".to_string())
			}
		} else if pref[0] == "%mod" {
			// ok
		} else {
			self.open_pref(pref)
		}
		Ok(())
	}
}
