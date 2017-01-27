use syn::*;
use bytecode::state::*;
use bytecode::registers::*;
use bytecode::cmd::*;
//use bytecode::global_conf::*;
use bytecode::compile_expr as c_expr;

pub fn compile(acts : &Vec<ActF>, state : &mut State, /*gc : &GlobalConf,*/cmds : &mut Vec<Cmd>) {
	for a in acts.iter() {
		state.clear_stacks();
		match a.val {
			ActVal::Expr(ref e) => {
				let out = c_expr::compile(e, state, cmds);
				if out != Reg::Null {
					set_last_mov(cmds, Reg::Null)
				}
			},
			ActVal::DFun(_) => panic!(),
			ActVal::DVar(ref name, ref tp, ref val) => {
				let reg = state.env.get_loc_var(name, &**tp);
				match *val {
					None => cmds.push(Cmd::Mov(Reg::Null, reg)),
					Some(ref val) => {
						c_expr::compile(val, state, cmds);
						set_last_mov(cmds, reg);
					}
				}
			},
			ActVal::Asg(ref var, ref val) => {
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
					let len = cmds.len();
					match cmds[len - 1] {
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
			ActVal::Ret(ref e) =>
				match *e {
					Some(ref e) => {
						let out = c_expr::compile(e, state, cmds);
						cmds.push(Cmd::Ret(out))
					},
					_ => cmds.push(Cmd::Ret(Reg::Null))
				},
			ActVal::Break(_, ref n) => {
				cmds.push(Cmd::Goto(state.break_label(*n)))
			},
			ActVal::While(_, ref cond, ref act) => {
				state.push_loop();
				cmds.push(Cmd::Label(state.loop_in_label()));
				let res = c_expr::compile(cond, state, cmds);
				state.clear_stacks();
				let mut body = vec![];
				compile(act, state,/* gc,*/ &mut body);
				body.push(Cmd::Goto(state.loop_in_label()));
				let cmd = Cmd::If(res, body, vec![Cmd::Goto(state.loop_out_label())]);
				cmds.push(cmd);
				cmds.push(Cmd::Label(state.loop_out_label()));
				state.pop_loop();
			},
			//ActVal::For(Option<String>,String,Expr,Expr,Vec<Act<DF>>), // for i in range(a + 1, b - 2) {}
			//ActVal::Foreach(Option<String>,String,RType, Expr,Vec<Act<DF>>),  // for i in array {}
			ActVal::If(ref cond, ref ok, ref fail) => {
				let res = c_expr::compile(cond, state, cmds);
				state.clear_stacks();
				let mut ok_body = vec![];
				compile(ok, state,/* gc,*/ &mut ok_body);
				if fail.len() > 0 {
					let mut no_body = vec![];
					compile(fail, state,/* gc,*/ &mut no_body);
					cmds.push(Cmd::If(res, ok_body, no_body));
				} else {
					cmds.push(Cmd::If(res, ok_body, vec![]));
				}
			},
			ActVal::Try(ref body, ref ctchs) => {
				state.push_trycatch();
				let ok = state.try_ok_label();
				compile(body, state,/* gc,*/ cmds);
				cmds.push(Cmd::Goto(ok.clone()));
				cmds.push(Cmd::Label(state.try_catch_label()));
				state.pop_trycatch();
				let mut ctchs_res = vec![];
				for c in ctchs.iter() {
					let id = state.gc.excepts.get(&c.epref, &c.ekey);
					let mut code = vec![];
					match c.vname {
						Some(ref name) => code.push(Cmd::Mov(Reg::Exc,state.env.get_loc_var(name, &c.vtype))),
						_ => ()
					}
					compile(&c.act, state,/* gc,*/ &mut code);
					code.push(Cmd::Goto(ok.clone()));
					ctchs_res.push(Catch {
						key  : id,
						code : code
					});
				};
				cmds.push(Cmd::Catch(ctchs_res, state.try_catch_label()));
				cmds.push(Cmd::Label(ok));
			}
			ActVal::For(_, _, _, _, _) => panic!(),
			ActVal::Foreach(_, _, _, _, _) => panic!(),
			ActVal::Throw(_, _, _) => panic!()
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
			Cmd::Mov(ref in_reg,_) => {
				if /*i - 1 >= 0*/i >= 1 {
					match cmds[i-1].get_out() {
						Some(reg) =>
							if reg == in_reg {
								//cmds.pop();
								// NEED POP
							} else {
								break
							},
						_ => break
					}
				} else {
					break
				}
			},
			_ => break
		}
		// IF BREAK HASN'T CALLED THEN NEED POP
		cmds.pop();
	}
	// SETTING OUT
	let len = cmds.len();
	cmds[len - 1].set_out(dst)
}
