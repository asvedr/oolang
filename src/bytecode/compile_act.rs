use syn::*;
use bytecode::state::*;
use bytecode::registers::*;
use bytecode::cmd::*;
//use bytecode::global_conf::*;
use bytecode::compile_expr as c_expr;

pub fn compile<'a>(acts : &'a Vec<ActF>, state : &mut State, cmds : &mut Vec<Cmd>, loc_funs : &mut Vec<&'a SynFn>) {
	for a in acts.iter() {
		//a.print();
		state.clear_stacks();
		match a.val {
			ActVal::Expr(ref e) => {
				let out = c_expr::compile(e, state, cmds);
				if out != Reg::Null {
					set_last_mov(cmds, Reg::Null)
				}
			},
			ActVal::DFun(ref df) => {
				loc_funs.push(df);
				let fname = format!("{}_L_{}", state.pref_for_loc, df.name);
				let reg = state.env.get_loc_var(&df.name, &*df.ftype);
				let mut outers = vec![];
				for (name,tp) in df.outers.iter() {
					outers.push(state.env.get_loc_var(name, tp))
				}
				let mk_clos = MakeClos {
					func   : fname,
					to_env : outers,
					dst    : reg
				};
				cmds.push(Cmd::MakeClos(Box::new(mk_clos)));
			},
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
					let reg_val = c_expr::compile(val, state, cmds);
					match var.val {
						EVal::Attr(ref obj, ref pname, _) => {
							let reg_var = c_expr::compile(var, state, cmds);
							let cname = obj.kind.class_name();
							let ind = state.property(&cname, pname);
							cmds.push(Cmd::SetProp(reg_var, ind, reg_val));
						}
						_ => panic!()
					}
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
				compile(act, state,/* gc,*/ &mut body, loc_funs);
				body.push(Cmd::Goto(state.loop_in_label()));
				let cmd = Cmd::If(res, body, vec![Cmd::Goto(state.loop_out_label())]);
				cmds.push(cmd);
				cmds.push(Cmd::Label(state.loop_out_label()));
				state.pop_loop();
			},
			ActVal::If(ref cond, ref ok, ref fail) => {
				let res = c_expr::compile(cond, state, cmds);
				state.clear_stacks();
				let mut ok_body = vec![];
				compile(ok, state,/* gc,*/ &mut ok_body, loc_funs);
				if fail.len() > 0 {
					let mut no_body = vec![];
					compile(fail, state,/* gc,*/ &mut no_body, loc_funs);
					cmds.push(Cmd::If(res, ok_body, no_body));
				} else {
					cmds.push(Cmd::If(res, ok_body, vec![]));
				}
			},
			ActVal::Try(ref body, ref ctchs) => {
				//println!("ACT BEGIN");
				state.push_trycatch();
				let ok = state.try_ok_label();
				compile(body, state,/* gc,*/ cmds, loc_funs);
				cmds.push(Cmd::Goto(ok.clone()));
				cmds.push(Cmd::Label(state.try_catch_label()));
				state.pop_trycatch();
				let mut ctchs_res = vec![];
				for c in ctchs.iter() {
					let id =
						if c.ekey.len() != 0 {
							Some(state.gc.get_exc(&c.epref, &c.ekey))
						} else {
							None
						};
					let mut code = vec![];
					match c.vname {
						Some(ref name) => code.push(Cmd::Mov(Reg::Exc,state.env.get_loc_var(name, &c.vtype))),
						_ => ()
					}
					compile(&c.act, state,/* gc,*/ &mut code, loc_funs);
					code.push(Cmd::Goto(ok.clone()));
					ctchs_res.push(Catch {
						key  : id,
						code : code
					});
				};
				cmds.push(Cmd::Catch(ctchs_res, state.try_catch_label()));
				cmds.push(Cmd::Label(ok));
				//println!("ACT END");
			}
			ActVal::For(_, _, _, _, _) => panic!(),
			//ActVal::Foreach(_, ref vname, ref vtp, ref cont, ref body) =>
			ActVal::Foreach(_, _, _, _, _) => {
				panic!()
				/*state.push_loop();
				let reg  = state.push_this_stack(vtp);
				let iter = state.push_i();
				let len  = state.push_i();
				cmds.push(Cmd::SetI(0, iter));
				let len_call = Call{
					func : Reg::Name(Box::new("_std_vec_len".to_string())),
					args : vec![
					dst  : 
					catch_block : if
				};
				state.pop_loop();*/
			},
			ActVal::Throw(ref pref, ref name, ref param) => {
				//state.no_throw = false;
				let num = state.gc.get_exc(pref, name);
				let param = match *param {
					Some(ref val) => Some(c_expr::compile(val, state, cmds)),
					_ => None
				};
                let lab = state.try_catch_label();
				cmds.push(Cmd::Throw(num, param, lab))
			}
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
	cmds[len - 1].set_out(dst);
}
