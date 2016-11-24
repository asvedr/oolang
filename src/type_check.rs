use syn_common::*;
use type_check_utils::*;
use std::collections::{HashMap/*, HashSet, BTreeMap*/};
use std::mem;
use regressor::*;

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
	packs       : HashMap<String,Pack>
}

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
			packs       : HashMap::new()
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
	pub fn check_mod(&self, smod : &mut SynMod) -> CheckRes {
		let mut pack = Pack::new();
		for f in smod.funs.iter() {
			let n = match f.name {
				Some(ref n) => n.clone(),
				_ => panic!()
			};
			match pack.fns.insert(n, f.type_of()) {
				Some(_) => throw!("fun with this name already exist in this module", f.addr),
				_ => ()
			}
		}
		for f in smod.funs.iter_mut() {
			try!(self.check_fn(&pack, f, None));
		}
		ok!()
	}
	fn check_fn(&self, pack : &Pack, fun : &mut SynFn, out_env : Option<&LocEnv>) -> CheckRes {
		let mut env = LocEnv::new(&*pack);
		for t in fun.tmpl.iter() {
			env.templates.insert(t.clone());
		}
		for arg in fun.args.iter_mut() {
			try!(self.check_type(&env, &mut arg.tp, &fun.addr));
			let p : *mut Type = &mut arg.tp;
			add_loc_knw!(env, arg.name.clone(), p, fun.addr);
		}
		try!(self.check_type(&env, &mut fun.rettp, &fun.addr));
		env.set_ret_type(&fun.rettp);
		self.check_actions(&mut env, &mut fun.body)
	}
	fn check_actions(&self, env : &mut LocEnv, src : &mut Vec<ActF>) -> CheckRes {
		macro_rules! expr {($e:expr) => {try!(self.check_expr(env, $e))}; }
		for act in src.iter_mut() {
			match act.val {
				ActVal::Expr(ref mut e) => act.exist_unk = expr!(e),
				ActVal::Ret(ref mut opt_e) => {
					match *opt_e {
						Some(ref mut e) => {
							expr!(e);
							if e.kind.is_unk() {
								// REGRESS CALL
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
				ActVal::DVar(ref name, ref mut tp, ref mut val) => {
					match *val {
						Some(ref mut val) => {
							//println!("<<<");
							act.exist_unk = expr!(val);
							//println!(">>>");
						},
						None => {
							act.exist_unk = false;
						}
					}
					match *tp {
						Type::Unk => {
							*tp = match *val {
								None => Type::Unk,
								Some(ref mut v) => {
									v.kind.clone()
								}
							};
							add_loc_unk!(env, name, tp, act.addres);
						},
						_ => {
							// regression recovery
							add_loc_knw!(env, name, tp, act.addres);
						}
					}
				},
				ActVal::Asg(ref mut var, ref mut val) => {
					act.exist_unk = expr!(var) || expr!(val);
					let ua = var.kind.is_unk();
					let ub = val.kind.is_unk();
					if ua && ub {
						// PASS
					} else if ua {
						// REGRESS CALL
					} else if ub {
						// REGRESS CALL
					} else if var.kind != val.kind {
						throw!(format!("assign parts incompatible: {:?} and {:?}", var.kind, val.kind), act.addres)
					}
				},
				ActVal::Throw(ref mut e) => {
					expr!(e);
					if !e.kind.is_class() {
						throw!(format!("expr must be a class"), e.addres)
					}
				},
				//ActVal::Try() => {}
				_ => ()
			}
		}
		ok!()
	}
	fn check_expr(&self, env : &LocEnv, expr : &mut Expr) -> CheckAns<bool> {
		//println!("CHECK EXPR");
		let mut has_unk = false;
		// recursive check expression
		macro_rules! check {($e:expr) => {has_unk = try!(self.check_expr(env, $e)) || has_unk};};
		macro_rules! check_type {($t:expr) => {try!(self.check_type(env, $t, &expr.addres))}}
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
									has_unk = true;
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
								f.kind = type_fn!(vec![Type::Real, Type::Real], Type::Real);
								expr.kind = Type::Real
							}
						// ALL OPERATIONS
						} else if seq_l == &self.all_op {
							if *a == *b {
								f.kind = type_fn!(vec![(*a).clone(), (*b).clone()], Type::Bool);
								expr.kind = Type::Bool
							} else {
								throw!(format!("expect {:?}, found {:?}", *a, *b), args[1].addres.clone())
							}
						// BOOL OPERATIONS (bool, bool) -> bool
						} else /* if seq_l == &self.bool_op */ {
							expr.kind = Type::Bool
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
										let a = &args[i];
										let t = &args_t[i];
										if a.kind == *t {
											// ALL OK
										} else if a.kind.is_unk() {
											// REGRESS CALL
										} else {
											throw!(format!("expect {:?}, found {:?}", t, a.kind), a.addres.clone())
										}
									}
									expr.kind = (**res_t).clone();
								}
							},
							Type::Unk => has_unk = true,
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
							(*env.global).get_cls(None, name).unwrap()
						}
						else { 
							(*env.global).get_cls(Some(pref), name).unwrap()
						};
					if (*cls).params != pcnt {
						throw!(format!("class {} expect {} params, given {}", name, (*cls).params, pcnt), &expr.addres)
					}
					if (*cls).args.len() != args.len() {
						throw!(format!("class {} initializer expect {} args, given {}", name, (*cls).args.len(), args.len()), &expr.addres)
					}
					for i in 0 .. args.len() {
						if args[i].kind.is_unk() {
							// REGRESS CALL
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
					has_unk = true;
				} else if a.kind.is_arr() {
					if i.kind.is_int() {
						// ALL OK
					} else if i.kind.is_unk() {
						// REGRESS CALL
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
					} else {
						throw!(format!("expect {:?}, found {:?}", key, i.kind), i.addres.clone())
					}
					expr.kind = val;
				} else {
					throw!(format!("expect arr or asc, found {:?}", a.kind), a.addres.clone())
				}
			},
			EVal::Var(ref mut pref, ref name) => { // namespace, name
				println!("GET VAR FOR {:?} {}", pref, name);
				//println!("{}", env.show());
				try!(env.get_var(pref, name, &mut expr.kind, &expr.addres));
				/* MUST RECUSRIVE CHECK FOR COMPONENTS */
				match expr.kind {
					Type::Unk => return Ok(true),
					_ => return Ok(false)
				}
			},
			EVal::Arr(ref mut items) => {
				let mut item : Option<Type> = match expr.kind {
					Type::Unk => None,
					Type::Arr(ref v) => Some((**v).clone()),
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
					Some(t) => expr.kind = Type::Arr(Box::new(t)),
					_ => has_unk = true
				}
				// REGRESS CALL
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
						Type::Unk  => has_unk = true,
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
						Some(v) => expr.kind = Type::Class(vec!["%std".to_string()], "Asc".to_string(), Some(vec![k, v])),
						_ => has_unk = true
					},
					_ => has_unk = true
				}
				// REGRESS CALL
			},
			EVal::Prop(ref mut obj, _) => check!(obj),
			EVal::ChangeType(ref mut e, ref mut tp) => {
				check!(e);
				check_type!(tp);
			}
			_ => ()
		}
		return Ok(has_unk);
	}
	fn check_type(&self, env : &LocEnv, t : &mut Type, addr : &Cursor) -> CheckRes {
		macro_rules! rec {($t:expr) => {try!(self.check_type(env, $t, addr))}; }
		match *t {
			Type::Arr(ref mut item) => {
				rec!(&mut **item);
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
