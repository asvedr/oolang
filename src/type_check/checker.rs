use syn::*;
use type_check::utils::*;
use type_check::tclass::*;
use type_check::pack::*;
use type_check::noexc_check;
use std::collections::{HashMap/*, HashSet, BTreeMap*/};
use std::mem;
use type_check::regressor::*;
use preludelib::*;

/*
	Типы привязаны к выражениям. В синтаксическом дереве выражений и прочего хранятся все типы.
	местные структуры нужны лишь для быстрого доступа поэтому в них только ссылки на типы
*/

/*
	Расстановка неизвестных типов алго:
		в env суем мутабельный указатель на def var.
		если при проверке удалось восстановить тип - меняем его через указатель
		если после проверки есть неизвестные типы
			если после проверки типов в функции кол-во неизвестных изменилось
				повторяем проверку
			иначе
				возвращаем ошибку
		иначе
			конец (проверка совместимости)
			
*/

pub struct Checker {
	int_real_op : HashMap<String,RType>,
	int_op      : HashMap<String,RType>,
	real_op     : HashMap<String,RType>,
	all_op      : HashMap<String,RType>,
	bool_op     : HashMap<String,RType>,
	packs       : HashMap<String,Pack>,
	std         : Prelude
}

/*
	using checker:
		let checker = Checker::new()
		...
		checker.add_pack(pack) // filling env
		...
		checker.check_mod(module-to-check) // it return Ok(()) or Err(type-check-error)
		// .check_mod setting prefixes to names in expr, calculating types for implicitly and setting it clearly.
		checker.add_pack(make_pack(module-to-check)) // adding mod to env
		// checking next modules
*/

macro_rules! set_var_type {
	($env:expr, $exp:expr, $tp:expr) => {
		if $exp.kind.is_unk() {
			match $exp.val {
				EVal::Var(ref p, ref n) =>
					if p.len() == 0 {
						$env.replace_unk(n, &$tp);
					},
				_ => ()
			}
		}
	};
}

impl Checker {
	pub fn new() -> Checker {
		let mut res = Checker {
			int_real_op : HashMap::new(),
			int_op      : HashMap::new(),
			real_op     : HashMap::new(),
			all_op      : HashMap::new(),
			bool_op     : HashMap::new(),
			packs       : HashMap::new(),
			std         : Prelude::new()
		};
		let u = Type::unk();
		macro_rules! adds {
				($s:expr, $a:expr) => {$s.insert($a.to_string(), u.clone())};
				($s:expr, $a:expr, $v:expr) => {$s.insert($a.to_string(), $v)};
		}
		let u = Type::unk();
		let b = Type::bool();
		adds!(res.int_real_op, "+", u.clone());
		adds!(res.int_real_op, "-", u.clone());
		adds!(res.int_real_op, "*", u.clone());
		adds!(res.int_real_op, "/", u.clone());
		adds!(res.int_real_op, ">", b.clone());
		adds!(res.int_real_op, "<", b.clone());
		adds!(res.int_real_op, ">=",b.clone());
		adds!(res.int_real_op, "<=",b.clone());
		adds!(res.int_op,      "%");
		adds!(res.real_op,     "**");
		adds!(res.all_op,      "==");
		adds!(res.all_op,      "!=");
		adds!(res.bool_op,     "&&");
		adds!(res.bool_op,     "||");
		res
	}
	// this is only one public fun for checking
	pub fn check_mod(&self, smod : &mut SynMod, mod_name : &Vec<String>) -> CheckRes {
		let mut pack = Pack::new();
		pack.out_cls.reserve(self.std.pack.cls.len());
		for c in self.std.pack.cls.keys() {
			pack.out_cls.insert(c.clone(), &*self.std.pack);
		}
		pack.out_fns.reserve(self.std.pack.fns.len());
		for f in self.std.pack.fns.keys() {
			pack.out_fns.insert(f.clone(), &*self.std.pack);
		}
		pack.out_exc.reserve(self.std.pack.exceptions.len());
		for e in self.std.pack.exceptions.keys() {
			pack.out_exc.insert(e.clone(), &*self.std.pack);
		}
		pack.packs.reserve(smod.imports.len());
		pack.packs.insert("%std".to_string(), &*self.std.pack);
		pack.cls.reserve(smod.classes.len());
		pack.fns.reserve(smod.funs.len() + smod.c_fns.len());
		pack.exceptions.reserve(smod.exceptions.len());
		// ADD FUNS TO ENV
		for f in smod.funs.iter_mut() {
			// GETTING NAME
			let n = f.name.clone();
			// CHECK TMPL
			let tl = f.tmpl.len();
			if tl > 0 {
				for i in 0 .. tl {
					for j in i+1 .. tl {
						if f.tmpl[i] == f.tmpl[j] {
							throw!(format!("template {} used more then once", f.tmpl[i]), f.addr)
						}
					}
				}
			}
			// FIX TYPE
			unsafe{ self.check_type_pack(&pack, &f.tmpl, &mut f.ftype, &f.addr)? };
			// ADD TO ENV
			match pack.fns.insert(n, f.ftype.clone()) {
				Some(_) => throw!("fun with this name already exist in this module", f.addr),
				_ => ()
			}
			// OPT FLAG
			if f.no_except {
				pack.fns_noex.insert(f.name.clone());
			}
		}
		// ADD CLASSES TO ENV
		for c in smod.classes.iter_mut() {
			// CHECK TEMPLATE
			//println!("CALL FOR {}", c.name);
			let tlen = c.template.len();
			for i in 0 .. tlen {
				for j in i+1 .. tlen {
					if c.template[i] == c.template[j] {
						throw!(format!("using '{}' in template more then once", c.template[i]), c.addres)
					}
				}
			}
			// CHECK PARENT
			let par = match c.parent {
				Some(ref mut tp) => unsafe {
					//let tp : &mut (&mut Type) = mem::transmute(tp);
					let tp : &mut Type = mem::transmute(&**tp);
					match *tp {
						Type::Class(ref mut pref, ref name, ref mut pars) => {
							let p_ref : Option<*const Vec<RType>> =
								match *pars {
									Some(ref mut vec) => {
										for par in vec.iter_mut() {
											self.check_type_pack(&pack, &c.template, par, &c.addres)?
										}
										Some(&*vec)
									},
									_ => None
								};
							pack.check_class(pref, name, pars, &c.addres)?;
							let cls =
								if pref[0] == "%mod" {
									let p = Vec::new();
									pack.get_cls_rc(&p, name)
								} else {
									pack.get_cls_rc(pref, name)
								};
							let cls = cls.unwrap().clone();
							Some(Parent::new(cls, p_ref))
						},
						_ => throw!(format!("can't inherit from {:?}", tp), c.addres)
					}
				},
				_ => None
			};
			// GETTING 'TClass'
			let tcls = TClass::from_syn(c, par, &mod_name)?;
			// ADDING TO ENV
			//println!("ADD TO ENV {}", c.name);
			match pack.cls.insert(c.name.clone(), tcls) {
				Some(_) => throw!(format!("class with name {} already exist", c.name), c.addres),
				_ => ()
			}
			// FIX INITIALIZER
			let mut init_found = false;
			for meth in c.pub_fn.iter_mut() {
				let f = meth.func.name == "init";
				if f {
					unsafe{ self.check_type_pack(&pack, &c.template, &mut meth.func.ftype, &meth.func.addr)? };
					meth.ftype = meth.func.ftype.clone();
					//println!("INITIALIZER FOUND, tp:{:?}", meth.ftype);
					init_found = true;
					break;
				}
			}
			if !init_found {
				let addr = c.addres.clone();
				let has_parent = match c.parent {Some(_) => true, _ => false};
				c.pub_fn.push(gen_default_init(has_parent, addr))
			}
			match pack.cls.get_mut(&c.name) {
				Some(tcl) => {
					unsafe { tcl.borrow_mut().check_initializer()? }
					//println!("INIT PARAMS FOR {}: {:?}", tcl.args)
				},
				_ => ()
			}
		}
		// CHEKC exceptions ADD exceptions TO ENV
		let tmpl_plug = Vec::new();
		for e in smod.exceptions.iter_mut() {
			if pack.exceptions.contains_key(&e.name) {
				throw!(format!("exception {} already exist", e.name), e.addr);
			} else {
				match e.arg {
					Some(ref mut tp) => {
						unsafe{ self.check_type_pack(&pack, &tmpl_plug, tp, &e.addr)? };
						pack.exceptions.insert(e.name.clone(), Some(tp.clone()));
					},
					_ => {
						pack.exceptions.insert(e.name.clone(), None);
					}
				}
			}
		}
		// CHECK CLASSES
		for c in smod.classes.iter_mut() {
			self.check_class(&pack, c)?;
		}
		// CHECK FUNS
		for f in smod.funs.iter_mut() {
			self.check_fn(&pack, f, None, None)?;
		}
        // AUTOSET #NOEXCEPT FLAG
        noexc_check::recalculate(smod, &mut pack);
		ok!()
	}
	fn check_class(&self, pack : &Pack, class : &mut Class) -> CheckRes {
		let self_t : RType = {
			let tmpl : Option<Vec<RType>>;
			if class.template.len() > 0 {
				let mut t : Vec<RType> = vec![];
				for tp in class.template.iter() {
					t.push(type_c!(vec!["%tmpl".to_string()], tp.clone(), None))
				}
				tmpl = Some(t)
			} else {
				tmpl = None
			}
			type_c!(vec!["%mod".to_string()], class.name.clone(), tmpl)
		};
		let tmpl : &Vec<String> = &class.template;
		// priv_prop, pub_prop, priv_fn, pub_fn
		// CHECK PROPS TYPE
		for prop in class.priv_prop.iter_mut() {
			unsafe{ self.check_type_pack(&pack, tmpl, &mut prop.ptype, &prop.addres)? }
		}
		for prop in class.pub_prop.iter_mut() {
			unsafe{ self.check_type_pack(&pack, tmpl, &mut prop.ptype, &prop.addres)? }
		}
		let pref = vec![];
		let self_tclass = match pack.get_cls_rc(&pref, &class.name) {
			Some(a) => a,
			_ => panic!()
		};
		// CHECK METHODS
		macro_rules! precheck_meth {($m:expr) => {{
			// FIX TYPE
			unsafe{ self.check_type_pack(&pack, &tmpl, &mut $m.func.ftype, &$m.func.addr)? };
			// FIX TMPL
			$m.func.tmpl = tmpl.clone();
			$m.ftype = $m.func.ftype.clone();
			// FIX INITIALIZER INHERITING
			if $m.func.name == "init" {
				match self_tclass.borrow().parent {
					Some(ref par) => {
						let par_init = par.class.borrow().args.clone();
						let f = &mut $m.func;
						let replaced = replace_inherit_init(&mut f.body);
						if !replaced {
							let pos = f.addr.clone();
							if par_init.len() == 0 {
								put_inherit_init(&mut f.body, pos);
							} else {
								throw!("initializer must include parent init call", pos)
							}
						}
					},
					_ => ()
				}
			}
		}}; }
		for m in class.priv_fn.iter_mut() {
			if m.func.name == "init" {
				syn_throw!("init must be public", m.func.addr);
			} else {
				precheck_meth!(m);
			}
		}
		for m in class.pub_fn.iter_mut() {
			precheck_meth!(m);
		}
		for m in class.priv_fn.iter_mut() {
			self.check_fn(&pack, &mut m.func, None, Some(self_t.clone()))?;
		}
		for m in class.pub_fn.iter_mut() {
			self.check_fn(&pack, &mut m.func, None, Some(self_t.clone()))?;
		}
		// FINALIZING
		// CHECK ATTRIBS. SINGLETON, INITIALIZER
		
		ok!()
	}
	fn check_fn(&self, pack : &Pack, fun : &mut SynFn, out_env : Option<&LocEnv>, _self : Option<RType>) -> CheckAns<isize> {
		let mut env =
            match out_env {
                Some(_) => {
                    // THIS IS CLOSURE. NEED REC PARAMS
                    let name = fun.name.clone();
                    let tp = fun.ftype.clone();
                    LocEnv::new_loc(&*pack, &fun.tmpl, _self, name, tp)
                },
                _ =>
                    // THIS IS GLOBAL FUN
                    LocEnv::new_glob(&*pack, &fun.tmpl, _self)
            };
		// PREPARE LOCAL ENV
		let top_level = match out_env {
			Some(eo) => {
				env.add_outer(eo);
				false
			},
			_ => true
		};
		// ARGS
		for arg in fun.args.iter_mut() {
			unsafe{ self.check_type(&env, &mut arg.tp, &fun.addr)? };
			//let p : *mut Type = &mut arg.tp;
			add_loc_knw!(env, &arg.name, arg.tp.clone(), fun.addr);
		}
		// RET TYPE
		unsafe{ self.check_type(&env, &mut fun.rettp, &fun.addr)? };
		env.fun_env_mut().set_ret_type(fun.rettp.clone());
		// CHECK BODY AND COERSING
		if top_level {
			let mut unknown = -1;
			loop {
				let unk_count = try!(self.check_actions(&mut env, &mut fun.body, unknown > 0));
				if unk_count > 0 {
					if unknown < 0 || unknown > unk_count {
						// REPEATING CHECK TYPE FOR REGRESS CALCULATION
						unknown = unk_count;
					} else {
						// CAN'T GET TYPE SOLUTION
						let pos = find_unknown(&fun.body);
						throw!("can't calculate type of expression", pos);
					}
				} else {
					// TYPING OK
					// NOT NEED fun.outers TOP LEVEL HAS NULL
                    fun.rec_used = env.is_rec_used();
					return Ok(0)
				}
			}
		} else {
			let cnt = try!(self.check_actions(&mut env, &mut fun.body, false));
			//fun.outers =
			let fenv = env.fun_env();
            fun.rec_used = fenv.rec_used;
			unsafe {
				for (n,t) in fenv.outers.iter() {
					let t = match *t {
						Ok(ref t) => t.clone(),
						Err(ref r) => (**r).clone()
					};
					fun.outers.insert(n.clone(), t);
				}
			}
			return Ok(cnt)
		}
	}
	fn check_actions(&self, env : &mut LocEnv, src : &mut Vec<ActF>, repeated : bool) -> CheckAns<isize> {
		// 'repeated' is a flag for non first check
		// if it true then var won't added to env on DefVar
		let mut unk_count = 0;
		macro_rules! expr {($e:expr) => { unk_count += self.check_expr(env, $e)?}; }
		macro_rules! actions {($e:expr, $a:expr) => { unk_count += try!(self.check_actions($e, $a, false)) }; }
		let env_pt : *mut LocEnv = &mut *env;
		let env_ln : &mut LocEnv = unsafe{ mem::transmute(env_pt) };
		macro_rules! regress {
			($e:expr, $t:expr) => {regress_expr(env_ln, $e, $t)?};
			($env:expr, $e:expr, $t:expr) => {regress_expr($env, $e, $t)?};
		}
		for act in src.iter_mut() {
			match act.val {
				ActVal::Expr(ref mut e) => /*act.exist_unk =*/ expr!(e),
				ActVal::Ret(ref mut opt_e) => {
					match *opt_e {
						Some(ref mut e) => {
							expr!(e);
							if e.kind.is_unk() {
								// REGRESS CALL
								regress!(e, env.fun_env().ret_type().clone());
							} else if !env.fun_env().check_ret_type(&e.kind) {
								throw!(format!("expect type {:?}, found {:?}", env.fun_env().ret_type(), e.kind), act.addres)
							}
						},
						None =>
							if !env.fun_env().check_ret_type(&Type::void()) {
								throw!(format!("expect type {:?}, found void", env.fun_env().ret_type()), act.addres)
							}
					}
				},
				ActVal::If(ref mut cond, ref mut th_act, ref mut el_act) => {
					expr!(cond);
					{
						let mut sub = LocEnv::inherit(env);
						actions!(&mut sub, th_act);
					}
					{
						let mut sub = LocEnv::inherit(env);
						actions!(&mut sub, el_act);
					}
				},
				ActVal::DVar(ref name, ref mut tp, ref mut val) => {
					let unk_val = match *val {
						Some(ref mut val) => {
							expr!(val);
							val.kind.is_unk()
						},
						None => false
					};
					match **tp {
						Type::Unk => {
							*tp = match *val {
								None => Type::unk(),
								Some(ref mut v) => {
									v.kind.clone()
								}
							};
							if !repeated {
								add_loc_unk!(env, name, tp, act.addres);
							}
						},
						_ => {
							// regression recovery
							unsafe{ self.check_type(env, tp, &act.addres)? };
							if !repeated {
								add_loc_knw!(env, name, tp.clone(), act.addres);
							}
						}
					}
					if unk_val {
						match *val {
							Some(ref mut v) => {
								let tp = env.get_local_var(name);
								if !tp.is_unk() {
									// BAD SOLUTION
									// special case for empty array-assoc situation
									regress!(v, tp.clone())
									/*match val.val {
										EVal::Asc(item_t) if tp.is_arr() => {
											env.replace_unk(name, )
											regress!()
										},
										_ => regress!(v, tp)
									}*/
								}
							},
							_ => panic!()
						}
					}
				},
				ActVal::Asg(ref mut var, ref mut val) => {
					//act.exist_unk = expr!(var) || expr!(val);
					expr!(var);
					expr!(val);
					let ua = var.kind.is_unk();
					let ub = val.kind.is_unk();
					if ua && ub {
						// PASS
					} else if ua {
						// REGRESS CALL
						regress!(var, val.kind.clone());
					} else if ub {
						// REGRESS CALL
						regress!(val, var.kind.clone());
					} else if var.kind != val.kind {
						throw!(format!("assign parts incompatible: {:?} and {:?}", var.kind, val.kind), act.addres)
					}
					// ADD CHECK FOR WHAT CAN BE AT LEFT PART OF ASSIG
					match var.val {
						EVal::Var(_,_) => (),
						EVal::Item(_,_) => (),
						EVal::Attr(_,_,ref is_m) =>
							if *is_m {
								throw!("can't use assigment for method", var.addres)
							},
						_ => {
							throw!("can't use assigment for this expr", var.addres)
						}
					}
				},
				ActVal::Throw(ref mut pref, ref mut key, ref mut e) => {
					// CAN RETURN ANY CLASS BUT CAN'T RETURN A PRIMITIVE
					match *e {
						Some(ref mut e) => expr!(e),
						_ => ()
					}
					let param = env.fun_env().check_exception(pref, key, &act.addres)?;
					match *e {
						Some(ref mut e) => {
							//expr!(e);
							match param {
								Some(t) =>
									if e.kind.is_unk() {
										regress!(e, t);
									} else if e.kind == t {
										()
									} else {
										throw!(format!("exception excpect param {:?}, but found {:?}", t, e.kind), e.addres)
									},
								_ =>
									throw!(format!("exception excpect no params, but found {:?}", e.kind), e.addres)
							}
						},
						_ => match param {
							Some(t) => throw!(format!("exception expect param {:?}, but has none", t), act.addres),
							_ => ()
						}
					}
				},
				ActVal::DFun(ref mut df) => {
					{
						let pack : &Pack = env.pack();
						let _self = env.self_val();
						unk_count += self.check_fn(pack, &mut **df, Some(env), _self)?;
					}
					if !repeated {
						add_loc_knw!(env, &df.name, df.ftype.clone(), df.addr);
					}
					for name in df.outers.keys() {
						let mut pref = Vec::new();
						let mut tp = Type::unk();
						let _ = env.get_var(&mut pref, name, &mut tp, &df.addr);
						if pref[0] == "%out" {
							env.fun_env_mut().used_outers.insert(name.clone());
						} else if pref[0] == "%rec" {
                            env.set_rec_used(true);
                        }
					}
				},
				ActVal::Try(ref mut body, ref mut catches) => {
				// благодаря тому, что в LocEnv ссылки, а не типы, расформирование и формирование LocEnv заново не влияют на вычисление типов
				// окружение текущей функции остается, но локальное для блоков здесь формируется заново при каждом проходе 
					{
						let mut sub = LocEnv::inherit(env);
						actions!(&mut sub, body);
						//unk_count += try!(self.check_actions(&mut sub, body, false/*repeated*/));
					}
					for catch in catches.iter_mut() {
						// CHECK PARAMS
						if catch.ekey.len() > 0 {
							let earg = env.fun_env().check_exception(&mut catch.epref, &catch.ekey, &catch.addres)?;
							match catch.vname {
								Some(_) =>
									match earg {
										Some(t) => catch.vtype = t,
										_ => throw!(format!("exception {} had no params", catch.ekey), catch.addres)
									},
								_ => ()
							}
						}
						let mut sub = LocEnv::inherit(env);
						// UPDATE ENV IF NEEDED
						match catch.vname {
							Some(ref name) => add_loc_knw!(sub, name, catch.vtype.clone(), catch.addres),
							_ => ()
						}
						// DO WITH SUB
						actions!(&mut sub, &mut catch.act);
					}
				},
				ActVal::While(ref lname, ref mut cond, ref mut body) => {
					// adding label if exist
					match *lname {
						Some(ref name) => env.add_loop_label(name),
						_ => ()
					}
					// checking cond
					expr!(cond);
					// checking body
					{
						let mut sub = LocEnv::inherit(env);
						actions!(&mut sub, body);
					}
					// pop label if it was
					match *lname {
						Some(_) => env.pop_loop_label(),
						_ => ()
					}
				},
				ActVal::For(ref lname, ref vname, ref mut val_from, ref mut val_to, ref mut body) => {
					match *lname {
						Some(ref name) => env.add_loop_label(name),
						_ => ()
					}
					expr!(val_from);
					expr!(val_to);
					{
						let mut sub = LocEnv::inherit(env);
						// type for env
						add_loc_knw!(sub, vname, Type::int(), act.addres);
						actions!(&mut sub, body);
					}
					match *lname {
						Some(_) => env.pop_loop_label(),
						_ => ()
					}
				},
				ActVal::Foreach(ref lname, ref vname, ref mut vt, ref mut cont, ref mut body) => {
					match *lname {
						Some(ref name) => env.add_loop_label(name),
						_ => ()
					}
					{
						let mut sub = LocEnv::inherit(env);
						expr!(cont);
						let mut need_regress = false;
						match *cont.kind {
							Type::Arr(ref item) => {
								/*if vt.is_unk() {
									*vt = item[0].clone();
									add_loc_unk!(sub, vname, &mut item[0], act.addres);
								} else*/ if *vt != item[0] {
									throw!(format!("foreach var expected {:?}, found {:?}", item, vt), act.addres);
								} else {
									add_loc_knw!(sub, vname, item[0].clone(), act.addres);
								}
							},
							Type::Unk => {
								if vt.is_unk() {
									add_loc_unk!(sub, vname, &mut *vt, act.addres);
								} else {
									need_regress = true;
								}
							},
							_ => throw!("you can foreach only through array", cont.addres)
						}
						if need_regress {
							regress!(&mut sub, cont, vt.clone());
							add_loc_knw!(sub, vname, vt.clone(), act.addres);
						}
						actions!(&mut sub, body);
					}
					match *lname {
						Some(_) => env.pop_loop_label(),
						_ => ()
					}
				},
				ActVal::Break(ref lname, ref mut cnt) =>
					match *lname {
						Some(ref name) => 
							match env.get_loop_label(name) {
								Some(n) => *cnt = n,
								_ => *cnt = 0
							},
						_ => *cnt = 0
					}
				//_ => ()
			}
		}
		Ok(unk_count)
	}
	fn check_expr(&self, env : &mut LocEnv, expr : &mut Expr) -> CheckAns<isize> {
		//println!("CHECK EXPR");
		//expr.print();
		let mut unk_count = 0;
		// recursive check expression
		macro_rules! check {($e:expr) => {unk_count += try!(self.check_expr(env, $e))};};
		macro_rules! check_type {($t:expr) => {unsafe{self.check_type(env, $t, &expr.addres)?}}}
		macro_rules! regress {($e:expr, $t:expr) => {regress_expr(env, $e, $t)?}; }
		// macro for check what category of operator is
		macro_rules! is_in {
			($e:expr, $out:expr, $seq:ident, $els:expr) => {
				match self.$seq.get($e) {
					Some(t) => {
						$out = t.clone();
						Some(&self.$seq)
					},
					_ => $els
				}
			};
		}
		// check fun is operator
		macro_rules! check_fun {($e:expr, $o:expr) =>
			{match $e.val {
				EVal::Var(ref pref, ref name) => {
					if pref.len() > 0 {
						if pref[0] == "%opr" {
							is_in!(name, $o, int_real_op, is_in!(name, $o, int_op, is_in!(name, $o, real_op, is_in!(name, $o, all_op, is_in!(name, $o, bool_op, None)))))
						} else {
							None
						}
					} else {
						None
					}
				},
				_ => None
			}};
		}
		// don't calculate if expression checked
		// match expr.kind {
		// 	Type::Unk => (), _ => return Ok(false)
		// }
		match expr.val {
			EVal::Call(ref mut tmpl, ref mut f, ref mut args, ref mut noexc) => {
				let mut res_type = Type::unk();
				let chf : Option <*const HashMap<String,RType>> = check_fun!(**f, res_type);
				for a in args.iter_mut() {
					check!(a);
				}
				match *tmpl {
					Some(ref mut tmpl) => {
						for t in tmpl.iter_mut() {
							check_type!(t);
						}
					},
					_ => ()
				}
				match chf {
					Some(seq_l) => {
						// CHECK OPERATION
						let a : RType = args[0].kind.clone();
						let b : RType = args[1].kind.clone();
						// INT OR REAL OPERATIONS + - * >= <= < > / 
						if seq_l == &self.int_real_op {
							macro_rules! ok {($tp:expr) => {{
								//set_var_type!(env, args[0], $tp);
								//set_var_type!(env, args[1], $tp);
								// REGRESS CALL
								*noexc = true;
								regress!(&mut args[0], $tp.clone());
								regress!(&mut args[1], $tp.clone());
								match *res_type {
									Type::Unk => {
										f.kind = type_fn!(vec![$tp, $tp], $tp);
										expr.kind = $tp
									},
									_ => {
										f.kind = type_fn!(vec![$tp, $tp], res_type.clone());
										expr.kind = res_type
									}
								}
							}};}
							if (*a).is_int() {
								if (*b).is_int() || (*b).is_unk() {
									ok!(Type::int())
								} else if (*b).is_real() {
									let addr = args[0].addres.clone();
									let arg = mem::replace(&mut args[0].val, EVal::Null);
									let arg = Expr{val : arg, kind : Type::int(), addres : addr, op_flag : 0};
									args[0].val = EVal::ChangeType(Box::new(arg), Type::real());
									args[0].kind = Type::real();
									//args[0] = Expr{val : EVal::ChangeType(Box::new(args[0]), Type::Real), kind : Type::Real, addres : addr};
									ok!(Type::real())
								} else {
									throw!(format!("expect int found {:?}", *b), args[1].addres.clone())
								}
							} else if (*a).is_real() {
								if (*b).is_real() || (*b).is_unk() {
									ok!(Type::real())
								} else if (*b).is_int() {
									let addr = args[1].addres.clone();
									let arg = mem::replace(&mut args[1].val, EVal::Null);
									let arg = Expr{val : arg, kind : Type::int(), addres : addr, op_flag : 0};
									args[1].val = EVal::ChangeType(Box::new(arg), Type::real());
									args[1].kind = Type::real();
									//args[1] = Expr{val : EVal::ChangeType(Box::new(args[1]), Type::Real), kind : Type::Real, addres : addr};
									ok!(Type::real())
								} else {
									throw!(format!("expect real found {:?}", *b), args[1].addres.clone())
								}
							} else if (*a).is_unk() {
								if (*b).is_int() {
									ok!(Type::int())
								} else if (*b).is_real() {
									ok!(Type::real())
								} else if (*b).is_unk() {
									unk_count += 1;
								} else {
									throw!(format!("operands must be int or real, found {:?}", *b), args[1].addres.clone())
								}
							} else {
								throw!(format!("operands must be int or real, found {:?}", *a), args[0].addres.clone())
							}
						// INT OPERATIONS (int, int) -> int
						} else if seq_l == &self.int_op {
							if !((*a).is_int() || (*a).is_unk()) {
								throw!(format!("expect int, found {:?}", *a), args[0].addres.clone())
							} else if !((*b).is_int() || (*b).is_unk()) {
								throw!(format!("expect int, found {:?}", *b), args[1].addres.clone())
							} else {
								//set_var_type!(env, args[0], Type::Int);
								//set_var_type!(env, args[1], Type::Int);
								// REGRESS CALL
								let i = Type::int();
								regress!(&mut args[0], i.clone());
								regress!(&mut args[1], i.clone());
								f.kind = type_fn!(vec![i.clone(), i.clone()], i.clone());
								expr.kind = i.clone()
							}
						// REAL OPERATIONS (real, real) -> real
						} else if seq_l == &self.real_op {
							if !((*a).is_real() || (*a).is_unk()) {
								throw!(format!("expect real found {:?}", *a), args[0].addres.clone())
							} else if !((*b).is_real() || (*b).is_unk()) {
								throw!(format!("expect real found {:?}", *b), args[1].addres.clone())
							} else {
								//set_var_type!(env, args[0], Type::Real);
								//set_var_type!(env, args[1], Type::Real);
								// REGRESS CALL
								let r = Type::real();
								regress!(&mut args[0], r.clone());
								regress!(&mut args[1], r.clone());
								f.kind = type_fn!(vec![r.clone(), r.clone()], r.clone());
								expr.kind = r.clone()
							}
						// ALL OPERATIONS
						} else if seq_l == &self.all_op {
							if (*a).is_unk() && (*b).is_unk() {
								// PASS
							} if *a == *b || a.is_unk() || b.is_unk() {
								let tp : RType;
								if a.is_unk() {
									tp = b.clone();
									regress!(&mut args[0], tp.clone());
								} else if b.is_unk() {
									tp = a.clone();
									regress!(&mut args[1], tp.clone());
								} else {
									tp = a.clone();
								}
								f.kind = type_fn!(vec![tp.clone(), tp], Type::bool());
								expr.kind = Type::bool();
                                *noexc = true;
							} else {
								throw!(format!("expect {:?}, found {:?}", *a, *b), args[1].addres.clone())
							}
						// BOOL OPERATIONS (bool, bool) -> bool
						} else /* if seq_l == &self.bool_op */ {
							if !((*a).is_bool() || (*a).is_unk()) {
								throw!(format!("expect bool, found {:?}", *a), args[0].addres.clone());
							} else if !((*b).is_bool() || (*b).is_unk()) {
								throw!(format!("expect bool, found {:?}", *b), args[1].addres.clone());
							} else {
								let b = Type::bool();
								regress!(&mut args[0], b.clone());
								regress!(&mut args[1], b.clone());
								f.kind = type_fn!(vec![b.clone(), b.clone()], b.clone());
								expr.kind = b.clone()
							}
						}
					},
					// NOT OPERATOR, REGULAR FUNC CALL
					None => {
						// ref mut tmpl, ref mut f, ref mut args
						match *tmpl {
							Some(ref mut t) =>
								for tp in t.iter_mut() {
									check_type!(tp);
								},
							_ => (),
						}
						check!(f);
						/*for a in args.iter_mut() {
							check!(a);
						}*/
						//let mut type_known = false;
						// TYPING
						match *f.kind {
							Type::Fn(ref tmpl_t, ref args_t, ref res_t) => {
								// CHECK TMPL
								if args.len() != args_t.len() {
									throw!(format!("expect {} args, found {}", args_t.len(), args.len()), expr.addres.clone());
								} else {
									for i in 0 .. args.len() {
										let a = &mut args[i];
										let t = &args_t[i];
										if a.kind == *t {
											// ALL OK
										} else if a.kind.is_unk() {
											// REGRESS CALL
											regress!(a, t.clone());
										} else {
											throw!(format!("expect {:?}, found {:?}", t, a.kind), a.addres.clone())
										}
									}
									expr.kind = res_t.clone();
								}
								//type_known = true;
							},
							Type::Unk => unk_count += 1,
							ref t => {
								throw!(format!("expect Fn found {:?}", t), f.addres.clone())
							}
						}
						//if type_known {
							// OPT FLAG
							// CHECK NOEXCEPT
							match f.val {
								EVal::Var(ref pref, ref name) => {
									if pref[0] != "%loc" {
										*noexc = env.pack().is_fn_noexcept(pref, name)
									}
								},
								EVal::Attr(ref obj, ref prop_name, ref is_meth) => {
									if *is_meth {
										match *obj.kind {
											Type::Class(ref pref, ref name, _) => {
												match env.pack().get_cls(pref, name) {
													Some(cls_ptr) => unsafe {
														*noexc = (*cls_ptr).is_method_noexc(prop_name)
													},
													_ => panic!()
												}
											},
											Type::Arr(_) => {
												let pref = vec!["%std".to_string()];
												let name = "%arr".to_string();
												match env.pack().get_cls(&pref, &name) {
													Some(cls_ptr) => unsafe {
														*noexc = (*cls_ptr).is_method_noexc(prop_name)
													},
													_ => panic!()
												}
											},
											Type::Str => {
												let pref = vec!["%std".to_string()];
												let name = "%str".to_string();
												match env.pack().get_cls(&pref, &name) {
													Some(cls_ptr) => unsafe {
														*noexc = (*cls_ptr).is_method_noexc(prop_name)
													},
													_ => panic!()
												}
											},
											_ => ()
										}
									}
								}
								_ => ()
							}
						//}
					}
				}
			},
			EVal::NewClass(ref mut tmpl, ref mut pref, ref mut name, ref mut args) => {
				let pcnt = match *tmpl {
					Some(ref mut tmpl) => {
						for t in tmpl.iter_mut() {
							check_type!(t);
						};
						tmpl.len()
					},
					_ => 0
				};
				try!(env.fun_env().check_class(pref, name, tmpl, &expr.addres));
				for a in args.iter_mut() {
					check!(a);
				}
				unsafe {
					let cls =
						if pref[0] == "%mod" {
							let p = Vec::new();
							env.pack().get_cls(&p, name).unwrap()
						}
						else { 
							env.pack().get_cls(pref, name).unwrap()
						};
					if (*cls).params.len() != pcnt {
						throw!(format!("class {} expect {} params, given {}", name, (*cls).params.len(), pcnt), &expr.addres)
					}
					if (*cls).args.len() != args.len() {
						throw!(format!("class {} initializer expect {} args, given {}", name, (*cls).args.len(), args.len()), &expr.addres)
					}
					for i in 0 .. args.len() {
						if args[i].kind.is_unk() {
							// REGRESS CALL
							regress!(&mut args[i], (*cls).args[i].clone());
						} else if (*cls).args[i] != args[i].kind {
							throw!(format!("expected {:?}, found {:?}", (*cls).args[i], args[i].kind), &args[i].addres)
						}
					}
				}
				expr.kind = type_c!(pref.clone(), name.clone(), tmpl.clone())
			},
			EVal::Item(ref mut a, ref mut i) => {
				// FOR 'a' ALLOW TYPES: Vec<_>, Asc<_,_>, Str
				check!(a);
				check!(i);
				if a.kind.is_unk() {
					unk_count += 1;
				} else if a.kind.is_arr() || a.kind.is_str() {
					if i.kind.is_int() {
						// ALL OK
					} else if i.kind.is_unk() {
						// REGRESS CALL
						regress!(i, Type::int());
					} else {
						throw!(format!("expect int, found {:?}", i.kind), i.addres.clone())
					}
					expr.kind = a.kind.arr_item().clone();
				} else if a.kind.is_asc() {
					let (key, val) = a.kind.asc_key_val();
					if key == i.kind {
						// ALL OK
					} else if i.kind.is_unk() {
						// REGRESS CALL
						regress!(i, key);
					} else {
						throw!(format!("expect {:?}, found {:?}", key, i.kind), i.addres.clone())
					}
					expr.kind = val;
				} else {
					throw!(format!("expect arr or asc, found {:?}", a.kind), a.addres.clone())
				}
			},
			EVal::Var(ref mut pref, ref name) => { // namespace, name
				//println!("GET VAR FOR {:?} {}", pref, name);
				//println!("{}", env.show());
				if name == "%init" {
					if expr.kind.is_unk() {
						expr.kind = env.fun_env().parent_init();
					}
				} else {
					try!(env.get_var(pref, name, &mut expr.kind, &expr.addres));
					//println!("GET VAR OK: {:?}", pref);
					if pref[0] == "%out" {
						env.fun_env_mut().used_outers.insert(name.clone());
					} else if pref[0] == "%rec" {
                        env.set_rec_used(true);
                    }
					/* MUST RECUSRIVE CHECK FOR COMPONENTS */
					match *expr.kind {
						Type::Unk => return Ok(1),
						_ => return Ok(0)
					}
				}
			},
			EVal::Arr(ref mut items) => {
				let mut item : Option<RType> = match *expr.kind {
					Type::Unk => None,
					Type::Arr(ref v) => Some(v[0].clone()),
					_ => panic!()
				};
				for i in items.iter_mut() {
					check!(i);
					match item {
						Some(ref t) =>
							if *t != i.kind && !i.kind.is_unk() {
								throw!(format!("expected {:?}, found {:?}", t, i.kind), i.addres.clone())
							},
						_ if !i.kind.is_unk() => item = Some(i.kind.clone()),
						_ => ()
					}
				}
				match item {
					Some(t) => {
						// REGRESS CALL
						for i in items.iter_mut() {
							regress!(i, t.clone());
						}
						expr.kind = Type::arr(t);
					},
					_ => unk_count += 1
				}
			},
			EVal::Asc(ref mut items) => {
				let mut key_type : Option<RType> = None;
				let mut val_type : Option<RType> = None;
				if expr.kind.is_asc() {
					let (a,b) = expr.kind.asc_key_val();
					key_type = Some(a);
					val_type = Some(b);
				}
				macro_rules! skey {($tp:expr, $addr:expr) => {
					match key_type {
						None        => key_type = Some($tp),
						Some(ref t) =>
							if *t != $tp {
								throw!(format!("expected {:?}, found {:?}", t, $tp), $addr)
							}
					}
				};}
				for pair in items.iter_mut() {
					check!(&mut pair.b);
					check!(&mut pair.a);
					match *pair.a.kind {
						Type::Str  => skey!(Type::str(),  pair.a.addres),
						Type::Char => skey!(Type::char(), pair.a.addres),
						Type::Int  => skey!(Type::int(),  pair.a.addres),
						Type::Unk  => unk_count += 1,
						ref a => throw!(format!("asc key must be int, char or str, found {:?}", a), pair.a.addres.clone())
					}
					match val_type {
						None => if !pair.b.kind.is_unk() {
							val_type = Some(pair.b.kind.clone());
						},
						Some(ref t) => {
							if !pair.b.kind.is_unk() && pair.b.kind != *t {
								throw!(format!("expected {:?}, found {:?}", t, pair.b.kind), pair.b.addres.clone())
							}
						}
					}
				}
				match key_type {
					Some(k) => match val_type {
						Some(v) => {
							// REGRESS CALL
							for pair in items.iter_mut() {
								regress!(&mut pair.a, k.clone());
								regress!(&mut pair.b, v.clone());
							}
							expr.kind = type_c!(vec!["%std".to_string()], "Asc".to_string(), Some(vec![k, v]));
						},
						_ => unk_count += 1
					},
					_ => unk_count += 1
				}
			},
			EVal::Attr(ref mut obj, ref pname, ref mut m_flag) => {
				check!(obj);
				if obj.kind.is_unk() {
					unk_count += 1;
				} else {
					let is_self = match obj.val {
						EVal::TSelf => true,
						_ => false
					};
					/*println!("GET METHOD");
					match env.fun_env().get_cls(&obj.kind) {
						Some(c) => unsafe {(*c).print()},
						_ => panic!()
					};*/
					match env.fun_env().get_attrib(&obj.kind, pname, is_self) {
						Some( (tp,is_m) ) => {
							expr.kind = tp;
							*m_flag = is_m;
						},
						_ => throw!(format!("property {} not found for {:?}", pname, obj.kind), expr.addres)
					}
				}
			},
			EVal::ChangeType(ref mut e, ref mut tp) => {
				check!(e);
				check_type!(tp);
			}
			EVal::TSelf =>
				match env.self_val() {
					Some(tp) => expr.kind = tp,
					_ => throw!("using 'self' out of class", expr.addres)
				},
			_ => ()
		}
		return Ok(unk_count);
	}
	#[allow(mutable_transmutes)]
	unsafe fn check_type_pack(&self, pack : &Pack, tmpl : &Vec<String>, t : &Type, addr : &Cursor) -> CheckRes {
		macro_rules! rec {($t:expr) => {self.check_type_pack(pack, tmpl, $t, addr)?}; }
		let t : &mut Type = mem::transmute(t);
		match *t {
			Type::Arr(ref mut item) => {
				rec!(&mut item[0]);
			},
			Type::Class(ref mut pref, ref name, ref mut params) => {
				match *params {
					Some(ref mut params) =>
						for t in params.iter_mut() {
							rec!(t);
						},
					_ =>
						// CHECK FOR TEMPLATE
						if pref.len() == 0 {
							for name1 in tmpl.iter() {
								if name == name1 {
									// TEMPLATE FOUND
									pref.push("%tmpl".to_string());
									return Ok(())
								}
							}
						}
				}
				pack.check_class(pref, name, params, addr)?;
			}
			Type::Fn(_, ref mut args, ref mut res) => {
				/*match *tmpl {
					Some(ref tmpl) =>
						for t in tmpl.iter() {
							rec!(t);
						},
					_ => ()
				}*/
				for t in args.iter_mut() {
					rec!(t);
				}
				rec!(&**res);
			},
			_ => ()
		}
		Ok(())
	}
	// CHANGING t VAR AS MUTABLE
	#[allow(mutable_transmutes)]
	unsafe fn check_type(&self, env : &LocEnv, t : &Type, addr : &Cursor) -> CheckRes {
		macro_rules! rec {($t:expr) => {self.check_type(env, $t, addr)?}; }
		let t : &mut Type = mem::transmute(t);
		match *t {
			Type::Arr(ref mut item) => {
				rec!(&mut item[0]);
			},
			Type::Class(ref mut pref, ref name, ref mut params) => {
				match *params {
					Some(ref mut params) =>
						for t in params.iter_mut() {
							rec!(t);
						},
					_ => ()
				}
				try!(env.fun_env().check_class(pref, name, params, addr));
			}
			Type::Fn(_, ref mut args, ref mut res) => {
				/*match *tmpl {
					Some(ref tmpl) =>
						for t in tmpl.iter() {
							rec!(t);
						},
					_ => ()
				}*/
				for t in args.iter_mut() {
					rec!(t);
				}
				rec!(&**res);
			},
			_ => ()
		}
		Ok(())
	}
}
