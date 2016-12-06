use syn::*;
use type_check::utils::*;
use type_check::tclass::*;
use type_check::pack::*;
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
	int_real_op : HashMap<String,Type>,
	int_op      : HashMap<String,Type>,
	real_op     : HashMap<String,Type>,
	all_op      : HashMap<String,Type>,
	bool_op     : HashMap<String,Type>,
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
					match *p {
						None => $env.replace_unk(n, &$tp),
						_ => ()
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
		macro_rules! adds {
				($s:expr, $a:expr) => {$s.insert($a.to_string(), Type::Unk)};
				($s:expr, $a:expr, $v:expr) => {$s.insert($a.to_string(), $v)};
		}
		adds!(res.int_real_op, "+", Type::Unk);
		adds!(res.int_real_op, "-", Type::Unk);
		adds!(res.int_real_op, "*", Type::Unk);
		adds!(res.int_real_op, "/", Type::Unk);
		adds!(res.int_real_op, ">", Type::Bool);
		adds!(res.int_real_op, "<", Type::Bool);
		adds!(res.int_real_op, ">=",Type::Bool);
		adds!(res.int_real_op, "<=",Type::Bool);
		adds!(res.int_op,      "%");
		adds!(res.real_op,     "**");
		adds!(res.all_op,      "==");
		adds!(res.all_op,      "!=");
		adds!(res.bool_op,     "&&");
		adds!(res.bool_op,     "||");
		res
	}
	// this is only one public fun for checking
	pub fn check_mod(&self, smod : &mut SynMod) -> CheckRes {
		let mut pack = Pack::new();
		for c in self.std.pack.cls.keys() {
			pack.out_cls.insert(c.clone(), &self.std.pack);
		}
		pack.packs.insert("%std".to_string(), &self.std.pack);
		for f in smod.funs.iter() {
			let n = match f.name {
				Some(ref n) => n.clone(),
				_ => panic!()
			};
			match pack.fns.insert(n, f.ftype.clone()) {
				Some(_) => throw!("fun with this name already exist in this module", f.addr),
				_ => ()
			}
		}
		// TODO CLASS CHECK
		/*
		for c in smod.classes.iter() {
			let n = match 
		}
		for c in smod.classes.iter_mut() {
			try!(self.check_class(&pack, c));
		}
		*/
		for f in smod.funs.iter_mut() {
			try!(self.check_fn(&pack, f, None, None));
		}
		ok!()
	}
	/*fn check_class(&self, pack : &Pacl, class : &mut Class) -> CheckRes {
		
	}*/
	fn check_fn(&self, pack : &Pack, fun : &mut SynFn, out_env : Option<&LocEnv>, _self : Option<Type>) -> CheckAns<isize> {
		let mut env = LocEnv::new(&*pack, &fun.tmpl, _self);
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
			try!(self.check_type(&env, &mut arg.tp, &fun.addr));
			let p : *mut Type = &mut arg.tp;
			add_loc_knw!(env, &arg.name, p, fun.addr);
		}
		// RET TYPE
		try!(self.check_type(&env, &mut fun.rettp, &fun.addr));
		env.set_ret_type(&fun.rettp);
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
					return Ok(0)
				}
			}
		} else {
			self.check_actions(&mut env, &mut fun.body, false)
		}
	}
	fn check_actions(&self, env : &mut LocEnv, src : &mut Vec<ActF>, repeated : bool) -> CheckAns<isize> {
		// 'repeated' is a flag for non first check
		// if it true then var won't added to env on DefVar
		let mut unk_count = 0;
		macro_rules! expr {($e:expr) => { unk_count += try!(self.check_expr(env, $e))}; }
		macro_rules! actions {($e:expr, $a:expr) => { unk_count += try!(self.check_actions($e, $a, false)) }; }
		let env_pt : *mut LocEnv = &mut *env;
		let env_ln : &mut LocEnv = unsafe{ mem::transmute(env_pt) };
		macro_rules! regress {
			($e:expr, $t:expr) => {try!(regress_expr(env_ln, $e, $t))};
			($env:expr, $e:expr, $t:expr) => {try!(regress_expr($env, $e, $t))};
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
								regress!(e, env.ret_type());
							} else if !env.check_ret_type(&e.kind) {
								throw!(format!("expect type {:?}, found {:?}", env.ret_type(), e.kind), act.addres)
							}
						},
						None =>
							if !env.check_ret_type(&Type::Void) {
								throw!(format!("expect type {:?}, found void", env.ret_type()), act.addres)
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
					match *tp {
						Type::Unk => {
							*tp = match *val {
								None => Type::Unk,
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
							try!(self.check_type(env, tp, &act.addres));
							if !repeated {
								add_loc_knw!(env, name, tp, act.addres);
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
									regress!(v, tp)
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
						regress!(var, &val.kind);
					} else if ub {
						// REGRESS CALL
						regress!(val, &var.kind);
					} else if var.kind != val.kind {
						throw!(format!("assign parts incompatible: {:?} and {:?}", var.kind, val.kind), act.addres)
					}
					// ADD CHECK FOR WHAT CAN BE AT LEFT PART OF ASSIG
				},
				ActVal::Throw(ref mut e) => {
					// CAN RETURN ANY CLASS BUT CAN'T RETURN A PRIMITIVE
					expr!(e);
					if !e.kind.is_class() || e.kind.is_arr() {
						throw!(format!("expr must be a class"), e.addres)
					}
				},
				ActVal::DFun(ref mut df) => {
					if !repeated {
						match df.name {
							Some(ref name) => add_loc_knw!(env, name, &df.ftype, df.addr),
							_ => panic!()
						}
					}
					let pack : &Pack = env.pack();
					let _self = env.self_val().clone();
					unk_count += try!(self.check_fn(pack, &mut **df, Some(env), _self));
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
						let mut sub = LocEnv::inherit(env);
						match catch.except {
							Some(ref mut t) => {
								try!(self.check_type(env, t, &catch.addres));
								match catch.vname {
									Some(ref name) => add_loc_knw!(sub, name, t, catch.addres),
									_ => ()
								}
							},
							_ => ()
						}
						actions!(&mut sub, &mut catch.act);
						//unk_count += try!(self.check_actions(&mut sub, &mut catch.act, false/*repeated*/));
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
						let int_vt = Type::Int;
						add_loc_knw!(sub, vname, &int_vt, act.addres);
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
						match cont.kind {
							Type::Arr(ref mut item) => {
								if vt.is_unk() {
									*vt = item[0].clone();
									add_loc_unk!(sub, vname, &mut item[0], act.addres);
								} else if *vt != item[0] {
									throw!(format!("foreach var expected {:?}, found {:?}", item, vt), act.addres);
								} else {
									add_loc_knw!(sub, vname, &item[0], act.addres);
								}
							},
							Type::Unk => {
								if vt.is_unk() {
									add_loc_unk!(sub, vname, &mut *vt, act.addres);
								} else {
									regress!(&mut sub, cont, vt);
									//try!(regress_expr(sub, cont, vt))
									add_loc_knw!(sub, vname, &mut *vt, act.addres);
								}
							},
							_ => throw!("you can foreach only through array", cont.addres)
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
		let mut unk_count = 0;
		// recursive check expression
		macro_rules! check {($e:expr) => {unk_count += try!(self.check_expr(env, $e))};};
		macro_rules! check_type {($t:expr) => {try!(self.check_type(env, $t, &expr.addres))}}
		macro_rules! regress {($e:expr, $t:expr) => {try!(regress_expr(env, $e, $t))}; }
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
					match *pref {
						Some(ref vec) =>
							if vec[0] == "#opr" {
								is_in!(name, $o, int_real_op, is_in!(name, $o, int_op, is_in!(name, $o, real_op, is_in!(name, $o, all_op, is_in!(name, $o, bool_op, None)))))
							} else {
								None
							},
						_ => None
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
			EVal::Call(ref mut tmpl, ref mut f, ref mut args) => {
				let mut res_type = Type::Unk;
				let chf : Option <*const HashMap<String,Type>> = check_fun!(**f, res_type);
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
					Some(seq_l) => unsafe {
						// CHECK OPERATION
						let a : *const Type = &args[0].kind;
						let b : *const Type = &args[1].kind;
						// INT OR REAL OPERATIONS + - * >= <= < > / 
						if seq_l == &self.int_real_op {
							macro_rules! ok {($tp:expr) => {{
								//set_var_type!(env, args[0], $tp);
								//set_var_type!(env, args[1], $tp);
								// REGRESS CALL
								regress!(&mut args[0], &$tp);
								regress!(&mut args[1], &$tp);
								match res_type {
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
									ok!(Type::Int)
								} else if (*b).is_real() {
									let addr = args[0].addres.clone();
									let arg = mem::replace(&mut args[0].val, EVal::Null);
									let arg = Expr{val : arg, kind : Type::Int, addres : addr, op_flag : 0};
									args[0].val = EVal::ChangeType(Box::new(arg), Type::Real);
									args[0].kind = Type::Real;
									//args[0] = Expr{val : EVal::ChangeType(Box::new(args[0]), Type::Real), kind : Type::Real, addres : addr};
									ok!(Type::Real)
								} else {
									throw!(format!("expect int found {:?}", *b), args[1].addres.clone())
								}
							} else if (*a).is_real() {
								if (*b).is_real() || (*b).is_unk() {
									ok!(Type::Real)
								} else if (*b).is_int() {
									let addr = args[1].addres.clone();
									let arg = mem::replace(&mut args[1].val, EVal::Null);
									let arg = Expr{val : arg, kind : Type::Int, addres : addr, op_flag : 0};
									args[1].val = EVal::ChangeType(Box::new(arg), Type::Real);
									args[1].kind = Type::Real;
									//args[1] = Expr{val : EVal::ChangeType(Box::new(args[1]), Type::Real), kind : Type::Real, addres : addr};
									ok!(Type::Real)
								} else {
									throw!(format!("expect real found {:?}", *b), args[1].addres.clone())
								}
							} else if (*a).is_unk() {
								if (*b).is_int() {
									ok!(Type::Int)
								} else if (*b).is_real() {
									ok!(Type::Real)
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
								regress!(&mut args[0], &Type::Int);
								regress!(&mut args[1], &Type::Int);
								f.kind = type_fn!(vec![Type::Int, Type::Int], Type::Int);
								expr.kind = Type::Int
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
								regress!(&mut args[0], &Type::Real);
								regress!(&mut args[1], &Type::Real);
								f.kind = type_fn!(vec![Type::Real, Type::Real], Type::Real);
								expr.kind = Type::Real
							}
						// ALL OPERATIONS
						} else if seq_l == &self.all_op {
							if (*a).is_unk() && (*b).is_unk() {
								// PASS
							} if *a == *b || (*a).is_unk() || (*b).is_unk() {
								let tp : Type;
								if (*a).is_unk() {
									tp = (*b).clone();
									regress!(&mut args[0], &tp);
								} else if (*b).is_unk() {
									tp = (*a).clone();
									regress!(&mut args[1], &tp);
								} else {
									tp = (*a).clone();
								}
								f.kind = type_fn!(vec![tp.clone(), tp], Type::Bool);
								expr.kind = Type::Bool
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
								regress!(&mut args[0], &Type::Bool);
								regress!(&mut args[1], &Type::Bool);
								f.kind = type_fn!(vec![Type::Bool, Type::Bool], Type::Bool);
								expr.kind = Type::Bool
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
						match f.kind {
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
											regress!(a, t);
										} else {
											throw!(format!("expect {:?}, found {:?}", t, a.kind), a.addres.clone())
										}
									}
									expr.kind = (**res_t).clone();
								}
							},
							Type::Unk => unk_count += 1,
							ref t => throw!(format!("expect Fn found {:?}", t), f.addres.clone())
						}
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
				try!(env.check_class(pref, name, tmpl, &expr.addres));
				for a in args.iter_mut() {
					check!(a);
				}
				unsafe {
					let cls =
						if pref[0] == "%mod" {
							/*(*env.global)*/env.pack().get_cls(None, name).unwrap()
						}
						else { 
							/*(*env.global)*/env.pack().get_cls(Some(pref), name).unwrap()
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
							regress!(&mut args[i], &*(*cls).args[i]);
						} else if *(*cls).args[i] != args[i].kind {
							throw!(format!("expected {:?}, found {:?}", *(*cls).args[i], args[i].kind), &args[i].addres)
						}
					}
				}
			},
			EVal::Item(ref mut a, ref mut i) => {
				check!(a);
				check!(i);
				if a.kind.is_unk() {
					unk_count += 1;
				} else if a.kind.is_arr() {
					if i.kind.is_int() {
						// ALL OK
					} else if i.kind.is_unk() {
						// REGRESS CALL
						regress!(i, &Type::Int);
					} else {
						throw!(format!("expect int, found {:?}", i.kind), i.addres.clone())
					}
					expr.kind = a.kind.arr_item().clone();
				} else if a.kind.is_asc() {
					let mut key = Type::Unk;
					let mut val = Type::Unk;
					a.kind.asc_key_val(&mut key, &mut val);
					if key == i.kind {
						// ALL OK
					} else if i.kind.is_unk() {
						// REGRESS CALL
						regress!(i, &key);
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
				try!(env.get_var(pref, name, &mut expr.kind, &expr.addres));
				/* MUST RECUSRIVE CHECK FOR COMPONENTS */
				match expr.kind {
					Type::Unk => return Ok(1),
					_ => return Ok(0)
				}
			},
			EVal::Arr(ref mut items) => {
				let mut item : Option<Type> = match expr.kind {
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
							regress!(i, &t);
						}
						expr.kind = Type::Arr(/*Box::new*/vec![t]);
					},
					_ => unk_count += 1
				}
			},
			EVal::Asc(ref mut items) => {
				let mut key_type : Option<Type> = None;
				let mut val_type : Option<Type> = None;
				if expr.kind.is_asc() {
					let mut a = Type::Unk;
					let mut b = Type::Unk;
					expr.kind.asc_key_val(&mut a, &mut b);
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
					match pair.a.kind {
						Type::Str  => skey!(Type::Str,  pair.a.addres),
						Type::Char => skey!(Type::Char, pair.a.addres),
						Type::Int  => skey!(Type::Int,  pair.a.addres),
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
								regress!(&mut pair.a, &k);
								regress!(&mut pair.b, &v);
							}
							expr.kind = Type::Class(vec!["%std".to_string()], "Asc".to_string(), Some(vec![k, v]));
						},
						_ => unk_count += 1
					},
					_ => unk_count += 1
				}
			},
			EVal::Prop(ref mut obj, ref pname) => {
				check!(obj);
				if obj.kind.is_unk() {
					unk_count += 1;
				} else {
					let is_self = match obj.val {
						EVal::TSelf => true,
						_ => false
					};
					match env.get_method(&obj.kind, pname, is_self) {
						Some(f) => expr.kind = f,
						_ => throw!(format!("method {} not found for {:?}", pname, obj.kind), expr.addres)
					}
				}
			},
			EVal::ChangeType(ref mut e, ref mut tp) => {
				check!(e);
				check_type!(tp);
			}
			EVal::TSelf =>
				match *env.self_val() {
					Some(ref tp) => expr.kind = tp.clone(),
					_ => throw!("using 'self' out of class", expr.addres)
				},
			_ => ()
		}
		return Ok(unk_count);
	}
	fn check_type(&self, env : &LocEnv, t : &mut Type, addr : &Cursor) -> CheckRes {
		macro_rules! rec {($t:expr) => {try!(self.check_type(env, $t, addr))}; }
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
				try!(env.check_class(pref, name, params, addr));
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
				rec!(&mut **res);
			},
			_ => ()
		}
		Ok(())
	}
}
