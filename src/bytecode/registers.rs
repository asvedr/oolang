#[derive(Debug,PartialEq)]
pub enum Reg {
	IVar(u8), // index of 'int' var
	RVar(u8), // index of 'double' var
	Var(u8),  // index of 'Var' var
	I1,       // int 1, result
	I2,       // int 2
	R1,       // real 1, result
	R2,       // real 2
	VT,       // temp var. 
	RSelf,    // var 'self'
	Arg(u8),  // fun args
	Env(u8)   // closure env
}
