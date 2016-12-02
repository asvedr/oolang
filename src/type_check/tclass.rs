use syn::type_sys::*;
use std::collections::BTreeMap;

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
