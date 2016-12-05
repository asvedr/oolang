use syn::*;
use type_check::pack::*;
use type_check::tclass::*;
use std::collections::{HashSet, BTreeMap};
use std::fmt::Write;

macro_rules! throw {
	($mess:expr, $curs:expr) => {syn_throw!($mess, $curs)};
}

macro_rules! ok {() => {return Ok(())};}

type VMap = BTreeMap<String, Result<*const Type, *mut Type>>;

type CheckRes    = Result<(),Vec<SynErr>>;
type CheckAns<A> = Result<A,Vec<SynErr>>;

pub struct FunEnv {
	pub global      : *const Pack,
	pub local       : VMap, 
	pub outers      : VMap,
	pub templates   : HashSet<String>,    // local templates
	pub ret_type    : Option<*const Type>,
	pub loop_labels : Vec<*const String>  // for 'break' cmd
}

impl FunEnv {
	pub fn new(pack : *const Pack) -> FunEnv {
		FunEnv {
			global      : pack,
			local       : BTreeMap::new(),
			outers      : BTreeMap::new(),
			templates   : HashSet::new(),
			ret_type    : None,
			loop_labels : Vec::new()
		}
	}
	pub fn add_outer(&mut self, out : &FunEnv) {
		for name in out.local.keys() {
			self.outers.insert(name.clone(), out.local.get(name).unwrap().clone());
		}
		for name in out.outers.keys() {
			if !self.outers.contains_key(name) {
				self.outers.insert(name.clone(), out.outers.get(name).unwrap().clone());
			}
		}
		for name in out.templates.iter() {
			self.templates.insert(name.clone());
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
						_ => panic!("replace_unk: var known: {}", name)
					}
				},
			_ =>
				match self.outers.get(name) {
					Some(ans) => 
						unsafe {
							match *ans {
								Err(ref ptr) => **ptr = tp.clone(),
								_ => panic!("replace_unk: var known: {}", name)
							}
						},
					_ => panic!("replace_unk: var out: {}", name)
				}
		}
	}
	pub fn get_local_var(&self, name : &String) -> &Type {
		match self.local.get(name) {
			Some(v) =>
				match *v {
					Ok(l)  => unsafe { &*l },
					Err(l) => unsafe { &*l }
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
									None => {
										throw!(format!("var {} not found", name), pos)
									}
								}
							}
						}
				}
			},
			Some(ref mut arr) => unsafe {
				if arr[0] == "%loc" {
					*tp_dst = clone_type!(self.local.get(name).unwrap());
					ok!()
				} else if arr[0] == "%out" {
					*tp_dst = clone_type!(self.outers.get(name).unwrap());
					ok!()
				} else if arr[0] == "%mod" {
					*tp_dst = (*(*self.global).get_fn(None, name).unwrap()).clone();
					ok!()
				}
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
			// IT'S TEMPLATE
				pref.push("%tmpl".to_string())
			} else {
			// IT'S IN IMPORTED SPACE
				unsafe {
					match (*self.global).get_cls(None, name) {
						Some(cls) => {
							let pcnt = match *params {Some(ref vec) => vec.len(), _ => 0};
							if (*cls).params.len() != pcnt {
								throw!(format!("class {:?}{} need {} params, given {}", pref, name, (*cls).params.len(), pcnt), pos)
							}
						},
						None => {
							throw!(format!("class {} not found", name), pos)
						}
					}
					match (*self.global).pack_of_cls(name) {
						None => pref.push("%mod".to_string()),
						Some(path) => *pref = path
					}
				}
			}
		} else {
		// IT'S IN AVAILABLE MODULES
			unsafe {
				match (*self.global).get_cls(Some(pref), name) {
					None => {
						throw!(format!("class {} not found", name), pos)
					},
					Some(cls) => {
						(*self.global).open_pref(pref);
						let pcnt = match *params {Some(ref vec) => vec.len(), _ => 0};
						if (*cls).params.len() != pcnt {
							throw!(format!("class {:?}{} need {} params, given {}", pref, name, (*cls).params.len(), pcnt), pos)
						}
					}
				}
			}
		}
		ok!()
	}
	pub fn get_method(&self, cls : &Type, mname : &String, priv_too : bool) -> Option<Type> {
		unsafe {
			match *cls {
				Type::Class(ref pref, ref cname, ref params) => {
					let cls : *const TClass =
						if pref.len() == 0 {
							// PREFIX NOT EXIST OR IT'S A TEMPLATE TYPE
							if self.templates.contains(cname) {
								return None
							} else {
							// IT'S IN IMPORTED SPACE
								match (*self.global).get_cls(None, cname) {
									Some(cls) => cls,
									None => return None
								}
							}
						} else {
							// IT'S IN AVAILABLE MODULES
							match (*self.global).get_cls(Some(pref), cname) {
								None => return None,
								Some(cls) => cls
							}
						};
					let params = match *params {
						None => None,
						Some(ref p) => Some(p)
					};
					let m = if priv_too { (*cls).look_in_all(mname, params) } else { (*cls).look_in_pub(mname, params) };
					match m {
						Some(res) =>
							match res {
								Ok(lnk) => return Some((*lnk).clone()),
								Err(t) => return Some(t)
							},
						None => return None
					}
				},
				Type::Arr(ref params) => {
					let cname = format!("%arr");
					let cls = match (*self.global).get_cls(None, &cname) { Some(c) => c, _ => panic!() };
					let m = if priv_too { (*cls).look_in_all(mname, Some(params)) } else { (*cls).look_in_pub(mname, Some(params)) };
					match m {
						Some(res) =>
							match res {
								Ok(lnk) => return Some((*lnk).clone()),
								Err(t) => return Some(t)
							},
						None => return None
					}
				},
				_ => return None
			}
		}
	}
}
