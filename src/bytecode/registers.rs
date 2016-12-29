#[derive(Debug,PartialEq)]
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
	Null        // no value
}

impl Reg {
	pub fn is_int(&self) -> bool {
		match *self {
			Reg::IVar(_)|Reg::IStack(_) => true,
			_ => false
		}
	}
	pub fn is_real(&self) -> bool {
		match *self {
			Reg::RVar(_)|Reg::RStack(_) => true,
			_ => false
		}
	}
}
