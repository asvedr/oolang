use syn::*;
use type_check::pack::*;
use type_check::fun_env::*;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::rc::Rc;

#[macro_export]
macro_rules! throw {
	($mess:expr, $curs:expr) => {syn_throw!($mess, $curs)};
}

#[macro_export]
macro_rules! ok {() => {return Ok(())};}

pub type VMap = BTreeMap<String, Result<RType, *mut RType>>;
// Ok  (WE TRULY KNOW WHAT IT IS)
// Err (WE CALCULATED THIS AND WE CAN MISTAKE)

pub struct SubEnv {
	parent      : *mut LocEnv,
	local       : VMap
}

pub enum LocEnv {
	FunEnv(FunEnv),
	SubEnv(SubEnv)
}

// apply to fun env, skip sub env
macro_rules! get_fenv {
	($_self:expr) => {{
		let mut var : *const LocEnv = &*$_self;
		let res : &FunEnv;
		unsafe { loop {
			match *var {
				LocEnv::SubEnv(ref se) => var = se.parent,
				LocEnv::FunEnv(ref env) => {
					res = env;
					break;
				}
			}
		}}
		res
	}};
}

// apply to fun env, skip sub env
macro_rules! get_fenv_m {
	($_self:expr) => {{
		let mut var : *mut LocEnv = &mut *$_self;
		let res : &mut FunEnv;
		unsafe { loop {
			match *var {
				LocEnv::SubEnv(ref se) => var = se.parent,
				LocEnv::FunEnv(ref mut env) => {
					res = env;
					break;
				}
			}
		}}
		res
	}};
}

impl LocEnv {
	pub fn new(pack : *const Pack, tmpl : &Vec<String>, _self : Option<RType>) -> LocEnv {
		//LocEnv::FunEnv(FunEnv::new())
		let mut env = FunEnv::new(pack, _self);
		for t in tmpl.iter() {
			env.templates.insert(t.clone());
		}
		LocEnv::FunEnv(env)
	}
	pub fn inherit(parent : &mut LocEnv) -> LocEnv {
		LocEnv::SubEnv(SubEnv{parent : &mut *parent, local : BTreeMap::new()})
	}
	pub fn self_val(&self) -> Option<RType> {
		get_fenv!(self).self_val.clone()
	}
	pub fn pack(&self) -> &Pack {
		let mut link : *const LocEnv = &*self;
		unsafe {loop {
			match *link {
				LocEnv::FunEnv(ref fe) => return &*fe.global,
				LocEnv::SubEnv(ref le) => link = le.parent
			}
		}}
	}
	// labels only in fun_env
	pub fn add_loop_label(&mut self, name : &String) {
		get_fenv_m!(self).loop_labels.push(&*name);
	}
	// labels only in fun_env
	pub fn pop_loop_label(&mut self) {
		get_fenv_m!(self).loop_labels.pop();
	}
	// labels only in fun_env
	pub fn get_loop_label(&self, name : &String) -> Option<usize> {
		// getting count of loops which must skip to stop target
		// or 'None' if target not exist
		let loop_labels = &get_fenv!(self).loop_labels;
		let len = loop_labels.len();
		for i in 0 .. len {
			let val = unsafe { *loop_labels[len - i - 1] == *name };
			if val {
				return Some(i);
			}
		}
		return None;
	}
	pub fn add_outer(&mut self, out : &LocEnv) {
		match *self {
			LocEnv::FunEnv(ref mut loc_env) => { //le.add_outer(out),
				let mut env : *const LocEnv = &*out;
				unsafe { loop {
					match *env {
						LocEnv::FunEnv(ref fe) => {
							for name in fe.outers.keys() {
								loc_env.outers.insert(name.clone(), fe.outers.get(name).unwrap().clone());
							}
							for name in fe.local.keys() {
								loc_env.outers.insert(name.clone(), fe.local.get(name).unwrap().clone());
							}
							for t in fe.templates.iter() {
								loc_env.templates.insert(t.clone());
							}
						},
						LocEnv::SubEnv(ref se) => {
							for name in se.local.keys() {
								loc_env.outers.insert(name.clone(), se.local.get(name).unwrap().clone());
							}
							env = se.parent;
						}
					}
				}}
			},
			_ => panic!()
		}
	}
	pub fn show(&self) -> String {
		match *self {
			LocEnv::FunEnv(ref fe) => fe.show(),
			LocEnv::SubEnv(ref se) => {
				let mut s = String::new();
				let _ = write!(s, "HAS SUB\n");
				unsafe{ let _ = write!(s, "{}", (*se.parent).show()); }
				let _ = write!(s, "SUB: [");
				for k in se.local.keys() {
					let _ = write!(s, "{},", k);
				}
				let _ = write!(s, "]\n");
				return s;
			}
		}
	}
	pub fn replace_unk(&self, name : &String, tp : RType) {
		let mut lnk : *const LocEnv = &*self;
		unsafe { loop {
			match *lnk {
				LocEnv::FunEnv(ref fe) => return fe.replace_unk(name, tp),
				LocEnv::SubEnv(ref se) => {
					match se.local.get(name) {
						Some(ans) =>
							match *ans {
								Err(ref ptr) => {
									**ptr = tp.clone();
									return;
								},
								_ => panic!("replace_unk: var known: {}", name)
							},
						_ => //unsafe { (*se.parent).replace_unk(name, tp) }
							lnk = se.parent
					}
				}
			}
		}}
	}
	pub fn get_local_var(&self, name : &String) -> &RType {
		let mut lnk : *const LocEnv = &*self;
		unsafe { loop {
			match *lnk {
				LocEnv::FunEnv(ref fe) => return fe.get_local_var(name),
				LocEnv::SubEnv(ref se) => {
					match se.local.get(name) {
						Some(v) => match *v {
							Ok(ref l) => return &l,
							Err(ref l) => return &**l
						},
						None => //unsafe { (*se.parent).get_local_var(name) }
							lnk = se.parent
					}
				}
			}
		}}
	}
	pub fn get_var(&self, pref : &mut Vec<String>, name : &String, tp_dst : &mut RType, pos : &Cursor) -> CheckRes {
		macro_rules! clone_type { ($t:expr) => { match *$t {Ok(ref t) => (*t).clone(), Err(ref t) => (**t).clone()} }; }
		let mut lnk : *const LocEnv = &*self;
		unsafe { loop {
			match *lnk {
				LocEnv::FunEnv(ref fe) => return fe.get_var(pref, name, tp_dst, pos),
				LocEnv::SubEnv(ref se) => {
					//let pref_l : *mut Option<Vec<String>> = &mut *pref;
					if pref.len() == 0 {
							match se.local.get(name) {
								Some(t) => {
									*tp_dst = clone_type!(t);
									pref.push("%loc".to_string());
									return Ok(())
								},
								None => //unsafe { (*se.parent).get_var(pref, name, tp_dst, pos) }
									lnk = se.parent
							}
					} else {
						if pref[0] == "%loc" {
							*tp_dst = clone_type!(se.local.get(name).unwrap());
							return Ok(())
						} else {
							//unsafe { (*se.parent).get_var(&mut *pref_l, name, tp_dst, pos) }
							lnk = se.parent
						}
					}
				}
			}
		}}
	}
	pub fn add_loc_var(&mut self, name : &String, tp : Result<RType, *mut RType>, pos : &Cursor) -> CheckRes {
		//let env = get_fenv_m!(self);
		let local = match *self {
			LocEnv::FunEnv(ref mut env) => &mut env.local,
			LocEnv::SubEnv(ref mut env) => &mut env.local
		};
		match local.insert(name.clone(), tp) {
			Some(_) => throw!(format!("local var {} already exist", name), pos),
			_ => ok!()
		}
	}
	pub fn fun_env(&self) -> &FunEnv {
		get_fenv!(self)
	}
	pub fn fun_env_mut(&mut self) -> &mut FunEnv {
		get_fenv_m!(self)
	}
}

#[macro_export]
macro_rules! add_loc_unk {
	($loc_e:expr, $name:expr, $tp:expr, $pos:expr) => { try!($loc_e.add_loc_var($name, Err($tp), &$pos)) };
}
#[macro_export]
macro_rules! add_loc_knw {
	($loc_e:expr, $name:expr, $tp:expr, $pos:expr) => { try!($loc_e.add_loc_var($name, Ok($tp), &$pos)) };
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
				EVal::Call(_, ref f, ref a, _) => {
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
				EVal::Attr(ref a, _, _) => go_e!(a),
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
				ActVal::Foreach(_,_,ref t,ref e,ref a) => {
					if t.is_unk() {
						return Some(&act.addres);
					} else {
						go_e!(e);
						go_a!(a);
					}
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
				ActVal::Throw(_, _, ref e) =>
					match *e {
						Some(ref e) => go_e!(e),
						_ => ()
					},
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

// pub empty parent constructor
pub fn put_inherit_init(seq : &mut Vec<ActF>, pos : Cursor) {
	let mut new_seq = Vec::new();
	new_seq.reserve(seq.len() + 1);
	let p1 = pos.clone();
	let p2 = p1.clone();
	new_seq.push(Act{
		val    : ActVal::Expr(Expr{
			val     : EVal::Call(
				None,
				Box::new(Expr {
					val     : EVal::Var(vec![], "%parent".to_string()),
					kind    : Type::unk(),
					addres  : p2,
					op_flag : 0
				}),
				vec![],
				false
			),
			kind    : Type::void(),
			addres  : pos,
			op_flag : 0
		}),
		addres : p1
	});
	new_seq.append(seq);
	*seq = new_seq;
}

pub fn replace_inherit_init(seq : &mut Vec<ActF>) -> bool {
	if seq.len() > 0 {
		match seq[0].val {
			ActVal::Expr(ref mut e) =>
				match e.val {
					EVal::Call(_, ref mut fnc, _, _) => {
						match fnc.val {
							EVal::Var(_, ref mut name) => {
								if *name == "init_parent" {
									*name = "%parent".to_string();
									return true
								}
							},
							_ => ()
						}
					},
					_ => ()
				},
			_ => ()
		}
	}
	return false;
}

pub fn gen_default_init(has_parent : bool, addr : Cursor) -> Method {
	macro_rules! parent {() => {
		Act {
			val    : ActVal::Expr(Expr{
				val     : EVal::Call(
					None,
					Box::new(Expr {
						val     : EVal::Var(vec![], "%parent".to_string()),
						kind    : Type::unk(),
						addres  : addr.clone(),
						op_flag : 0
					}),
					vec![],
					false
				),
				kind    : Type::void(),
				addres  : addr.clone(),
				op_flag : 0
			}),
			addres : addr.clone()
		}
	};}
	let body = if has_parent {vec![parent!()]} else {vec![]};
	let t = type_fn!(vec![], Type::void());
	let f = SynFn {
		name        : "init".to_string(),
		tmpl        : vec![],
		args        : vec![],
		rettp       : Type::void(),
		body        : body,
		addr        : addr,
		can_be_clos : false,
		has_named   : false,
		ftype       : t.clone(),
		outers      : vec![],
		no_except   : false
	};
	Method {
		is_virt : false,
		func : f,
		ftype : t
	}
}
