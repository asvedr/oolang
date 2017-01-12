#[derive(Debug,PartialEq,Clone)]
pub enum Reg {
	IVar(u8),   // index of 'int' var
	RVar(u8),   // index of 'double' var
	Var(u8),    // index of 'Var' var
	IStack(u8), // stack of int
	RStack(u8), // stack of real
	VStack(u8), // stack of Var
	RSelf,      // var 'self'
	Arg(u8),    // fun args
	Env(u8),    // closure env(outer vars)
	Temp,       // SINGLE temp var
	TempI,
	TempR,
	Null,       // no value
	Name(Box<String>) // getting global symbol
}

impl Reg {
	pub fn is_int(&self) -> bool {
		match *self {
			Reg::IVar(_)|Reg::IStack(_)|Reg::TempI => true,
			_ => false
		}
	}
	pub fn is_real(&self) -> bool {
		match *self {
			Reg::RVar(_)|Reg::RStack(_)|Reg::TempR => true,
			_ => false
		}
	}
	pub fn is_stack(&self) -> bool {
		match *self {
			Reg::IStack(_)|Reg::RStack(_)|Reg::VStack(_) => true,
			_ => false
		}
	}
	pub fn is_var(&self) -> bool {
		match *self {
			Reg::IVar(_)|Reg::RVar(_)|Reg::Var(_)|Reg::Arg(_)|Reg::Env(_) => true,
			_ => false
		}
	}
}
