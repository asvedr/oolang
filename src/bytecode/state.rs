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
	//fargs : usize               // count vars for fun args
}

pub struct State {
	pub mod_name : String,
	pub env      : Env,
	//pub max_i  : u8,
	//pub max_r  : u8,
	//pub max_v  : u8,
	stack_i : u8,
	stack_r : u8,
	stack_v : u8
}

macro_rules! push {($_self:expr, $st:ident, $mx:ident) => {{
		$_self.$st += 1;
		//if $_self.$st > $_self.$mx {
		//	$_self.$mx = $_self.$st;
		//}
		$_self.$st - 1
}};}
macro_rules! pop{($_self:expr, $st:ident) => {{
	$_self.$st -= 1;
	return $_self.$st + 1;
}};}

impl State {
	pub fn new(e : Env, mod_name : String) -> State {
		State{
			mod_name : mod_name,
			env      : e,
//			max_i    : 0,
//			max_r    : 0,
//			max_v    : 0,
			stack_i  : 0,
			stack_r  : 0,
			stack_v  : 0
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
