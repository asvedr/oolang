use bytecode::func::*;
use bytecode::cmd::*;
use bytecode::registers::*;
use std::collections::HashMap;
use syn::*;
use std::fmt::Write;

pub fn compile(fun : &SynFn/*, dst : &mut Vec<CodeFn>, */) {
	let mut env = Env{
		out   : HashMap::new(),
		args  : HashMap::new(),
		loc_i : HashMap::new(),
		loc_r : HashMap::new(),
		loc_v : HashMap::new()
	};
	make_env(fun, &mut env);
	env.print()
}

struct Env {
	out   : HashMap<String,u8>,
	args  : HashMap<String,u8>,
	loc_i : HashMap<String,u8>,
	loc_r : HashMap<String,u8>,
	loc_v : HashMap<String,u8>
}

impl Show for Env {
	fn show(&self, _ : usize) -> Vec<String> {
		let mut res = vec![];
		macro_rules! go {($name:expr, $store:ident) => {{
			let mut line = String::new();
			let _ = write!(line, "{}: {}", $name, '{');
			for name in self.$store.keys() {
				let _ = write!(line, "{}:{}, ", name, self.$store.get(name).unwrap());
			}
			let _ = write!(line, "{}", '}');
			res.push(line);
		}};}
		go!("OUTERS", out);
		go!("ARGS", args);
		go!("LOC INT",  loc_i);
		go!("LOC REAL", loc_r);
		go!("LOC VAR",  loc_v);
		res
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
