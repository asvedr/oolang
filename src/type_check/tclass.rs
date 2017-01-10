use syn::type_sys::*;
use syn::class::*;
use syn::reserr::*;
use std::collections::BTreeMap;
use std::rc::Rc;

pub struct Parent {
	class  : *const TClass,
	params : Option<*const Vec<RType>>
}

impl Parent {
	pub fn new(cls : *const TClass, pars : Option<*const Vec<RType>>) -> Parent {
		Parent {
			class  : cls,
			params : pars
		}
	}
}

pub struct Attr {
	pub _type     : RType,
	pub is_method : bool,
	pub is_no_exc : bool
}

impl Attr {
	pub fn method(t : RType, noexc : bool) -> Attr {
		Attr{
			_type : t,
			is_method : true,
			is_no_exc : noexc
		}
	}
	pub fn prop(t : RType) -> Attr {
		Attr {
			_type : t,
			is_method : false,
			is_no_exc : false
		}
	}
}

pub struct TClass {
	pub source : Option<*const Class>,
	pub parent : Option<Parent>,
	pub privs  : BTreeMap<String,Attr>, // orig type saved in syn_class
	pub pubs   : BTreeMap<String,Attr>, 
	pub params : Vec<String>,           // template
	pub args   : Vec<RType>             // constructor 
}

impl TClass {
	pub fn print(&self) {
		let mut tabs = String::new();
		macro_rules! addl {($($args:expr),+)  => {println!("{}{}",tabs,format!($($args,)+))};}
		macro_rules! attr {($name:expr, $attr:expr) => {
			if $attr.is_method {
				if $attr.is_no_exc {
					addl!("METHOD NO EXC: {} = {:?}", $name, *$attr._type);
				} else {
					addl!("METHOD: {} = {:?}", $name, *$attr._type);
				}
			} else {
				addl!("PROPERTY: {} = {:?}", $name, *$attr._type);
			}
		};}
		addl!("CLASS");
		tabs.push(' ');
		addl!("PARAMS:{:?}", self.params);
		addl!("ARGS:{:?}", self.args);
		match self.source {
			None => addl!("PARENT: NO"),
			_    => addl!("PARENT: YES")
		}
		addl!("PRIVS");
		tabs.push(' ');
		for name in self.privs.keys() {
			attr!(name, self.privs.get(name).unwrap())
		}
		tabs.pop();
		addl!("PUBS");
		tabs.push(' ');
		for name in self.pubs.keys() {
			attr!(name, self.pubs.get(name).unwrap());
		}
	}

	pub fn from_syn(cls : &Class, parent : Option<Parent>) -> Result<TClass,Vec<SynErr>> {
		let mut privs : BTreeMap<String, Attr> = BTreeMap::new();
		let mut pubs  : BTreeMap<String, Attr> = BTreeMap::new();
		macro_rules! make {($par:expr) => {{
		//	let fname = format!("init");
		//	match privs.get(&fname) {
		//		Some(_) => syn_throw!("initializer must be pub", cls.addres),
		//		_ => {
					/* let args = match pubs.get(&fname) {
						Some(t) => {
							match **t {
								Type::Fn(_,ref args_src,ref ret) => {
									let mut args : Vec<Type> = vec![];
									for a in args_src.iter() {
										args.push(a.clone());
									}
									if ret.is_void() {
										// ok
										args
									} else {
										syn_throw!("initializer must be void", cls.addres)
									}
								},
								_ =>
									syn_throw!(format!("initializer must be function, but it {:?}", **t), cls.addres)
							}
						},
						_ => {
							// ok
							Vec::new()
						}
					}; */
					// returning value
					return Ok(TClass {
						source : Some(&*cls),
						parent : $par,
						privs  : privs,
						pubs   : pubs,
						params : cls.template.clone(),
						args   : vec![]//args
					});
		//		}
		//	}
		}}; }
		macro_rules! foreach_prop{
			($seq_src:ident, $seq_dst:expr, $parent:expr) => {
				for prop in cls.$seq_src.iter() {
					if (*$parent).exist_attr(&prop.name) {
						syn_throw!("this prop exist in parent", prop.addres)
					} else {
						match $seq_dst.insert(prop.name.clone(), Attr::prop(prop.ptype.clone())) {
							Some(_) => syn_throw!(format!("this prop already exist"), prop.addres),
							_ => ()
						}
					}
				}
			};
			($seq_src:ident, $seq_dst:expr) => {
				for prop in cls.$seq_src.iter() {
					match $seq_dst.insert(prop.name.clone(), Attr::prop(prop.ptype.clone())) {
						Some(_) => syn_throw!(format!("this prop already exist"), prop.addres),
						_ => ()
					}
				}
			};
		}
		macro_rules! foreach_meth{
			($seq_src:ident, $seq_dst:expr, $parent:expr) => {
				for prop in cls.$seq_src.iter() {
					if (*$parent).exist_attr(&prop.func.name) {
						syn_throw!("this prop exist in parent", prop.func.addr)
					} else {
						match $seq_dst.insert(prop.func.name.clone(), Attr::method(prop.ftype.clone(), prop.func.no_except)) {
							Some(_) => syn_throw!(format!("this prop already exist"), prop.func.addr),
							_ => ()
						}
					}
				}
			};
			($seq_src:ident, $seq_dst:expr) => {
				for prop in cls.$seq_src.iter() {
					match $seq_dst.insert(prop.func.name.clone(), Attr::method(prop.ftype.clone(), prop.func.no_except)) {
						Some(_) => syn_throw!(format!("this prop already exist"), prop.func.addr),
						_ => ()
					}
				}
			};
		}
		unsafe {
			match parent {
				Some(par) => {
					foreach_prop!(priv_prop, privs, par.class);
					foreach_prop!(pub_prop, pubs, par.class);
					foreach_meth!(priv_fn, privs, par.class);
					foreach_meth!(pub_fn, pubs, par.class);
					make!(Some(par))
				},
				None => {
					foreach_prop!(priv_prop, privs);
					foreach_prop!(pub_prop, pubs);
					foreach_meth!(priv_fn, privs);
					foreach_meth!(pub_fn, pubs);
					make!(None)
				}
			}
		}
	}
	pub unsafe fn check_initializer(&mut self) -> Result<(),Vec<SynErr>> {	
		let fname = format!("init");
		let cls : &Class = match self.source {
			Some(ptr) => &*ptr,
			_ => panic!()
		};
		match self.privs.get(&fname) {
			Some(_) => syn_throw!("initializer must be pub", cls.addres),
			_ => {
				let args : Vec<RType> = match self.pubs.get(&fname) {
					Some(t) => {
						match *(t._type) {
							Type::Fn(_,ref args_src,ref ret) => {
								let mut args : Vec<RType> = vec![];
								for a in args_src.iter() {
									args.push(a.clone());
								}
								if ret.is_void() {
									// ok
									args
								} else {
									syn_throw!("initializer must be void", cls.addres)
								}
							},
							_ =>
								syn_throw!(format!("initializer must be function, but it {:?}", *(t._type)), cls.addres)
						}
					},
					_ => {
						// ok
						Vec::new()
					}
				};
				// setting args
				self.args = args;
				Ok(())
			}
		}
	}
	fn exist_attr(&self, name : &String) -> bool {
		let mut lnk : *const TClass = &*self;
		unsafe { loop {
			if (*lnk).privs.contains_key(name) || (*lnk).pubs.contains_key(name) {
				return true
			} else {
				match (*lnk).parent {
					Some(ref par) => lnk = par.class,
					_ => return false
				}
			}
		}}
	}
	pub fn look_in_all(&self, name : &String, tmpl : Option<&Vec<RType>>) -> Option<RType> {
		// TODO: rec to loop
		unsafe {
			let lnk = match self.pubs.get(name) {
				Some(lnk) => &lnk._type,
				None =>
					match self.privs.get(name) {
						Some(lnk) => &lnk._type,
						None =>
							match self.parent {
								Some(ref lnk) => return (*lnk.class).look_in_all(name, tmpl),
								None => return None
							}
					}
			};
			match tmpl {
				Some(vec) => Some(self.replace_type(lnk, vec, true)),
				_ => Some(lnk.clone())
			}
		}
	}
	pub fn look_in_pub(&self, name : &String, tmpl : Option<&Vec<RType>>) -> Option<RType> {
		// TODO: rec to loop
		unsafe {
			let lnk = match self.pubs.get(name) {
				Some(lnk) => &lnk._type,
				None =>
					match self.parent {
						Some(ref lnk) => return (*lnk.class).look_in_pub(name, tmpl),
						None => return None
					}
			};
			match tmpl {
				Some(vec) => Some(self.replace_type(lnk, vec, true)),
				_ => Some(lnk.clone())
			}
		}
	}
	// true if attr is method, false if prop. there is no check for existing here
	pub fn is_method(&self, name : &String) -> bool {
		unsafe {
			let mut lnk : *const TClass = &*self;
			loop {
				match (*lnk).pubs.get(name) {
					Some(p) => return p.is_method,
					_ =>
						match (*lnk).privs.get(name) {
							Some(p) => return p.is_method,
							_ =>
								match (*lnk).parent {
									Some(ref par) => lnk = par.class,
									_ => panic!()
								}
						}
				}
			}
		}
	}
	pub fn is_method_noexc(&self, name : &String) -> bool {
		unsafe {
			let mut lnk : *const TClass = &*self;
			loop {
				match (*lnk).pubs.get(name) {
					Some(p) => return p.is_no_exc,
					_ =>
						match (*lnk).privs.get(name) {
							Some(p) => return p.is_no_exc,
							_ =>
								match (*lnk).parent {
									Some(ref par) => lnk = par.class,
									_ => panic!()
								}
						}
				}
			}
		}
	}
	// REPLACING TEMPLATES
	fn replace_type(&self, src : &RType, args : &Vec<RType>, top : bool) -> RType {
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
		match **src {
			Type::Arr(ref p) => {
				Type::arr(self.replace_type(&p[0], args, false))
			},
			Type::Class(ref pref, ref name, ref params) => {
				let i = if pref.len() == 0 {
					get_i!(name)
				} else {
					None
				};
				match i {
					Some(i) if top => {		
						args[i].clone()
					},
					Some(i) => {
						args[i].clone() // 'cause if use ok-link, then parent method will not construct type
					},
					None => {
						match *params {
							None => src.clone(),
							Some(ref list) => {
								let mut params = vec![];
								for p in list.iter() {
									params.push(self.replace_type(p, args, false));
								}
								return type_c!(pref.clone(), name.clone(), Some(params));
							}
						}
					},
				}
			},
			Type::Fn(_, ref pars, ref res) => {
				let res = self.replace_type(res, args, false);
				let mut args_p = vec![];
				for p in pars.iter() {
					args_p.push(self.replace_type(p, &args, false))
				}
				return type_fn!(args_p, res);
			},
			_ => src.clone()
		}
	}
}
