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
	pub params : Vec<String>,                  // count of params
	pub args   : Vec<*const Type>              // constructor 
}

impl TClass {
	pub fn look_in_all(&self, name : &String, tmpl : Option<&Vec<Type>>) -> Option<Result<*const Type, Type>> {
		unsafe {
			let lnk = match self.pubs.get(name) {
				Some(lnk) => *lnk,
				None =>
					match self.privs.get(name) {
						Some(lnk) => *lnk,
						None =>
							match self.parent {
								Some(lnk) => return (*lnk).look_in_all(name, tmpl),
								None => return None
							}
					}
			};
			match tmpl {
				Some(vec) => Some(self.replace_type(&*lnk, vec, true)),
				_ => Some(Ok(lnk))
			}
		}
	}
	pub fn look_in_pub(&self, name : &String, tmpl : Option<&Vec<Type>>) -> Option<Result<*const Type, Type>> {
		unsafe {
			let lnk = match self.pubs.get(name) {
				Some(lnk) => *lnk,
				None =>
					match self.parent {
						Some(lnk) => return (*lnk).look_in_pub(name, tmpl),
						None => return None
					}
			};
			match tmpl {
				Some(vec) => Some(self.replace_type(&*lnk, vec, true)),
				_ => Some(Ok(lnk))
			}
		}
	}
	fn replace_type(&self, src : &Type, args : &Vec<Type>, top : bool) -> Result<*const Type, Type> {
		macro_rules! get_i {($tp:expr) => {{
			let mut ans = None;
			for i in 0 .. self.params.len() {
				if *self.params[i] == *$tp {
					ans = Some(i);
					break;
				}
			}
			ans
		}};}
		match *src {
			Type::Arr(ref p) => {
				match self.replace_type(&p[0], args, false) {
					Ok(_)  => Ok(&*src),
					Err(t) => Err(Type::Arr(vec![t]))
				}
			},
			Type::Class(ref pref, ref name, ref params) => {
				let i = if pref.len() == 0 {
					get_i!(name)
				} else {
					None
				};
				match i {
					Some(i) if top => {		
						Ok(&args[i])
					},
					Some(i) => {
						Err(args[i].clone()) // 'cause if use ok-link, then parent method will not construct type
					},
					None => {
						match *params {
							None => Ok(&*src),
							Some(ref list) => {
								let mut params = vec![];
								let mut was    = false;
								for p in list.iter() {
									match self.replace_type(p, args, false) {
										Ok(l) => params.push(Ok(l)),
										Err(t) => {
											was = true;
											params.push(Err(t));
										}
									}
								}
								if was {
									let mut params_r = vec![];
									for p in params {
										match p {
											Ok(p) => unsafe { params_r.push((*p).clone()) },
											Err(p) => params_r.push(p)
										}
									}
									return Err(Type::Class(pref.clone(), name.clone(), Some(params_r)));
								} else {
									return Ok(&*src);
								}
							}
						}
					},
				}
			},
			Type::Fn(_, ref pars, ref res) => {
				match self.replace_type(&**res, args, false) {
					Ok(_) => {
						let mut args_p = vec![];
						let mut was = false;
						for p in pars.iter() {
							match self.replace_type(p, args, false) {
								Ok(l) => args_p.push(Ok(l)),
								Err(t) => {
									was = true;
									args_p.push(Err(t));
								}
							}
						}
						if was {
							let mut args_r = vec![];
							for a in args_p {
								match a {
									Ok(l) => unsafe { args_r.push((*l).clone()) },
									Err(t) => args_r.push(t)
								}
							}
							return Err(type_fn!(args_r, (**res).clone()));
						} else {
							return Ok(&*src);
						}
					},
					Err(res) => {
						let mut args_p = vec![];
						for p in pars.iter() {
							match self.replace_type(p, &args, false) {
								Ok(l) => unsafe { args_p.push((*l).clone()) },
								Err(t) => args_p.push(t)
							}
						}
						return Err(type_fn!(args_p, res));
					}
				}
			},
			_ => Ok(&*src)
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
		match $loc_e.local.insert($name.clone(), Err($tp)) {
			Some(_) => throw!(format!("local var {} already exist", $name), $pos),
			_ => ()
		}
	};
}
#[macro_export]
macro_rules! add_loc_knw {
	($loc_e:expr, $name:expr, $tp:expr, $pos:expr) => {
		match $loc_e.local.insert($name.clone(), Ok($tp)) {
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
	pub fn add_outer(&mut self, out : &LocEnv) {
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

pub type CheckRes    = Result<(),Vec<SynErr>>;
pub type CheckAns<A> = Result<A,Vec<SynErr>>;

pub fn find_unknown(body : &Vec<ActF>) -> &Cursor {	
	macro_rules! go_e {($e:expr) => {match check($e) {Some(p) => return Some(p) , _ => ()}};}
	macro_rules! go_a {($e:expr) => {match rec($e) {Some(p) => return Some(p) , _ => ()}};}
	fn check(e : &Expr) -> Option<&Cursor> {	
		if e.kind.is_unk() {
			Some(&e.addres)
		} else {
			match e.val {
				EVal::Call(_, ref f, ref a) => {
					go_e!(f);
					for i in a.iter() {
						go_e!(i);
					}
				},
				EVal::NewClass(_,_,_,ref args) => {
					for a in args.iter() {
						go_e!(a);
					}
				},
				EVal::Item(ref a, ref b) => {
					go_e!(a);
					go_e!(b);
				},
				EVal::Arr(ref items) =>
					for i in items {
						go_e!(i);
					},
				EVal::Asc(ref pairs) => {
					for pair in pairs {
						go_e!(&pair.a);
						go_e!(&pair.b);
					}
				},
				EVal::Prop(ref a, _) => go_e!(a),
				EVal::ChangeType(ref a, _) => go_e!(a),
				_ => ()
			}
			None
		}
	}
	fn rec(body : &Vec<ActF>) -> Option<&Cursor> {
		for act in body.iter() {
			match act.val {
				ActVal::Expr(ref e) => go_e!(e),
				ActVal::DFun(ref dfun) => go_a!(&dfun.body),
				ActVal::DVar(_,_,ref oe) => for e in oe.iter() { go_e!(e) },
				ActVal::Asg(ref a, ref b) => {
					go_e!(a);
					go_e!(b);
				},
				ActVal::Ret(ref oe) => for e in oe.iter() { go_e!(e) },
				ActVal::While(_, ref e, ref a) => {
					go_e!(e);
					go_a!(a);
				},
				ActVal::For(_,_,ref e1,ref e2,ref a) => {
					go_e!(e1);
					go_e!(e2);
					go_a!(a);
				},
				ActVal::Foreach(_,_,ref e,ref a) => {
					go_e!(e);
					go_a!(a);
				},
				ActVal::If(ref e, ref a, ref b) => {
					go_e!(e);
					go_a!(a);
					go_a!(b);
				},
				ActVal::Try(ref a, ref ctchs) => {
					go_a!(a);
					for c in ctchs.iter() {
						go_a!(&c.act);
					}
				},
				ActVal::Throw(ref e) => go_e!(e),
				_ => ()
			}
		}
		None
	}
	match rec(body) {
		Some(a) => a,
		_ => panic!()
	}
}
