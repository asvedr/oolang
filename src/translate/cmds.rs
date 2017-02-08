use bytecode::cmds::*;
use bytecode::registers::*;
use std::io;
use std::io::Write;
use std::fs::File;

struct CodeBlock<'a> {
    pub code   : &'a Vec<Cmd>,
    pub pos    : usize,
//    pub signal : Option<String> // WHAT WILL BE BEFORE BLOCK AFTER 
}

// TODO add compile_func
// TODO add INCLINK to all args at func begining
// TODO add DECLINK to all args end env at return

pub fn to_c(cmds : &Vec<Cmd>, out : &mut File) -> io::Result<()> {
    let mut stack = vec![CodeBlock{code : cmds, pos : 0}];
    let mut space = String::new();
    let reg_res = Reg::Res;
    space.push('\t');
    macro_rules! set_prim {($pred:ident, $constr:expr, $dst:expr, $val:expr) => {{
        if $dst.$pred() {
            write!(out, "{} = {}", reg($dst), $val)?
        } else {
            write!(out, "DECLINK({});\n{}{}({}, {})", $dst, space, $constr, $dst, $val)?
        }
    }};}
    while !stack.is_empty() {
        if stack[0].pos >= stack[0].code.len() {
            stack.pop();
            space.clear();
            for _ in 0 .. stack.len() {
                space.push('\t');
            }
            write!(out, "{}{}", space, '}')?;
            continue;
        }
        let cmd : *const Cmd = &stack[0].code[stack[0].pos];
        write!(out, "{}", space)?;
        match (*cmd) {
	        Cmd::Mov(ref a, ref b) => set(reg(b), reg(a), out),
    	    Cmd::IOp(ref opr) =>
                if opr.is_f {
                    set_prim!(is_int, "NEWINT", opr.dst, format!("{}({},{})", opr.opr, get_i(&opr.a), get_i(&opr.b)))
                } else {
                    set_prim!(is_int, "NEWINT", opr.dst, format!("{}{}{}", get_i(&opr.a), opr.opr, get_i(&opr.b)))
                },
    	    Cmd::ROp(ref opr) =>
                if opr.is_f {
                    set_prim!(is_real, "NEWREAL", opr.dst, format!("{}({},{})", opr.opr, get_r(&opr.a), get_r(&opr.b)))
                } else {
                    set_prim!(is_real, "NEWREAL", opr.dst, format!("{}{}{}", get_r(&opr.a), opr.opr, get_r(&opr.b)))
                },
        	Cmd::VOp(ref opr) =>
                if opr.is_f {
                    set_prim!(is_int, "NEWINT", opr.dst, format!("{}({},{})", opr.opr, get_i(&opr.a), get_i(&opr.b)))
                } else {
                    set_prim!(is_int, "NEWINT", opr.dst, format!("{}{}{}", get_i(&opr.a), opr.opr, get_i(&opr.b)))
                },
	        Cmd::Call(ref call) => {
                let mut param = format!("");
                // XXX ARGS MUST BE COERSED TO RIGHT REGS BEFORE TRANSLATION
                for a in call.args.iter() {
                    if param.len() == 0 {
                        write!(param, "{}", reg(a))?;
                    } else {
                        write!(param, ",{}", reg(a))?;
                    }
                }
                match call.func {
                    Reg::Name(ref name) => {
                        // NO ENV. DIRECT CALLa
                        write!(out, "{}(NULL, {})", name, param)?;
                        exist_env = false;
                    },
                    ref r => {
                        let /*(ftp,*/callm/*)*/ =
                            if call.args.len() <= 5 {
                                /*(format!("CFun{}", call.args.len()), */format!("CALL{}", call.args.len())//)
                            } else {
                                /*(format!("CFunM"), */format!("CALLM")//)
                            };
                        write!(out, "_reg_func = VAL({});\n", reg(r))?;
                        write!(out, "{}", space)?;
                        write!(out, "{}(_reg_func,{})", callm, param)?;
                    }
                }
                write!(out, ";\n{}", space);
                match call.catch_block {
                    Some(ref label) => {
                        write!(out, "if (_reg_err_key) goto {};\n{}else ", label, tab)?;
                        if !call.dst.is_null() {
                            set_res(&call.dst, &reg_res, out)?;
                        }
                    },
                    _ => {
                        if !call.dst.is_null() {
                            set_res(&call.dst, &reg_res, out)?;
                        }
                    }
                }
            },
        	Cmd::SetI(ref val, ref r) => {
                if reg.is_obj() {
                    write!(out, "DECVAL({});\n{}NEWINT({},{})", reg(r), tab, reg(r), val)?
                } else {
                    write!(out, "{} = {}", reg(r), val)?
                }
            },
	        Cmd::SetR(ref val, ref r) => {
                if reg.is_obj() {
                    write!(out, "DECVAL({});\n{}NEWREAL({},{})", reg(r), tab, reg(r), val)?
                } else {
                    write!(out, "{} = {}", reg(r), val)?
                }
            },
    	    Cmd::SetS(ref s, ref r) => {
                write!(out, "_std_str_fromRaw({}, {});\n", s, s.len())?;
                write!(out, "{}", space)?;
                set_res(r, &reg_res)?
            },
    	    Cmd::WithItem(ref opr) => {
                if opr.is_get {
                    match opr.cont_type {
                        ContType::Vec => {
                            write!(out, "if({} < 0 || {} >= ((Vector*)VAL({})) -> size) THROW(INDEXERR);\n", get_i(opr.index))?;
                            let val = format!("((Vector*)VAL({})) -> data[{}]", reg(opr.cont), get_i(opr.index))?;
                            if opr.value.is_int() {
                                write!(out, "{} = VINT({})", reg(&opr.value), val)?;
                            } else if opr.value.is_real() {
                                write!(out, "{} = VREAL({})", reg(&opr.value), val)?;
                            } else {
                                write!(out, "ASSIGN({}, {})", reg(&opr.value), val)?;
                            }
                        },
                        ContType::Str => {
                            write!(out, "if({} < 0 || {} >= ((Str*)VAL({})) -> size) THROW(INDEXERR);\n", get_i(opr.index))?;
                            let val = format!("((Str*)VAL({})) -> data[{}]", reg(opr.cont), get_i(opr.index))?;
                            if opr.value.is_int() {
                                write!(out, "{} = {}", reg(&opr.value), val)?;
                            } else {
                                write!(out, "NEWINT({},{})", reg(&opr.value), val)?;
                            }
                        },
                        ContType::Asc => panic!() // TODO
                    }
                } else {
                    match opr.cont_type {
                        ContType::Vec => {
                            write!(out, "if({} < 0 || {} >= ((Vector*)VAL({})) -> size) THROW(INDEXERR);\n", get_i(opr.index))?;
                            let val = format!("((Vector*)VAL({})) -> data[{}]", reg(opr.cont), get_i(opr.index))?;
                            if opr.value.is_int() {
                                // CAUSE WE CANT PUT INT IN VEC OF VALUE
                                write!(out, "NEWINT({},{})", val, reg(&opr.value))?;
                            } else if opr.value.is_real() {
                                write!(out, "NEWREAL({},{})", val, reg(&opr.value))?;
                            } else {
                                write!(out, "ASSIGN({}, {})", val, reg(&opr.value))?;
                            }
                        },
                        ContType::Str => {
                            let val = format!("((Str*)VAL({})) -> data[{}]", reg(opr.cont), get_i(opr.index))?;
                            if opr.value.is_int() {
                                write!(out, "{} = {}", val, reg(&opr.value))?;
                            } else {
                                write!(out, "{} = VINT({})", val, reg(&opr.value))?;
                            }
                        },
                        ContType::Asc => panic!() // TODO
                    }
                }
            },
        	MethMake(Reg,Reg,Reg),
	        Cmd::MethCall(ref call, ref meth) => {
                let mut param = format!("");
                for a in call.args.iter() {
                    if param.len() == 0 {
                        write!(param, "{}", reg(a))?;
                    } else {
                        write!(param, ",{}", reg(a))?;
                    }
                }
                // TODO CHECK FOR NULL IF NEED
                let self_val = format!("&{}", reg(call.func));
                match *meth {
                    Reg::Name(ref name) => {
                        // NO ENV. DIRECT CALLa
                        write!(out, "{}({}, {})", name, self_val, param)?;
                        exist_env = false;
                    },
                    ref r => {
                        let ftp =
                            if call.args.len() <= 5 {
                                format!("CFun{}", call.args.len())
                            } else {
                                format!("CFunM")
                            };
                        // IN REGISER FUNC MUST BE AS PRIMARY-VAL-PTR
                        //write!(out, "_reg_func = PLINK({});\n", reg(r))?;
                        //write!(out, "{}", space)?;
                        write!(out, "(({})PLINK({}))({},{})", ftp, reg(r), self_val, param)?;
                    }
                }
                write!(out, ";\n{}", space);
                match call.catch_block {
                    Some(ref label) => {
                        write!(out, "if (_reg_err_key) goto {};\n{}else ", label, tab)?;
                        if !call.dst.is_null() {
                            set_res(&call.dst, &reg_res, out)?;
                        }
                    },
                    _ => {
                        if !call.dst.is_null() {
                            set_res(&call.dst, &reg_res, out)?;
                        }
                    }
                }
            },
        	MakeClos(Box<MakeClos>),
	        Cmd::Prop(ref obj, ref ind, ref out) => {
                let val = format!("((Object*)VAL({})) -> props[{}]", reg(obj), ind);
                if out.is_int() {
                    write!(out, "{} = VINT({})", reg(out), val)?;
                } else if out.is_real() {
                    write!(out, "{} = VREAL({})", reg(out), val)?;
                } else {
                    write!(out, "ASG({},{})", reg(out), val)?;
                }
            },
    	    Cmd::SetProp(ref obj, ref ind, ref val) => {
                let dst = format!("((Object*)VAL({})) -> props[{}]", reg(obj), ind);
                if val.is_int() {
                    write!(out, "VINT({}) = {}", dst, reg(val))?;
                } else if val.is_real() {
                    write!(out, "VREAL({}) = {}", dst, reg(val))?;
                } else {
                    write!(out, "ASG({},{})", dst, reg(val))?;
                }
            },
    	    Conv(Reg,Convert,Reg),
        	NewObj(usize,usize,Reg),
	        Cmd::Throw(ref code, ref arg, ref lab) =>
                match *arg {
                    Some(ref val) => {
                        write!(out, "THROWP_NORET({},{})", code, reg(val))?;
                        write!(out, ";\n{}goto {}", space, lab)?;
                    },
                    _ => {
                        write!(out, "THROW_NORET({})", code)?;
                        write!(out, ";\n{}goto {}", space, lab);
                    }
                },
        	Cmd::Ret(ref val) =>
                match *val {
                    Some(ref v) => {
                        set_res(&reg_res, v)?;
                        write!(out, ";\n{}RETURNJUST", space)?
                    },
                    _ => write!(out, "RETURNNULL")?
                },
	        Cmd::Goto(ref lab) => write!("goto {}", lab)?,
    	    Cmd::If(ref cond, ref code) => {
                if cond.is_int() {
                    write!(out, "if({}) {}", reg(cond), '{')?;
                } else {
                    write!(out, "if(VINT({})) {}", reg(cond), '{')?;
                }
                stack[0].pos += 1;
                stack.push(CodeBlock {
                    code   : code,
                    pos    : 0
                });
                continue
            },
            Cmd::Else(ref code) => {
                write!(out, "else {")?;
                stack[0].pos += 1;
                stack.push(CodeBlock {
                    code : code,
                    pos  : 0
                });
                continue
            },
    	    Cmd::ReRaise => write!(out, "return;")?,
        	Cmd::Noop => (),
	        Cmd::Label(ref lab) => write!("{}:", lab)?,
        	Catch(Vec<Catch>,String)
        }
        stack[0].pos += 1;
        write!(out, ";\n")?;
    }
}

fn set(dst : &Reg, src : &Reg, out : &mut File) -> io::Result<()> {
    macro_rules! put {($tmpl:expr) => ( write!(out, $tmpl, reg(dst), reg(src)) );}
    if dst.is_int() {
        if src.is_int() {
            put!("{} = {}")
        } else { // INT <= VAR
            put!("{} = VINT({})")
        }
    } else if dst.is_real() {
        if src.is_real() {
            put!("{} = {}")
        } else { // REAL <= VAR
            put!("{} = VREAL({})")
        }
    } else { // VAR
        if src.is_int() {
            write!(out, "DECLINK({}); ", reg(dst));
            put!("NEWINT({}, {})")
        } else if src.is_real() {
            write!(out, "DECLINK({}); ", reg(dst));
            put!("NEWREAL({}, {})")
        } else { // VAR <= VAR
            put!("ASSIGN({}, {})")
        }
    }
}

// mov without changing ref-counter
fn set_res(dst : &Reg, src : &Reg, out : &mut File) -> io::Result<()> {
    macro_rules! put {($tmpl:expr) => ( write!(out, $tmpl, reg(dst), reg(src)) );}
    if dst.is_int() {
        if src.is_int() {
            put!("{} = {}")
        } else { // INT <= VAR
            put!("{} = VINT({})")
        }
    } else if dst.is_real() {
        if src.is_real() {
            put!("{} = {}")
        } else { // REAL <= VAR
            put!("{} = VREAL({})")
        }
    } else { // VAR
        if src.is_int() {
            put!("NEWINT({}, {})")
        } else if src.is_real() {
            put!("NEWREAL({}, {})")
        } else { // VAR <= VAR
            put!("{} = {}")
        }
    }
}

fn reg(a : &Reg) -> String {
    match a {
    	Reg::IVar(ref i)    => format!("i_var{}", i),
	    Reg::RVar(ref i)    => format!("r_var{}", i),
    	Reg::Var(ref i)     => format!("v_var{}", i),
	    Reg::IStack(ref i)  => format!("i_stack{}", i),
    	Reg::RStack(ref i)  => format!("r_stack{}", i),
	    Reg::VStack(ref i)  => format!("v_stack{}", i),
    	Reg::RSelf          => format!("(*env)"),
	    Reg::Arg(ref i)     => format!("arg{}", i),
    	Reg::Env(ref i)     => format!("env[{}]", i),
	    Reg::Temp           => format!("temp"),
    	Reg::TempI          => format!("temp_i"),
	    Reg::TempR          => format!("temp_r"),
    	Reg::Exc            => format!("_reg_result.val"),
	    Reg::Null           => String::new(),
    	Reg::Name(ref s)    => s.clone()
    }
}
