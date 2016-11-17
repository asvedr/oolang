use syn_common::*;
use std::collections::{HashMap, HashSet, BTreeMap};

#[macro_export]
macro_rules! throw {
	($mess:expr, $curs:expr) => {syn_throw!($mess, $curs)};
}

#[macro_export]
macro_rules! ok {() => {return Ok(())};}

pub struct TClass {
	pub parent : Option<*const TClass>,
	pub privs  : BTreeMap<String,*const Type>, // orig type saved in syn_class
	pub pubs   : BTreeMap<String,*const Type>
}

impl TClass {
	pub fn look_in_all(&self, name : &str) -> Option<*const Type> {
		unsafe {
			match self.pubs.get(name) {
				Some(lnk) => Some(*lnk),
				None =>
					match self.privs.get(name) {
						Some(lnk) => Some(*lnk),
						None =>
							match self.parent {
								Some(lnk) => (*lnk).look_in_all(name),
								None => None
							}
					}
			}
		}
	}
	pub fn look_in_pub(&self, name : &str) -> Option<*const Type> {
		unsafe {
			match self.pubs.get(name) {
				Some(lnk) => Some(*lnk),
				None =>
					match self.parent {
						Some(lnk) => (*lnk).look_in_pub(name),
						None => None
					}
			}
		}
	}
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
}

macro_rules! pack_of {
	($_self:expr, $pref:expr) => {unsafe {
		let mut cur : *const Pack = &*$_self;
		for name in $pref.iter() {
			match (*cur).packs.get(name) {
				Some(pack) => cur = *pack,
				None => panic!()
			}
		}
		cur
	}};
}

macro_rules! get_obj {
	($_self:expr, $pref:expr, $name:expr, $map:ident, $out_map:ident) => {unsafe {
		match $pref {
			Some(pref) => {
				let pack = pack_of!($_self, pref);
				match (*pack).$map.get($name) {
					Some(ans) => Some(&*ans),
					None => None
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

impl Pack {
	pub fn get_cls(&self, pref : Option<&Vec<String>>, name : &String) -> Option<*const TClass> {
		get_obj!(self, pref, name, cls, out_cls)
	}
	pub fn get_fn(&self, pref : Option<&Vec<String>>, name : &String) -> Option<*const Type> {
		get_obj!(self, pref, name, fns, out_fns)
	}
	// changing arg
	pub fn open_pref(&self, pref : &mut Vec<String>) {
		let pack = pack_of!(self, pref);
		unsafe {*pref = (*pack).name.clone()};
	}
	pub fn pack_of_fn(&self, name : &String) -> Option<Vec<String>> { // Some(pack) or None[it mean then fun is in self module]
		find_import!(self, name, fns, out_fns)
	}
	pub fn pack_of_cls(&self, name : &String) -> Option<Vec<String>> { // Some(pack) or None[it mean then class is in self module]
		find_import!(self, name, cls, out_cls)
	}
}

pub struct LocEnv {
	pub global     : *const Pack,
	pub local      : BTreeMap<String, Result<*const Type, *mut Type>>, // Ok  (WE TRULY KNOW WHAT IT IS)
	pub outers     : BTreeMap<String, Result<*const Type, *mut Type>>  // Err (WE CALCULATED THIS AND WE CAN MISTAKE)
}

#[macro_export]
macro_rules! add_loc_unk {
	($loc_e:expr, $name:expr, $tp:expr, $pos:expr) => {
		match $loc_e.local.insert($name.clone(), Ok($tp)) {
			Some(_) => throw!(format!("local var {} already exist", $name), $pos),
			_ => ()
		}
	};
}
#[macro_export]
macro_rules! add_loc_knw {
	($loc_e:expr, $name:expr, $tp:expr, $pos:expr) => {
		match $loc_e.local.insert($name.clone(), Err($tp)) {
			Some(_) => throw!(format!("local var {} already exist", $name), $pos),
			_ => ()
		}
	};
}

impl LocEnv {
	pub fn new(pack : *const Pack) -> LocEnv {
		LocEnv {
			global : pack,
			local  : BTreeMap::new(),
			outers : BTreeMap::new()
		}
	}
	/*fn check_class_use(&self, cls : &Type, curs : Cursor) -> CheckRes {
		match *cls {
		self.global.get_cls()
		}
	}*/
	pub fn replace_unk(&self, name : &String, tp : &Type) {
		match self.local.get(name) {
			Some(ans) =>
				unsafe {
					match *ans {
						Err(ref ptr) => **ptr = tp.clone(),
						_ => ()
					}
				},
			_ => panic!()
		}
	}
	pub fn get_var(&self, pref : &mut Option<Vec<String>>, name : &String, tp_dst : &mut Type, pos : &Cursor) -> CheckRes {
		macro_rules! LOCAL   { () => { Some(vec![("%loc").to_string()]) }; }
		macro_rules! OUTER   { () => { Some(vec![("%out").to_string()]) }; }
		macro_rules! THISMOD { () => { Some(vec![("%mod").to_string()]) }; }
		macro_rules! clone_type { ($t:expr) => {unsafe { match *$t {Ok(ref t) => (**t).clone(), Err(ref t) => (**t).clone()}} }; }
		match *pref {
			None => {
				match self.local.get(name) {
					Some(t) => {
						*tp_dst = clone_type!(t);
						*pref = LOCAL!();
						ok!()
					},
					None =>
						match self.outers.get(name) {
							Some(t) => {
								*tp_dst = clone_type!(t);
								*pref = OUTER!();
								ok!()
							},
							None => unsafe {
								match (*self.global).get_fn(None, name) {
									Some(t) => {
										*tp_dst = (*t).clone();
										*pref = match (*self.global).pack_of_fn(name) {
											Some(p) => Some(p),
											None => THISMOD!()
										};
										ok!()
									},
									None => throw!(format!("var {} not found", name), pos)
								}
							}
						}
				}
			},
			Some(ref mut arr) => unsafe {
				match (*self.global).get_fn(Some(arr), name) {
					Some(t) => {
						(*self.global).open_pref(arr);
						*tp_dst = (*t).clone();
						ok!()
					}
					None => {
						let mut fname = String::new();
						for p in arr.iter() {
							fname.push_str(&*p);
							fname.push_str("::");
						}
						fname.push_str(name);
						throw!(format!("var {} not found", fname), pos)
					}
				}
			}
		}
	}
}

pub type CheckRes    = Result<(),Vec<SynErr>>;
pub type CheckAns<A> = Result<A,Vec<SynErr>>;
