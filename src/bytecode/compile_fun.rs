//use bytecode::func::*;
use bytecode::cmd::*;
use bytecode::registers::*;
use bytecode::state::*;
use bytecode::global_conf::*;
use bytecode::compile_act as c_act;
use syn::*;
use std::collections::HashMap;

pub struct CFun {
	pub name    : String,
	pub arg_cnt : u8,
	pub out_cnt : u8,
	pub stack_i : u8,
	pub stack_r : u8,
	pub stack_v : u8,
	pub var_i   : u8,
	pub var_r   : u8,
	pub var_v   : u8,
	pub body    : Vec<Cmd>
}

impl Show for CFun {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		let mut res = vec![format!("{}fn {}", tab, self.name)];
		tab.push(' ');
		tab.push(' ');
		res.push(format!("{}args:  {}", tab, self.arg_cnt));
		res.push(format!("{}outer: {}", tab, self.out_cnt));
		res.push(format!("{}stack: {} {} {}", tab, self.stack_i, self.stack_r, self.stack_v));
		res.push(format!("{}local: {} {} {}", tab, self.var_i, self.var_r, self.var_v));
		tab.pop();
		tab.pop();
		for c in self.body.iter() {
			for l in c.show(layer + 1) {
				res.push(l);
			}
		}
		res.push(format!("{}endfn", tab));
		res
	}
}

pub fn compile(fun : &SynFn/*, dst : &mut Vec<CodeFn>, */) -> CFun {
	let mut env = Env{
		out   : HashMap::new(),
		args  : HashMap::new(),
		loc_i : HashMap::new(),
		loc_r : HashMap::new(),
		loc_v : HashMap::new()
	};
	make_env(fun, &mut env);
	let gc = GlobalConf::new(6);
	let mut state = State::new(env, &gc, "main".to_string());
	state.exc_off = fun.no_except;
	let mut body = vec![];
	state.push_trycatch();
	c_act::compile(&fun.body, &mut state, &mut body);
	if body.len() == 0 || match body[body.len() - 1] {Cmd::Ret(_) => false, _ => true} {
		body.push(Cmd::Ret(Reg::Null))
	}
	if !fun.no_except {
		body.push(Cmd::Label(state.try_catch_label()));
		body.push(Cmd::ReRaise);
		body.push(Cmd::Ret(Reg::Null))
	}
	optimize_movs(&mut body);
	let mut max_i = 0;
	let mut max_r = 0;
	let mut max_v = 0;
	get_stacks_size(&body, &mut max_i, &mut max_r, &mut max_v);
	CFun {	
		name    : format!("{}_{}", state.mod_name, fun.name),
		arg_cnt : fun.args.len() as u8,
		out_cnt : fun.outers.len() as u8,
		stack_i : max_i,
		stack_r : max_r,
		stack_v : max_v,
		var_i   : state.env.loc_i.len() as u8,
		var_r   : state.env.loc_r.len() as u8,
		var_v   : state.env.loc_v.len() as u8,
		body    : body
	}
}

fn get_stacks_size(cmds : &Vec<Cmd>, si : &mut u8, sr : &mut u8, sv : &mut u8) {
	macro_rules! set_max {($var:expr, $val:expr) => {if $val > *$var {*$var = $val}}; }
	for c in cmds.iter() {
		let checked = {
			let reg = c.get_out();
			match reg {
				Some(reg) => {
					match *reg {
						Reg::IStack(ref n) => set_max!(si, *n + 1),
						Reg::RStack(ref n) => set_max!(sr, *n + 1),
						Reg::VStack(ref n) => set_max!(sv, *n + 1),
						_ => ()
					}
					true
				},
				_ => false
			}
		};
		if !checked {
			match *c {
				Cmd::If(_, ref a, ref b) => {
					get_stacks_size(a, si, sr, sv);
					get_stacks_size(b, si, sr, sv);
				},
				Cmd::Catch(ref catchs, _) =>
					for catch in catchs.iter() {
						get_stacks_size(&catch.code, si, sr, sv)
					},
				_ => ()
			}
		}
	}
}

fn optimize_movs(code : &mut Vec<Cmd>) {
	let mut i = 0;
	let mut used = Vec::new();
	while i < code.len() {
		if i == 0 {
			i += 1;
			continue
		}
		let ismov = match code[i] {Cmd::Mov(_,_) => true, _ => false};
		if ismov {
			unsafe {
				let replace_by : Option<Reg> = match code[i].get_out() {
					Some(r) => {
						if r.is_int() || r.is_real() {
							Some(r.clone())
						} else {
							code[i-1].regs_in_use(&mut used);
							let mut cross = false;
							for ru in used.iter() {
								cross = cross || **ru == *r;
							}
							if !cross {
								Some(r.clone())
							} else {
								None
							}
						}
					},
					_ => None
				};
				match replace_by {
					Some(r) => {
						code[i-1].set_out(r);
						code.remove(i);
					},
					_ => i += 1
				}
			}
		} else {
			i += 1;
		}
	}
}

fn make_env(fun : &SynFn, env : &mut Env) {
	for i in 0 .. fun.args.len() {
		env.args.insert(fun.args[i].name.clone(), i as u8);
	}
	for i in 0 .. fun.outers.len() {
		env.out.insert(fun.outers[i].clone(), i as u8);
	}
	fn act(action : &ActF, env : &mut Env) {
		macro_rules! add {($store:expr, $name:expr) => {
			if ! $store.contains_key($name) {
				let len = $store.len() as u8;
				$store.insert($name.clone(), len + 1);
			}
		};}
		match action.val {
			//ActVal::Expr(ref e) => expr(e, env),
			ActVal::DVar(ref name, ref tp, _) => {
				if tp.is_int() || tp.is_char() || tp.is_bool() {
					add!(env.loc_i, name)
				} else if tp.is_real() {
					add!(env.loc_r, name)
				} else {
					add!(env.loc_v, name)
				}
			},
			ActVal::DFun(ref df) => add!(env.loc_v, &df.name),
			ActVal::While(_, _, ref acts) =>
				for a in acts.iter() {
					act(a, env)
				},
			ActVal::For(_, ref name, _, _, ref acts) => {
				add!(env.loc_i, name);
				for a in acts.iter() {
					act(a, env)
				}
			},
			ActVal::Foreach(_, ref name, ref tp, _, ref acts) => {
				if tp.is_int() || tp.is_char() || tp.is_bool() {
					add!(env.loc_i, name)
				} else if tp.is_real() {
					add!(env.loc_r, name)
				} else {
					add!(env.loc_v, name)
				}
				for a in acts.iter() {
					act(a, env)
				}
			},
			ActVal::If(_, ref acts1, ref acts2) => {
				for a in acts1.iter() {
					act(a, env)
				}
				for a in acts2.iter() {
					act(a, env)
				}
			},
			ActVal::Try(ref acts, ref ctch) => {
				for a in acts.iter() {
					act(a, env)
				}
				for c in ctch.iter() {
					match c.vname {
						Some(ref v) => {
							if c.vtype.is_int() || c.vtype.is_bool() || c.vtype.is_char() {
								add!(env.loc_i, v);
							} else if c.vtype.is_real() {
								add!(env.loc_r, v);
							} else {
								add!(env.loc_v, v);
							}
						},
						_ => ()
					}
					for a in c.act.iter() {
						act(a, env)
					}
				}
			},
			_ => ()
		}
	}
	for a in fun.body.iter() {
		act(a, env);
	}
}

