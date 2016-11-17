use syn_common::*;
use type_check_utils::*;
use std::collections::{HashMap, HashSet, BTreeMap};

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
	int_real_op : HashSet<String>,
	int_op      : HashSet<String>,
	real_op     : HashSet<String>,
	all_op      : HashSet<String>,
	bool_op     : HashSet<String>,
	packs       : HashMap<String,Pack>
}

impl Checker {
	pub fn new() -> Checker {
		let mut res = Checker {
			int_real_op : HashSet::new(),
			int_op      : HashSet::new(),
			real_op     : HashSet::new(),
			all_op      : HashSet::new(),
			bool_op     : HashSet::new(),
			packs       : HashMap::new()
		};
		macro_rules! adds {($s:expr, $a:expr) => {$s.insert($a.to_string())}}
		adds!(res.int_real_op, "+");
		adds!(res.int_real_op, "-");
		adds!(res.int_real_op, "*");
		adds!(res.int_real_op, "/");
		adds!(res.int_real_op, ">");
		adds!(res.int_real_op, "<");
		adds!(res.int_real_op, ">=");
		adds!(res.int_real_op, "<=");
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
		for arg in fun.args.iter_mut() {
			let p : *mut Type = &mut arg.tp;
			add_loc_knw!(env, arg.name.clone(), p, fun.addr);
		}
		self.check_actions(&mut env, &mut fun.body)
	}
	fn check_actions(&self, env : &mut LocEnv, src : &mut Vec<ActF>) -> CheckRes {
		macro_rules! expr {($e:expr) => {try!(self.check_expr(env, $e))}; }
		for act in src.iter_mut() {
			match act.val {
				ActVal::Expr(ref mut e) => act.exist_unk = expr!(e),
				ActVal::DVar(ref name, ref mut tp, ref mut val) => {
					match *val {
						Some(ref mut val) => {
							act.exist_unk = expr!(val);
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
					/*match *tp {
						Some(ref mut tp) => add_loc!(env, name, &*tp),
						_ => panic!()
					}*/
					// add_loc!(env, name, link_type)
				},
				ActVal::Asg(ref mut var, ref mut val) => {
					act.exist_unk = expr!(var) || expr!(val);
				},
				_ => ()
			}
		}
		ok!()
	}
	fn check_expr(&self, env : &LocEnv, expr : &mut Expr) -> CheckAns<bool> {
		let mut has_unk = false;
		macro_rules! check {($e:expr) => {has_unk = try!(self.check_expr(env, $e)) || has_unk};};
		macro_rules! is_in {
			($e:expr, $seq:ident, $els:expr) => {
				if self.$seq.contains($e) {
					Some(&self.$seq)
				} else {
					$els
				}
			};
		}
		macro_rules! check_fun {($e:expr) =>
			{match $e.val {
				EVal::Var(ref pref, ref name) => {
					match *pref {
						Some(ref vec) =>
							if vec[0] == "#opr" {
								is_in!(name, int_real_op, is_in!(name, int_op, is_in!(name, real_op, is_in!(name, all_op, is_in!(name, bool_op, None)))))
							} else {
								None
							},
						_ => None
					}
				},
				_ => None
			}};
		}
		match expr.kind {
			Type::Unk => (), _ => return Ok(false)
		}
		match expr.val {
			EVal::Call(_, ref mut f, ref mut args) => {
				let chf : Option <*const HashSet<String>> = check_fun!(**f);
				for a in args.iter_mut() {
					check!(a);
				}
				match chf {
					Some(seq_l) => {
						let a = &args[0].kind;
						let b = &args[1].kind;
						if seq_l == &self.int_real_op {
							f.kind = type_fn!(vec![Type::Real, Type::Real], Type::Real);
							expr.kind = Type::Real
						} else if seq_l == &self.int_op {
							f.kind = type_fn!(vec![Type::Int, Type::Int], Type::Int);
							expr.kind = Type::Int
						} else if seq_l == &self.real_op {
							f.kind = type_fn!(vec![Type::Real, Type::Real], Type::Real);
							expr.kind = Type::Real
						} else if seq_l == &self.all_op {
						} else /* if seq_l == &self.bool_op */ {
							expr.kind = Type::Bool
						}
					},
					None => check!(f)
				}
			},
			//NewClass(Option<Vec<Type>>,Option<Vec<String>>,String,Vec<Expr>),
			EVal::Item(ref mut a, ref mut i) => {
				check!(a);
				check!(i);
			},
			EVal::Var(ref mut pref, ref name) => { // namespace, name
				println!("GET VAR FOR {:?} {}", pref, name);
				try!(env.get_var(pref, name, &mut expr.kind, &expr.addres));
				/* MUST RECUSRIVE CHECK FOR COMPONENTS */
				match expr.kind {
					Type::Unk => return Ok(true),
					_ => return Ok(false)
				}
			},
			EVal::Arr(ref mut items) => {
				for i in items.iter_mut() {
					check!(i);
				}
			},
			//Asc(ref items) => ,
			EVal::Prop(ref mut obj, _) => check!(obj),
			//ChangeType(Box<Expr>, Type),
			_ => ()
		}
		return Ok(has_unk);
	}
}
