use bytecode::state::*;
use bytecode::registers::*;
use bytecode::cmd::*;

pub fn compile(e : &Expr, state : &mut State, cmds : &mut Vec<Cmd>) -> Reg {
	match e {	
		EVal::Int(ref v)  => {
			let reg = Reg::IStack(state.push());
			cmds.push(Cmd::SetI(*v, reg.clone()));
			reg
		},
		EVal::Real(ref v) => {
			let reg = Reg::RStack(state.push());
			cmds.push(Cmd::SetR(*v, reg.clone()));
			reg
		},
		EVal::Str(ref v) => {
			let reg = Reg::VStack(state.push());
			cmds.push(Cmd::SetS(v.clone(), reg.clone()));
			reg
		},
		EVal::Char(ref c) => {
			let v = *c as isize;
			let reg = Reg::IStack(state.push());
			cmds.push(Cmd::SetI(v, reg.clone()));
			reg
		},
		EVal::Bool(ref b) => {
			let reg = Reg::IStack(state.push());
			cmds.push(Cmd::SetI(if b {1} else {0}, reg.clone()));
			reg
		},
		EVal::Var(ref pref, ref var) => {
			if pref[0] == "%loc" {
				state.env.get_loc_var(var, &e.kind)
			} else if pref[0] == "%out" {
				Reg::Env(state.env.out.get(var).unwrap())
			} else if pref[0] == "%mod" {
			} else if pref[0] == "%std" {
			} else {
			}
		},
		EVal::Call(ref tp, ref fun, ref args) => {
			match fun.val {
				EVal::Var(ref pref, ref name) => {
					match pref {
						
					}
				},
				_ => {
					let mut c_args = vec![];
					for a in args.iter() {
						c_args.push(compile(a, state));
					}
					let f = compile(fun, state);
					state.pop_v();
					for a in c_args.iter() {
						if a.is_int() {
							state.pop_i();
						} else if a.is_real() {
							state.pop_r();
						} else {
							state.pop_v();
						}
					};
					let dst = match *tp {
						Type::Int|Type::Char|Type::Bool => Reg::IStack(state.push_i()),
						Type::Real => Reg::RStack(state.push_r()),
						Type::Void => Reg::Null,
						_ => Reg::VStack(stack.push_v())
					};
					let call = Call {
						func        : f,
						args        : c_args,
						dst         : dst.clone(),
						can_throw   : panic!(),
						catch_block : panic!()
					};
					cmds.push(Cmd::Call(call));
					dst
				}
			}
		},
		NewClass   (Option<Vec<Type>>,Vec<String>,String,Vec<Expr>),
		EVal::Item(ref arr, ref index) => {
			let arr_c = compile(arr, state);
			let ind_c = compile(arr, state);
			state.pop_v();
			state.pop_i();
			macro_rules! make_cmd{($a:expr,$i:expr,$d:expr) => {{
				match arr.kind {
					Type::Str   => Cmd::ItemStr($a,$i,$d),
					Type::Arr   => Cmd::ItemVec($a,$i,$d),
					_ /* asc */ => Cmd::ItemAsc($a,$i,$d)
				}
			}};}
			match e.kind {
				Type::Int|Type::Char|Type::Bool => {
					let r = state.push_i();
					make_cmd!(arr_c, ind_c, r.clone());
					r
				},
				Type::Real => {
					let r = state.push_r();
					make_cmd!(arr_c, ind_c, r.clone());
					r
				},
				_ => {
					let r = state.push_v();
					make_cmd!(arr_c, ind_c, r.clone());
					r
				}
			}
		},
		Arr        (Vec<Expr>),                   // new arr
		Asc        (Vec<Pair<Expr,Expr>>),        // new Asc. Only strings, chars and int allowed for key
		//          obj       pname  is_meth
		Attr       (Box<Expr>,String,bool),       // geting class attrib: 'object.prop' or 'object.fun()'
		EVal::ChangeType(ref val, ref tp) => {
			let reg = compile(val, state);
			/*match val.kind {
				Type::Int  => 
					match *tp {
						Type::Real =>
						Type::Str  => cmds.push(Cmd::Conv(reg.clone(), Convert::ITOS, reg.clone()))
						Type::Bool => cmds.push(Cmd::Conv(reg.clone(), Convert::ITOB, reg.clone())),
						_ => reg
					}
				Type::Bool =>
				Type::Char =>
				Type::Str  =>
				_ => reg
			}*/
			reg
		},
		EVal::TSelf => Reg::RSelf,
		EVal::Null => Reg::Null
	}
}
