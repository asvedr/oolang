use bytecode::registers::*;
use bytecode::global_conf::*;
use bytecode::cmd::*;
use std::collections::HashMap;
use std::fmt::Write;
//use std::rc::Rc;
use syn::type_sys::*;
use syn::utils::Show;

// local environment for function
pub struct Env {
	pub out   : HashMap<String,u8>, // grabbed environment
	pub args  : HashMap<String,u8>, // fun args
	pub loc_i : HashMap<String,u8>, // local optimized
	pub loc_r : HashMap<String,u8>, // local optimized
	pub loc_v : HashMap<String,u8>, // local unoptimized
	//fargs : usize                 // count vars for fun args
}

// virtual machine state for fun-body
pub struct State<'a> {
	pub mod_name     : String,
	pub pref_for_loc : String, // prefix for local functions
	pub env          : Env,
	pub exc_off      : bool,
	pub gc           : &'a GlobalConf,
	catches          : Vec<u8>, // READONLY current stack of catch blocks
	loops            : Vec<u8>, // READONLY current stack of loops
	//pub max_i  : u8,
	//pub max_r  : u8,
	//pub max_v  : u8,
	stack_i          : u8,
	stack_r          : u8,
	stack_v          : u8,
	lambda_n         : usize,  // counter for making names for local funs
	c_counter        : u8, // id generator for catch sections
	l_counter        : u8, // id generator for loop sections
	init_name        : String // for classes
}

macro_rules! push {($_self:expr, $st:ident, $mx:ident) => {{
		$_self.$st += 1;
		//if $_self.$st > $_self.$mx {
		//	$_self.$mx = $_self.$st;
		//}
		$_self.$st - 1
}};}
macro_rules! pop{($_self:expr, $st:ident) => {{
	if $_self.$st == 0
		{ return 0; }
	$_self.$st -= 1;
	return $_self.$st + 1;
}};}

impl<'a> State<'a> {
	pub fn new(e : Env, gc : &'a GlobalConf, mod_name : String, pref_for_loc : String) -> State {
		State{
			mod_name     : mod_name,
			pref_for_loc : pref_for_loc,
			gc           : gc,
			env          : e,
			stack_i      : 0,
			stack_r      : 0,
			stack_v      : 0,
			lambda_n     : 0,
			exc_off      : false,
			catches      : vec![],
			loops        : vec![],
			l_counter    : 0,
			c_counter    : 0,
			init_name    : "init".to_string()
		}
	}
	// push_X, pop_X - change value of local stack vars
	// there are 3 stacks: INT, REAL and VAR
	pub fn push_i(&mut self) -> u8 {
		push!(self, stack_i, max_i)
	}
	pub fn push_r(&mut self) -> u8 {
		push!(self, stack_r, max_r)
	}
	pub fn push_v(&mut self) -> u8 {
		push!(self, stack_v, max_v)
	}
	pub fn pop_i(&mut self) -> u8 {
		pop!(self, stack_i)
	}
	pub fn pop_r(&mut self) -> u8 {
		pop!(self, stack_r)
	}
	pub fn pop_v(&mut self) -> u8 {
		pop!(self, stack_v)
	}
	// autouse pop_i, pop_r, pop_v
	pub fn pop_this_stack(&mut self, reg : &Reg) {
		if reg.is_stack() {
			if reg.is_int() {
				self.pop_i();
			} else if reg.is_real() {
				self.pop_r();
			} else {
				self.pop_v();
			}
		}
	}
	// autouse push_i, push_r, push_v
	pub fn push_this_stack(&mut self, tp : &Type) -> Reg {
		if tp.is_int() || tp.is_bool() || tp.is_char() {
			Reg::IStack(self.push_i())
		} else if tp.is_real() {
			Reg::RStack(self.push_r())
		} else {
			Reg::VStack(self.push_v())
		}
	}
	// autouse temp(Temp, TempI, TempR)
	pub fn this_temp(&mut self, tp : &Type) -> Reg {
		if tp.is_int() {
			Reg::TempI
		} else if tp.is_real() {
			Reg::TempR
		} else {
			Reg::Temp
		}
	}
	// clear stacks of local vars
	pub fn clear_stacks(&mut self) {
		self.stack_i = 0;
		self.stack_r = 0;
		self.stack_v = 0;
	}
	// loop labels stack
	pub fn push_loop(&mut self) -> u8 {
		self.loops.push(self.l_counter);
		self.l_counter += 1;
		self.l_counter
	}
	pub fn pop_loop(&mut self) {
		self.loops.pop();
	}
	// try-catch labels stack
	pub fn push_trycatch(&mut self) -> u8 {
		self.catches.push(self.c_counter);
		self.c_counter += 1;
		self.c_counter
	}
	pub fn pop_trycatch(&mut self) {
		self.catches.pop();
	}
	pub fn try_catch_label(&self) -> String {
		let n = self.catches[self.catches.len() - 1];
		format!("TRY_CATCH{}", n)
	}
	pub fn try_ok_label(&self) -> String {
		let n = self.catches[self.catches.len() - 1];
		format!("TRY_OK{}", n)
	}
	pub fn loop_in_label(&self) -> String {
		let n = self.loops[self.loops.len() - 1];
		format!("LOOP_BEGIN{}", n)
	}
	pub fn loop_out_label(&self) -> String {
		let n = self.loops[self.loops.len() - 1];
		format!("LOOP_END{}", n)
	}
	pub fn break_label(&self, skip : usize) -> String {
		let n = self.loops[self.loops.len() - skip];
		format!("LOOP_END{}", n)
	}
	// out in Reg::Temp
	pub fn call_method(&mut self, cname : &String, mname : &String, obj : Reg, args : Vec<Reg>, out : &mut Vec<Cmd>) {
		let mut ctch =
			if self.exc_off {
				None
			} else {
				Some(self.try_catch_label())
			};
		let cls = self.gc.get_class(cname);
		match cls.get_virt_i(mname) {
			Some(i) => {
				let tmp = Reg::VStack(self.push_v());
				self.pop_v();
				out.push(Cmd::Prop(obj.clone(), i, tmp.clone()));
				let cal = Box::new(Call {
					func        : obj,
					args        : args,
					dst         : Reg::Temp,
					catch_block : ctch
				});
				out.push(Cmd::MethCall(cal, tmp));
			},
			_ => {
				let reg = match cls.method2name(mname) {
					Some(n) => Reg::Name(Box::new(n)),
					_ => panic!()
				};
				if cls.is_method_noexc(mname) {
					ctch = None
				}
				let cal = Box::new(Call {
					func        : obj, // object
					args        : args, 
					dst         : Reg::Temp,
					catch_block : ctch
				});
				out.push(Cmd::MethCall(cal, reg));
			}
		}
	}
	// make closure from method and return register with value
	pub fn closure_method(&mut self, cname : &String, mname : &String, obj : Reg, cmds : &mut Vec<Cmd>) -> Reg {
		let cls = self.gc.get_class(cname);
		// cmds.push(Cmd::MethMake(obj, format!("{}_M_{}", cname, name), tmp.clone()));
		match cls.get_virt_i(mname) {
			Some(i) => cmds.push(Cmd::Prop(obj.clone(),i,Reg::Temp)),
			_ => {
				let name = match cls.method2name(mname) {
					Some(n) => n,
					_ => panic!()
				};
				cmds.push(Cmd::Mov(Reg::Name(Box::new(name)), Reg::Temp))
			}
		}
		let out = Reg::VStack(self.push_v());
		cmds.push(Cmd::MethMake(obj, Reg::Temp, out.clone()));
		out
	}
	pub fn property(&self, cname : &String, pname : &String) -> usize {
		match self.gc.classes.get(cname) {
			Some(tcls) => {
				match tcls.borrow().props_i.get(pname) {
					Some(i) => *i,
					_ => panic!()
				}
			},
			_ => panic!()
		}
	}
	// init object of class and return register with value
	pub fn init_class(&mut self, cname : &String, args : Vec<Reg>, cmds : &mut Vec<Cmd>) -> Reg {
		match self.gc.classes.get(cname) {
			Some(tcls) => {
				let c = tcls.borrow();
				//let out = Reg::VStack(self.push_v());
				cmds.push(Cmd::NewObj(c.prop_cnt, c.virt_cnt, Reg::Temp));
				let fname = match c.method2name(&self.init_name) {
					Some(a) => a,
					_ => panic!()
				};
				let ctch =
					if self.exc_off || c.is_method_noexc(&self.init_name) {
						None
					} else {
						Some(self.try_catch_label())
					};
				let call = Call {
					func        : Reg::Temp,//Reg::Name(Box::new(fname)),
					args        : args,
					dst         : Reg::Null,
					catch_block : ctch
				};
				cmds.push(Cmd::MethCall(Box::new(call), Reg::Name(Box::new(fname))));
				let out = Reg::VStack(self.push_v());
				cmds.push(Cmd::Mov(Reg::Temp, out.clone()));
				out
			},
			_ => panic!()
		}
	}
}

impl Env {
	// check for local and args
	pub fn get_loc_var(&self, name : &String, tp : &Type) -> Reg {
		match self.args.get(name) {
			Some(i) => Reg::Arg(*i),
			_ =>
				match *tp {
					Type::Int|Type::Char|Type::Bool => 
						Reg::IVar(*self.loc_i.get(name).unwrap()),
					Type::Real =>
						Reg::RVar(*self.loc_r.get(name).unwrap()),
					_ =>
						Reg::Var(*self.loc_v.get(name).unwrap())
				}
		}
	}
}

impl Show for Env {
	fn show(&self, _ : usize) -> Vec<String> {
		let mut res = vec![];
		macro_rules! go {($name:expr, $store:ident) => {{
			let mut line = String::new();
			let _ = write!(line, "{}: {}", $name, '{');
			for name in self.$store.keys() {
				let _ = write!(line, "{}:{}, ", name, self.$store.get(name).unwrap());
			}
			let _ = write!(line, "{}", '}');
			res.push(line);
		}};}
		go!("OUTERS", out);
		go!("ARGS", args);
		go!("LOC INT",  loc_i);
		go!("LOC REAL", loc_r);
		go!("LOC VAR",  loc_v);
		res
	}
}
