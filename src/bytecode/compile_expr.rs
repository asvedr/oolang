use bytecode::state::*;
use bytecode::registers::*;
use bytecode::cmd::*;
use syn::expr::*;
use syn::type_sys::*;
//use syn::utils::Show;

//             expr       any-stats           compiled-out             reg-with-last-value
pub fn compile(e : &Expr, state : &mut State, cmds : &mut Vec<Cmd>) -> Reg {
	match e.val {
		EVal::Int(ref v)  => {
			let reg = Reg::IStack(state.push_i());
			cmds.push(Cmd::SetI(*v as isize, reg.clone()));
			reg
		},
		EVal::Real(ref v) => {
			let reg = Reg::RStack(state.push_r());
			cmds.push(Cmd::SetR(*v, reg.clone()));
			reg
		},
		EVal::Str(ref v) => {
			let reg = Reg::VStack(state.push_v());
			cmds.push(Cmd::SetS(v.clone(), reg.clone()));
			reg
		},
		EVal::Char(ref c) => {
			let v = *c as isize;
			let reg = Reg::IStack(state.push_i());
			cmds.push(Cmd::SetI(v, reg.clone()));
			reg
		},
		EVal::Bool(ref b) => {
			let reg = Reg::IStack(state.push_i());
			cmds.push(Cmd::SetI(if *b {1} else {0}, reg.clone()));
			reg
		},
		EVal::Var(ref pref, ref var) => {
			if pref[0] == "%loc" {
				state.env.get_loc_var(var, &*e.kind)
			} else if pref[0] == "%out" {
				Reg::Env(*state.env.out.get(var).unwrap())
			} else if pref[0] == "%mod" {
				Reg::Name(Box::new(format!("{}_{}", state.mod_name, var)))
			} else if pref[0] == "%std" {
				Reg::Name(Box::new(format!("_std_{}", var)))
			} else {
				let mut r_name = String::new();
				for i in pref.iter() {
					r_name = format!("{}{}_", r_name, i);
				}
				Reg::Name(Box::new(format!("{}_{}", r_name, var)))
			}
		},
		EVal::Call(_, ref fun, ref args, ref noexc) => {
            // regular_expr USED FOR ALL VARIANTS EXCLUDE %opr
			macro_rules! regular_expr {($fun_v:expr) => {{
					let mut c_args = vec![];
					for a in args.iter() {
                        // COMPILE ARG VAL
                        let val_reg = compile(a, state, cmds);
                        // PREPARE VALUE (CAN USE ONLY VARS, NOT PRIMS)
                        if val_reg.is_int() || val_reg.is_real() {
                            let arg_reg = Reg::VStack(state.push_v());
                            cmds.push(Cmd::Mov(val_reg, arg_reg.clone()));
                            c_args.push(arg_reg);
                        } else {
						    c_args.push(val_reg);
                        }
					}
					let f;
					let mut is_attr = false;
					match $fun_v {
						Ok(v) => {
							{let _ : &Expr = v;}
							match v.val {
								EVal::Attr(_,_,ref is_meth) => is_attr = *is_meth,
								_ => ()
							}
							f = compile(v, state, cmds);
							state.pop_v();
						},
						Err(r) => f = r
					}
					for a in c_args.iter() {
						if a.is_int() {
							state.pop_i();
						} else if a.is_real() {
							state.pop_r();
						} else {
							state.pop_v();
						}
					};
					let dst = match *e.kind {
						Type::Int|Type::Char|Type::Bool => Reg::TempI,
						Type::Real => Reg::TempR,
						Type::Void => Reg::Null,
						_ => Reg::Temp
					};
					if is_attr {
						cmds.pop(); // pop MOV
						let mmake = cmds.pop().unwrap(); // pop MethMake
						let c_block;
						if *noexc || state.exc_off {
							c_block = None;
						} else {
							c_block = Some(state.try_catch_label());
							//state.no_throw = false;
						}
						match mmake {
							Cmd::MethMake(obj, mname, _) => {
								let call = Box::new(Call {
									func : obj,
									args : c_args,
									dst  : dst.clone(),
									catch_block : c_block
								});
								cmds.push(Cmd::MethCall(call, mname));
							},
							_ => panic!()
						}
					} else {
						let c_block;
						if *noexc || state.exc_off {
							c_block = None
						} else {
							c_block = Some(state.try_catch_label());
							//state.no_throw = false;
						}
						let call = Box::new(Call {
							func        : f,
							args        : c_args,
							dst         : dst.clone(),
							catch_block : c_block
						});
						cmds.push(Cmd::Call(call));
					}
					if dst == Reg::Null {
						dst
					} else {
						let res_reg;
						if dst.is_int() {
							res_reg = Reg::IStack(state.push_i());
						} else if dst.is_real() {
							res_reg = Reg::RStack(state.push_r());
						} else {
							res_reg = Reg::VStack(state.push_v());
						}
						cmds.push(Cmd::Mov(dst, res_reg.clone()));
						res_reg
					}
				}}
			}
			match fun.val {
				EVal::Var(ref pref, ref name) => {
					if pref[0] == "%loc" {
						regular_expr!(Ok(&fun))
					} else if pref[0] == "%mod" {
						let name = format!("{}_f_{}", state.mod_name, name);
						regular_expr!(Err(Reg::Name(Box::new(name))))
					} else if pref[0] == "%std" {
						let name = format!("_std_{}", name);
						regular_expr!(Err(Reg::Name(Box::new(name))))
					} else if pref[0] == "%opr" {
						let a = compile(&args[0], state, cmds);
						let b = compile(&args[1], state, cmds);
						if a.is_int() {
							state.pop_i();
						} else if a.is_real() {
							state.pop_r();
						} else {
							state.pop_v();
						}
						if b.is_int() {
							state.pop_i();
						} else if b.is_real() {
							state.pop_r();
						} else {
							state.pop_v();
						}
						match name.as_ref() {
							"+"|"-"|"*"|"/" => 
								if e.kind.is_int() {
									let out = Reg::IStack(state.push_i());
									cmds.push(Cmd::IOp(Box::new(Opr{a : a, b : b, dst : out.clone(), opr : name.clone(), is_f : false})));
									out
								} else { // real
									let out = Reg::RStack(state.push_r());
									let opr = Opr{a : a, b : b, dst : out.clone(), opr : name.clone(), is_f : false};
									cmds.push(Cmd::ROp(Box::new(opr)));
									out
								},
							"%" => {
								let out = Reg::IStack(state.push_i());
								let opr = Opr{a : a, b : b, dst : out.clone(), opr : name.clone(), is_f : false};
								cmds.push(Cmd::IOp(Box::new(opr)));
								out
							},
							"**" => {
								let out = Reg::RStack(state.push_r());
								let opr = Opr{a : a, b : b, dst : out.clone(), opr : "pow".to_string(), is_f : true};
								cmds.push(Cmd::ROp(Box::new(opr)));
								out
							},
							"<"|">"|"<="|">=" =>
								if args[0].kind.is_int() {
									let out = Reg::IStack(state.push_i());
									let opr = Opr{a : a, b : b, dst : out.clone(), opr : name.clone(), is_f : false};
									cmds.push(Cmd::IOp(Box::new(opr)));
									out
								} else { // real
									let out = Reg::RStack(state.push_r());
									let opr = Opr{a : a, b : b, dst : out.clone(), opr : name.clone(), is_f : false};
									cmds.push(Cmd::ROp(Box::new(opr)));
									out
								},
							"=="|"!=" =>
								match *args[0].kind {
									Type::Int|Type::Char|Type::Bool => {
										let out = Reg::IStack(state.push_i());
										let opr = Opr{a : a, b : b, dst : out.clone(), opr : name.clone(), is_f : false};
										cmds.push(Cmd::IOp(Box::new(opr)));
										out
									},
									Type::Real => {
										let out = Reg::IStack(state.push_i());
										let opr = Opr{a : a, b : b, dst : out.clone(), opr : name.clone(), is_f : false};
										cmds.push(Cmd::ROp(Box::new(opr)));
										out
									},
									Type::Str => {
										let out = Reg::IStack(state.push_i());
										let name = "_std_strCmp".to_string();
										let opr = Opr{a : a, b : b, dst : out.clone(), opr : name, is_f : true};
										cmds.push(Cmd::VOp(Box::new(opr)));
										out
									},
									Type::Fn(_,_,_) => {
										let out = Reg::IStack(state.push_i());
										cmds.push(Cmd::SetI(0, out.clone()));
										out
									},
									_ => {
										let out = Reg::IStack(state.push_i());
										let name = "_std_addrCmp".to_string();
										let opr = Opr{a : a, b : b, dst : out.clone(), opr : name, is_f : true};
										cmds.push(Cmd::VOp(Box::new(opr)));
										out
									}
								},
							"&&"|"||" => {
								let out = Reg::IStack(state.push_i());
								let opr = Opr{a : a, b : b, dst : out.clone(), is_f : false, opr : name.clone()};
								cmds.push(Cmd::IOp(Box::new(opr)));
								out
							},
							_ => panic!("bad %opr: {}", name)
						}
					} else {
						let mut name_res = pref[0].clone();
						for i in 1 .. pref.len() {
							name_res = format!("{}_{}", name_res, pref[i]);
						}
						name_res = format!("{}_{}", name_res, name);
						regular_expr!(Err(Reg::Name(Box::new(name_res))))
					}
				},
				_ => regular_expr!(Ok(&fun))
			}
		},
		EVal::NewClass(_, ref pref, ref name, ref args) => {
			let mut c_name = String::new();
			if pref[0] == "%std" {
				c_name.push_str("_std_");
			} else {
				for i in pref.iter() {
					c_name.push_str(&**i); //= format!("{}{}_", c_name, i);
					c_name.push('_');
				}
				c_name.push_str(&*name);
				//c_name = format!("{}_C_{}", c_name, name);
			}
			//let obj_reg = state.init_class(
			let mut c_args = vec![];
			for a in args.iter() {
				c_args.push(compile(a, state, cmds));
			}
			for r in c_args.iter() {
				if r.is_int() {
					state.pop_i();
				} else if r.is_real() {
					state.pop_r();
				} else {
					state.pop_v();
				}
			}
			state.init_class(&c_name, c_args, cmds)
		},
		EVal::Item(ref arr, ref index) => {
			let arr_c = compile(arr, state, cmds);
			let ind_c = compile(index, state, cmds);
			state.pop_v();
			state.pop_i();
			macro_rules! make_cmd{($a:expr,$i:expr,$d:expr) => {{
				let ctp = match *arr.kind {
					Type::Str    => ContType::Str,
					Type::Arr(_) => ContType::Vec,
					_ /* asc */  => ContType::Asc
				};
				WithItem{
					is_get    : true,
					container : $a,
					index     : $i,
					cont_type : ctp,
					value     : $d
				}
			}};}
			let cmd : WithItem = match *e.kind {
				Type::Int|Type::Char|Type::Bool => {
					let r = Reg::IStack(state.push_i());
					make_cmd!(arr_c, ind_c, r.clone())
				},
				Type::Real => {
					let r = Reg::RStack(state.push_r());
					make_cmd!(arr_c, ind_c, r.clone())
				},
				_ => {
					let r = Reg::VStack(state.push_v());
					make_cmd!(arr_c, ind_c, r.clone())
				}
			};
			let out = cmd.value.clone();
			cmds.push(Cmd::WithItem(Box::new(cmd)));
			out
		},
		EVal::ChangeType(ref val, ref tp) => {
			let reg = compile(val, state, cmds);
			if val.kind == *tp {
				reg
			} else {
				let out = if tp.is_int() {
					Reg::TempI
				} else if tp.is_real() {
					Reg::TempR
				} else {
					Reg::Temp
				};
				macro_rules! fun {($fname:expr) => {{
					let name = format!("_std_{}", $fname);
					let args = vec![reg];
					let c_block;
					if state.exc_off {
						c_block = None;
					} else {
						c_block = Some(state.try_catch_label());
						//state.no_throw = false;
					}
					let call = Call {
						func        : Reg::Name(Box::new(name)),
						args        : args,
						dst         : out.clone(),
						catch_block : c_block,//if state.exc_off {None} else {}
					};
					let call : Box<Call> = Box::new(call);
					cmds.push(Cmd::Call(call));
				}};}
				match *val.kind {
					Type::Int  => 
						match **tp {
							Type::Real => cmds.push(Cmd::Conv(reg, Convert::I2R, out.clone())),
							Type::Str  => fun!("int2str"),//cmds.push(Cmd::Conv(reg, Convert::ITOS, out.clone())),
							Type::Bool => cmds.push(Cmd::Conv(reg, Convert::I2B, out.clone())),
							Type::Char => return reg,
							_ => panic!()
						},
					Type::Bool =>
						match **tp {
							Type::Int => return reg,
							Type::Str => fun!("bool2str"),//cmds.push(Cmd::Conv(reg, Convert::BTOS, out.clone())),
							_ => panic!()
						},
					Type::Char =>
						match **tp {
							Type::Int => return reg,
							Type::Str => fun!("char2str"),//cmds.push(Cmd::Conv(reg, Convert::CTOS, out.clone())),
							_ => panic!()
						},
					Type::Str  =>
						match **tp {
							Type::Int  => fun!("str2int"),
							Type::Real => fun!("str2real"),
							Type::Bool => fun!("str2bool"),
							_ => panic!()
						},
					Type::Real =>
						match **tp {
							Type::Int => cmds.push(Cmd::Conv(reg, Convert::R2I, out.clone())),
							Type::Str => fun!("real2str"),
							_ => panic!()
						},
					_ => return reg
				}
				// XXX CHANGE TO STACK VAL
				out
			}
		},
		EVal::TSelf => Reg::RSelf,
		EVal::Null => Reg::Null,
		EVal::Arr(_) => panic!(),
		EVal::Asc(_) => panic!(),
		EVal::Attr(ref e, ref name, ref is_meth) => {
			// is in this branch then it's not ASSIGMENT left part and not METHOD CALL
			if *is_meth {
				let obj = compile(e, state, cmds);
				state.pop_v();
				let cname = e.kind.class_name();
				state.closure_method(&cname, name, obj, cmds);
				let out = Reg::VStack(state.push_v());
				cmds.push(Cmd::Mov(Reg::Temp, out.clone()));
				out
			} else {
				let cls = e.kind.class_name();
				let prop = state.property(&cls, name);
				let out = Reg::VStack(state.push_v());
				let obj = compile(e, state, cmds);
				cmds.push(Cmd::Prop(obj, prop, out.clone()));
				out
			}
			/*if *is_meth {
				let obj = compile(e, state, cmds);
				state.pop_this_stack(&obj);
				let cname = e.kind.class_name();
				let tmp = state.this_temp(&*e.kind);
				let out = state.push_this_stack(&*e.kind);
				cmds.push(Cmd::MethMake(obj, format!("{}_M_{}", cname, name), tmp.clone()));
				cmds.push(Cmd::Mov(tmp, out.clone()));
				out
			} else {
				panic!()
			}*/
		}
	}
}
