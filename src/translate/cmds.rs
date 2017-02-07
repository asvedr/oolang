use bytecode::cmds::*;
use bytecode::registers::*;
use std::io;
use std::io::Write;
use std::fs::File;

struct CodeBlock<'a> {
    pub code   : &'a Vec<Cmd>,
    pub pos    : usize,
    pub signal : Option<String> // WHAT WILL BE BEFORE BLOCK AFTER 
}

pub fn to_c(cmds : &Vec<Cmd>, out : &mut File) -> io::Result<()> {
    let mut stack = vec![CodeBlock{code : cmds, pos : 0, signal : None}];
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
        let cmd = &stack[0].code[stack[0].pos];
        write!(out, "{}", space)?;
        match *cmd {
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
                    Reg::Name(ref name) => write!(out, "{}({})", name, param)?,
                    ref r => {
                        let ftp =
                            if call.args.len() <= 5 {
                                format!("CFun{}", call.args.len())
                            } else {
                                format!("CFunM")
                            };
                        write!(out, "(({}){})({})", ftp, reg(r), param)?
                    }
                }
                write!(out, ";\n{}", space);
                match call.catch_block {
                    Some(ref label) => {
                        write!(out, "if (_reg_err_key) goto {};\n{}else ", label, tab)?;
                        set(&call.dst, &reg_res);
                    },
                    _ => {
                        set(&call.dst, &reg_res);
                    }
                }
            },
        	Cmd::SetI(ref val, ref r) => {
                if reg.is_var() {
                    write!(out, "DECVAL({});\n{}NEWINT({},{})", reg(r), tab, reg(r), val)?
                } else {
                    write!(out, "{} = {}", reg(r), val)?
                }
            },
	        SetR(f64,Reg),
    	    SetS(String,Reg),
    	    WithItem(Box<WithItem>),
        	MethMake(Reg,Reg,Reg),
	        MethCall(Box<Call>, Reg),
        	MakeClos(Box<MakeClos>),
	        Prop(Reg,usize,Reg),
    	    SetProp(Reg,usize,Reg),
    	    Conv(Reg,Convert,Reg),
        	NewObj(usize,usize,Reg),
	        Throw(usize,Option<Reg>),
        	Ret(Reg),
	        Goto(String),
    	    If(Reg,Vec<Cmd>,Vec<Cmd>),
    	    ReRaise => write!(out, "return;")?,
        	Noop => (),
	        Label(String),
        	Catch(Vec<Catch>,String)
        }
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

fn reg(a : &Reg) -> String {
    match a {
    	Reg::IVar(ref i)    => format!("i_var{}", i),
	    Reg::RVar(ref i)    => format!("r_var{}", i),
    	Reg::Var(ref i)     => format!("v_var{}", i),
	    Reg::IStack(ref i)  => format!("i_stack{}", i),
    	Reg::RStack(ref i)  => format!("r_stack{}", i),
	    Reg::VStack(ref i)  => format!("v_stack{}", i),
    	Reg::RSelf          => format!("env[0]")
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
