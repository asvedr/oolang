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

/*
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
*/
