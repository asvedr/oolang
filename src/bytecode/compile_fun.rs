use func::*;
use cmd::*;
use registers::*;
/*
pub fn compile(fun : &SynFn, dst : &mut Vec<CodeFn>, ) {
	
}
*/

struct Env {
	out   : HashMap<String,u8>,
	args  : HashMap<String,u8>,
	loc_i : HashMap<String,u8>,
	loc_r : HashMap<String,u8>,
	loc_v : HashMap<String,u8>
}

fn make_env(fun : &SynFn) -> Env {
	macro_rules! map {() => {HashMap::new()}; }
	let mut env = Env{out : map!(), args : map!(), loc_i : map!(), loc_r : map!(), loc_v : map!()};
	for i in 0 .. fun.args.len() {
		env.args.insert(fun.args[i].name, i);
	}
	fn act(src : &ActF, dst : &mut Env) {
		match src.val {
			ActVal::Expr(ref e)  => expr(e, dst),
			ActVal::DFun(ref df) => env.loc_v.insert(df.name, env.loc_v.len()),
			ActVal::
		}
	}
	for a in fun.body.iter() {
		act(a, &mut env);
	}
	return env;
}

fn make_env(fun : &SynFn, env : &mut Env) {
	for i in 0 .. fun.args.len() {
		env.args.insert(fun.args[i].name.clone(), i);
	}
	macro_rules! add {($store:expr, $name:expr) => {
		if ! $store.contain_key($name) {
			$store.insert($name.clone(), $store.len())
		}
	};}
	fn act(act : &ActF, env : &mut Env) {
		match act.val {
			ActVal::DVar(ref name, ref tp) => {
				if tp.is_int() || tp.is_char() || tp.is_bool() {
					add!(env.loc_i, name)
				} else tp.is_real() {
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
			ActVal::For(_, ref name, _, _, ref acts) =>
				add!(env.loc_i, name);
				for a in acts.iter() {
					act(a, env)
				},
			ActVal::Foreach(_, ref name, ref tp, _, ref acts) =>
				if tp.is_int() || tp.is_char() || tp.is_bool() {
					add!(env.loc_i, name)
				} else tp.is_real() {
					add!(env.loc_r, name)
				} else {
					add!(env.loc_v, name)
				}
				for a in acts.iter() {
					act(a, env)
				},
			ActVal::If(_, ref acts1, ref acts2) => {
				for a in acts1.iter() {
					act(a, env)
				}
				for a in acts2.iter() {
					act(a, env)
				}
			},
			ActVal::Try(ref act, ref ctch) => {
				for a in acts.iter() {
					act(a, env)
				}
				for c in ctch.iter() {
					
				}
			},
			_ => ()
		}
	}
	for a in fun.body.iter() {
		act(a, env);
	}
}
