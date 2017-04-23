use bytecode::cmd::*;
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
// TODO add DECLINK to all (args and env) at (return and reraise)

pub fn to_c(cmds : &Vec<Cmd>, finalizer : &Vec<String>, out : &mut File) -> io::Result<()> {
    let mut stack = vec![CodeBlock{code : cmds, pos : 0}];
    let mut space = String::new();
    let reg_res = Reg::Res;
    space.push('\t');
    macro_rules! set_prim {($pred:ident, $constr:expr, $dst:expr, $val:expr) => {{
        if $dst.$pred() {
            write!(out, "{} = {}", reg(&$dst), $val)?
        } else {
            write!(out, "DECLINK({});\n{}{}({}, {})", reg(&$dst), space, $constr, reg(&$dst), $val)?
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
        let cmd : &Cmd = &stack[0].code[stack[0].pos];
        write!(out, "{}", space)?;
        match *cmd {
            Cmd::Mov(ref a, ref b) => set_res(b, a, out)?,
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
                        param.push_str(&*reg(a));
                    } else {
                        param.push(',');
                        param.push_str(&*reg(a));
                    }
                }
                match call.func {
                    Reg::Name(ref name) => {
                        // NO ENV. DIRECT CALLa
                        write!(out, "{}(NULL, {})", name, param)?;
                        // exist_env = false;
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
                write!(out, ";\n{}", space)?;
                match call.catch_block {
                    Some(ref label) => {
                        write!(out, "if (_reg_err_key) goto {};\n{}else ", label, space)?;
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
                if r.is_obj() {
                    write!(out, "DECVAL({});\n{}NEWINT({},{})", reg(r), space, reg(r), val)?
                } else {
                    write!(out, "{} = {}", reg(r), val)?
                }
            },
            Cmd::SetR(ref val, ref r) => {
                if r.is_obj() {
                    write!(out, "DECVAL({});\n{}NEWREAL({},{})", reg(r), space, reg(r), val)?
                } else {
                    write!(out, "{} = {}", reg(r), val)?
                }
            },
            Cmd::SetS(ref s, ref r) => {
                write!(out, "_std_str_fromRaw({}, {});\n", s, s.len())?;
                write!(out, "{}", space)?;
                set_res(r, &reg_res, out)?
            },
            Cmd::WithItem(ref opr) => {
                if opr.is_get {
                    match opr.cont_type {
                        ContType::Vec => {
                            write!(
                                out,
                                "if({} < 0 || {} >= ((Vector*)VAL({})) -> size) THROW(INDEXERR);\n",
                                get_i(&opr.index),
                                get_i(&opr.index),
                                reg(&opr.container)
                            )?;
                            let val = format!("((Vector*)VAL({})) -> data[{}]", reg(&opr.container), get_i(&opr.index));
                            if opr.value.is_int() {
                                write!(out, "{} = VINT({})", reg(&opr.value), val)?;
                            } else if opr.value.is_real() {
                                write!(out, "{} = VREAL({})", reg(&opr.value), val)?;
                            } else {
                                write!(out, "ASSIGN({}, {})", reg(&opr.value), val)?;
                            }
                        },
                        ContType::Str => {
                            write!(
                                out,
                                "if({} < 0 || {} >= ((Str*)VAL({})) -> size) THROW(INDEXERR);\n",
                                get_i(&opr.index),
                                get_i(&opr.index),
                                reg(&opr.container)
                            )?;
                            let val = format!("((Str*)VAL({})) -> data[{}]", reg(&opr.container), get_i(&opr.index));
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
                            write!(
                                out,
                                "if({} < 0 || {} >= ((Vector*)VAL({})) -> size) THROW(INDEXERR);\n",
                                get_i(&opr.index),
                                get_i(&opr.index),
                                reg(&opr.container)
                            )?;
                            let val = format!("((Vector*)VAL({})) -> data[{}]", reg(&opr.container), get_i(&opr.index));
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
                            let val = format!("((Str*)VAL({})) -> data[{}]", reg(&opr.container), get_i(&opr.index));
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
            // TODO
            Cmd::MethMake(_,_,_) => panic!(),
            Cmd::MethCall(ref call, ref meth) => {
                let mut param = String::new();
                for a in call.args.iter() {
                    if param.len() == 0 {
                        param.push_str(&*reg(a));
                    } else {
                        param.push(',');
                        param.push_str(&*reg(a));
                    }
                }
                // TODO CHECK FOR NULL IF NEED
                let self_val = format!("&{}", reg(&call.func));
                match *meth {
                    Reg::Name(ref name) => {
                        // NO ENV. DIRECT CALLa
                        write!(out, "{}({}, {})", name, self_val, param)?;
                        // exist_env = false;
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
                write!(out, ";\n{}", space)?;
                match call.catch_block {
                    Some(ref label) => {
                        write!(out, "if (_reg_err_key) goto {};\n{}else ", label, space)?;
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
            Cmd::MakeClos(ref clos_conf) => {
                write!(
                    out,
                    "ASSIGN({}, newClosure({}, {}, &closure));\n",
                    reg(&clos_conf.dst),
                    clos_conf.to_env.len(),
                    clos_conf.func
                )?;
                for i in 0 .. clos_conf.to_env.len() {
                    write!(out, "\tclosure -> env[{}] = {};\n", i, reg(&clos_conf.to_env[i]))?;
                    write!(out, "\tINCLINK({});\n", reg(&clos_conf.to_env[i]))?;
                }
            },
            Cmd::Prop(ref obj, ref ind, ref out_reg) => {
                let val = format!("((Object*)VAL({})) -> props[{}]", reg(obj), ind);
                if out_reg.is_int() {
                    write!(out, "{} = VINT({})", reg(out_reg), val)?;
                } else if out_reg.is_real() {
                    write!(out, "{} = VREAL({})", reg(out_reg), val)?;
                } else {
                    write!(out, "ASG({},{})", reg(out_reg), val)?;
                }
            },
            Cmd::SetProp(ref obj, ref ind, ref val) => {
                let dst = format!("((Object*)VAL({})) -> props[{}]", reg(obj), ind);
                if val.is_int() {
                    write!(out, "VINT({}) = {}", dst, get_i(val))?;
                } else if val.is_real() {
                    write!(out, "VREAL({}) = {}", dst, get_r(val))?;
                } else {
                    write!(out, "ASG({},{})", dst, reg(val))?;
                }
            },
            Cmd::Conv(ref src, ref kind, ref dst) =>
                match *kind {
                    Convert::I2R =>
                        if dst.is_real() {
                            write!(out, "{} = (double){}", reg(dst), get_i(src))?;
                        } else {
                            write!(out, "NEWREAL({},(double)({}))", reg(dst), get_i(src))?;
                        },
                    Convert::I2B =>
                        if dst.is_int() {
                            write!(out, "{} = ({}) != 0", reg(dst), get_i(src))?;
                        } else {
                            write!(out, "NEWINT({}, (({}) != 0))", reg(dst), get_i(src))?;
                        },
                    Convert::R2I =>
                        if dst.is_int() {
                            write!(out, "{} = (int){}", reg(dst), get_r(src))?;
                        } else {
                            write!(out, "NEWINT({},(double)({}))", reg(dst), get_r(src))?;
                        }
                },
            // TODO
            Cmd::NewObj(_,_,_) => panic!(),
            Cmd::Throw(ref code, ref arg, ref lab) =>
                match *arg {
                    Some(ref val) => {
                        write!(out, "THROWP_NORET({},{})", code, reg(val))?;
                        write!(out, ";\n{}goto {}", space, lab)?;
                    },
                    _ => {
                        write!(out, "THROW_NORET({})", code)?;
                        write!(out, ";\n{}goto {}", space, lab)?;
                    }
                },
            Cmd::Ret(ref val) =>
                match *val {
                    Reg::Null => {
                        for line in finalizer {
                            write!(out, "{};\n{}", line, space)?;
                        }
                        write!(out, "RETURNNULL")?
                    },
                    ref v => {
                        set_res(&reg_res, v, out)?;
                        write!(out, ";\n")?;
                        for line in finalizer {
                            write!(out, "{}{};\n", space, line)?;
                        }
                        write!(out, "{}RETURNJUST", space)?
                    }
                },
            Cmd::Goto(ref lab) => write!(out, "goto {}", lab)?,
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
                write!(out, "else {}", '{')?;
                stack[0].pos += 1;
                stack.push(CodeBlock {
                    code : code,
                    pos  : 0
                });
                continue
            },
            Cmd::ReRaise => {
                for line in finalizer {
                    write!(out, "{};\n{}", line, space)?;
                }
                write!(out, "return;")?
            },
            Cmd::Noop => (),
            Cmd::Label(ref lab) => write!(out, "{}:", lab)?,
            // TODO
            Cmd::Catch(_,_) => panic!()
        }
        stack[0].pos += 1;
        write!(out, ";\n")?;
    }
    Ok(())
}

fn get_i(r : &Reg) -> String {
    if r.is_int() {
        reg(r)
    } else {
        format!("VINT({})", reg(r))
    }
}

fn get_r(r : &Reg) -> String {
    if r.is_real() {
        reg(r)
    } else {
        format!("VREAL({})", reg(r))
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
            write!(out, "DECLINK({}); ", reg(dst))?;
            put!("NEWINT({}, {})")
        } else if src.is_real() {
            write!(out, "DECLINK({}); ", reg(dst))?;
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
    match *a {
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
        Reg::Name(ref s)    => (**s).clone(),
        Reg::Res            => format!("result")
    }
}
