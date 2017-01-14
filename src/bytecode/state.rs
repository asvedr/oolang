use bytecode::registers::*;
use std::collections::HashMap;
use std::fmt::Write;
use syn::type_sys::*;
use syn::utils::Show;

pub struct Env {
	pub out   : HashMap<String,u8>, // grabbed environment
	pub args  : HashMap<String,u8>, // fun args
	pub loc_i : HashMap<String,u8>, // local optimized
	pub loc_r : HashMap<String,u8>, // local optimized
	pub loc_v : HashMap<String,u8>, // local unoptimized
	//fargs : usize                 // count vars for fun args
}

pub struct ExcKeys {
	map : HashMap<String,usize>,
	cnt : usize
}

macro_rules! make_name{($pref:expr, $name:expr, $res:expr) => {{
	for i in $pref.iter() {
		$res = format!("{}{}_", $res, i);
	}
	$res = format!("{}{}_", $res, $name);
}};}

impl ExcKeys {
	pub fn get(&self, pref : &Vec<String>, name : &String) -> usize {
		let mut res = String::new();
		make_name!(pref, name, res);
		match self.map.get(&res) {
			Some(a) => *a,
			_ => panic!("bad exception key: {}", name)
		}
	}
	pub fn add(&mut self, pref : &Vec<String>, name : &String) {
		let mut res = String::new();
		make_name!(pref, name, res);
		self.map.insert(res, self.cnt);
		self.cnt += 1;
	}
	pub fn new(c : usize) -> ExcKeys {
		ExcKeys {
			cnt : c,
			map : HashMap::new()
		}
	}
}

pub struct GlobalConf {
	pub excepts : ExcKeys
}

impl GlobalConf {
	pub fn new(c : usize) -> GlobalConf {
		GlobalConf{
			excepts : ExcKeys::new(c)
		}
	}
}

pub struct State {
	pub mod_name : String,
	pub env      : Env,
	pub exc_off  : bool,
	catches   : Vec<u8>, // READONLY current stack of catch blocks
	loops     : Vec<u8>, // READONLY current stack of loops
	//pub max_i  : u8,
	//pub max_r  : u8,
	//pub max_v  : u8,
	stack_i   : u8,
	stack_r   : u8,
	stack_v   : u8,
	lambda_n  : usize,  // counter for making names for local funs
	c_counter : u8, // id generator for catch sections
	l_counter : u8, // id generator for loop sections
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

impl State {
	pub fn new(e : Env, mod_name : String) -> State {
		State{
			mod_name  : mod_name,
			env       : e,
			stack_i   : 0,
			stack_r   : 0,
			stack_v   : 0,
			lambda_n  : 0,
			exc_off   : false,
			catches   : vec![],
			loops     : vec![],
			l_counter : 0,
			c_counter : 0
		}
	}
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
	pub fn push_this_stack(&mut self, tp : &Type) -> Reg {
		if tp.is_int() || tp.is_bool() || tp.is_char() {
			Reg::IStack(self.push_i())
		} else if tp.is_real() {
			Reg::RStack(self.push_r())
		} else {
			Reg::VStack(self.push_v())
		}
	}
	pub fn this_temp(&mut self, tp : &Type) -> Reg {
		if tp.is_int() {
			Reg::TempI
		} else if tp.is_real() {
			Reg::TempR
		} else {
			Reg::Temp
		}
	}
	pub fn clear_stacks(&mut self) {
		self.stack_i = 0;
		self.stack_r = 0;
		self.stack_v = 0;
	}
	pub fn push_loop(&mut self) -> u8 {
		self.loops.push(self.l_counter);
		self.l_counter += 1;
		self.l_counter
	}
	pub fn pop_loop(&mut self) {
		self.loops.pop();
	}
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
