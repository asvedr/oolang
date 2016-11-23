use syn_common::*;
use std::collections::{HashMap, HashSet, BTreeMap};
use std::fmt::Write;

#[macro_export]
macro_rules! throw {
	($mess:expr, $curs:expr) => {syn_throw!($mess, $curs)};
}

#[macro_export]
macro_rules! ok {() => {return Ok(())};}

pub struct TClass {
	pub parent : Option<*const TClass>,
	pub privs  : BTreeMap<String,*const Type>, // orig type saved in syn_class
	pub pubs   : BTreeMap<String,*const Type>, 
	pub params : usize,                        // count of params
	pub args   : Vec<*const Type>              // constructor 
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
	pub fn show(&self) -> String {
		let mut out = String::new();
		let _ = write!(out, "pack: {:?}\nusing: [", self.name);
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
}

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
				let pack = pack_of!($_self, pref);
				match pack {
					Some(ptr) =>
						match (*ptr).$map.get($name) {
							Some(ans) => Some(&*ans),
							None => None
						},
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
}

pub struct LocEnv {
	pub global     : *const Pack,
	pub local      : BTreeMap<String, Result<*const Type, *mut Type>>, // Ok  (WE TRULY KNOW WHAT IT IS)
	pub outers     : BTreeMap<String, Result<*const Type, *mut Type>>, // Err (WE CALCULATED THIS AND WE CAN MISTAKE)
	pub templates  : HashSet<String>,                                  // local templates
	pub ret_type   : Option<*const Type>
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
			global    : pack,
			local     : BTreeMap::new(),
			outers    : BTreeMap::new(),
			templates : HashSet::new(),
			ret_type  : None
		}
	}
	pub fn set_ret_type(&mut self, t : &Type) {
		self.ret_type = Some(&*t);
	}
	pub fn check_ret_type(&self, t : &Type) -> bool {
		match self.ret_type {
			Some(ref t1) => unsafe{ **t1 == *t },
			_ => false
		}
	}
	pub fn ret_type(&self) -> &Type {
		match self.ret_type {
			Some(ref t) => unsafe { &**t },
			_ => panic!()
		}
	}
	pub fn show(&self) -> String {
		let mut out = String::new();
		let _ = write!(out, "LocEnv:\ntempls: [");
		for name in self.templates.iter() {
			let _ = write!(out, "{}, ", name);
		}
		let _ = write!(out, "]\nlocal: [");
		for name in self.local.keys() {
			let _ = write!(out, "{}, ", name);
		}
		let _ = write!(out, "]\nouter: [");
		for name in self.local.keys() {
			let _ = write!(out, "{}, ", name);
		}
		let _ = write!(out, "]\n");
		out

	}
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
								//println!("LOOK FOR {} {}", name, unsafe {(*self.global).show()} );
								match (*self.global).get_fn(None, name) {
									Some(t) => {
										//println!("FOUND {}", name);
										*tp_dst = (*t).clone();
										*pref = match (*self.global).pack_of_fn(name) {
											Some(p) => Some(p),
											None => THISMOD!()
										};
										ok!()
									},
									None => {
										//println!("NOT FOUND {}", name);
										throw!(format!("var {} not found", name), pos)
									}
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
	pub fn check_class(&self, pref : &mut Vec<String>, name : &String, params : &Option<Vec<Type>>, pos : &Cursor) -> CheckRes {
		if pref.len() == 0 {
			// PREFIX NOT EXIST OR IT'S A TEMPLATE TYPE
			if self.templates.contains(name) {
				pref.push("%tmpl".to_string())
			} else {
				unsafe {
					match (*self.global).get_cls(None, name) {
						Some(cls) => {
							let pcnt = match *params {Some(ref vec) => vec.len(), _ => 0};
							if (*cls).params != pcnt {
								throw!(format!("class {:?}{} need {} params, given {}", pref, name, (*cls).params, pcnt), pos)
							}
						},
						None => throw!(format!("class {} not found", name), pos)
					}
					match (*self.global).pack_of_cls(name) {
						None => pref.push("%mod".to_string()),
						Some(path) => *pref = path
					}
				}
			}
		} else {
			unsafe {
				match (*self.global).get_cls(Some(pref), name) {
					None => throw!(format!("class {} not found", name), pos),
					Some(cls) => {
						(*self.global).open_pref(pref);
						let pcnt = match *params {Some(ref vec) => vec.len(), _ => 0};
						if (*cls).params != pcnt {
							throw!(format!("class {:?}{} need {} params, given {}", pref, name, (*cls).params, pcnt), pos)
						}
					}
				}
			}
		}
		ok!()
	}
}

pub type CheckRes    = Result<(),Vec<SynErr>>;
pub type CheckAns<A> = Result<A,Vec<SynErr>>;
