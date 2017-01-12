use syn::*;
use bytecode::state::*;
use bytecode::registers::*;
use bytecode::cmd::*;
use bytecode::compile_expr as c_expr;

pub fn compile(acts : &Vec<ActF>, state : &mut State, cmds : &mut Vec<Cmd>) {
	for a in acts.iter() {
		state.clear_stacks();
		match a.val {
			EVal::Expr(ref e) => {
				let out = c_expr::compile(e, state, cmds)
				if out != Reg::Null {
					set_last_mov(Reg::Null)
					/*let i = cmds.len() - 1;
					if match cmds[i] {Mov::(_,_) => true, _ => false} {
						cmds.pop();
						cmds[i-1].set_out(Reg::Null);
					} else {
						cmds[i].set_out(Reg::Null);
					}*/
				}
			},
			EVal::DFun(Box<DF>) => panic!(),
			EVal::DVar(ref name, ref tp, ref val) => {
				let reg = state.env.get_loc_var(name, &**tp);
				match val {
					None => cmds.push(Cmd::Mov(Reg::Null, reg)),
					Some(ref val) => {
						c_expr::compile(e, state, cmds);
						set_last_mov(cmds, reg);
					}
				}
			},
			EVal::Asg(ref var, ref val) => {
				let is_var  = match var.val {EVal::Var(_,_) => true, _ => false};
				let is_item = match var.val {EVal::Item(_,_) => true, _ => false};
				// else is attr
				if is_var {
					let reg = c_expr::compile(var, state, cmds);
					c_expr::compile(val, state, cmds);
					set_last_mov(cmds, reg);
				} else if is_item {
					let src = c_expr::compile(val, state, cmds);
					c_expr::compile(var, state, cmds);
					match cmds[cmds.len() - 1] {
						Cmd::WithItem(ref mut with_it) => {
							with_it.is_get = false;
							with_it.value = src;
						},
						_ => panic!()
					}
				} else { // is attr
					panic!()
				}
			},
			EVal::Ret(ref e) =>
				match *e {
					Some(ref e) => {
						let out = c_expr::compile(e, state, cmds);
						cmds.push(Cmd::Ret(out))
					},
					_ => cmds.push(Cmd::Ret(Reg::Null))
				},
			EVal::Break(_, ref n) => {
				cmds.push(Cmd::Goto(state.break_label(*n)))
			}
			EVal::While(Option<String>, Expr, Vec<Act<DF>>),
			EVal::For(Option<String>,String,Expr,Expr,Vec<Act<DF>>), // for i in range(a + 1, b - 2) {}
			EVal::Foreach(Option<String>,String,RType, Expr,Vec<Act<DF>>),  // for i in array {}
			EVal::If(Expr,Vec<Act<DF>>,Vec<Act<DF>>),
			EVal::Try(Vec<Act<DF>>,Vec<SynCatch<DF>>), // try-catch
			EVal::Throw(Vec<String>,String,Option<Expr>)
		}
	}
}

fn set_last_mov(cmds : &mut Vec<Cmd>, dst : Reg) {
	// STRIPPING MOVS
	loop {
		if cmds.len() == 0 {
			panic!()
		}
		let i = cmds.len() - 1;
		match cmds[i] {
			Reg::Mov(ref in_reg,_) => if {
				if i - 1 >= 0 {
					match cmds[i-1].get_out() {
						Some(reg) =>
							if reg == in_reg {
								cmds.pop()
							} else {
								break
							}
						_ => break
					}
				} else {
					break
				}
			},
			_ => break
		}
	}
	// SETTING OUT
	cmds[cmds.len() - 1].set_out(dst)
}
