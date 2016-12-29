use bytecode::registers::*;
use std::fmt::*;
use syn::utils::Show;

pub enum Cmd {
	//  from, to
	Mov(Reg,Reg),
	IOp(Box<Opr>), // int operation
	ROp(Box<Opr>), // real operation
	Call(Box<Call>),
	SetI(isize,Reg),
	SetR(f64,Reg),
	SetS(String,Reg),
	//   arr ind dst
	ItemVec(Reg,Reg,Reg),
	ItemAsc(Reg,Reg,Reg),
	ItemStr(Reg,Reg,Reg),
	//   obj  name  dst
	Meth(Reg,String,Reg), // it works like make-clos
	MakeClos(Box<MakeClos>),
	//   obj  ind  dst
	Prop(Reg,usize,Reg),
	//ObjToObj(),
	Conv(Reg,Convert,Reg),
	NewCls(Box<NewCls>),

	Throw(usize,Option<Reg>), // try optimize it: if catch in this function, just use simple goto
	Ret(Option<Reg>),
	Goto(String), // used by break, loops, try-catch
	If(Reg,Vec<Cmd>,Vec<Cmd>),

	// NOT EXECUTABLE
	Label(String), // for goto
	Catch(Vec<Catch>) // translated to switch(ex_type){case ...}
}

impl Show for Cmd {
	fn show(&self, layer : usize) -> Vec<String> {
		//maro_rules! line {($a:expr) => vec![$a]}
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		match *self {
			Cmd::Mov(ref a, ref b) => vec![format!("{}{:?} => {:?}", tab, a, b)],
			Cmd::IOp(ref opr) => vec![format!("{}int {:?}", tab, opr)], // int operation
			Cmd::ROp(ref opr) => vec![format!("{}real {:?}", tab, opr)], // real operation
			Cmd::Call(ref cal) => vec![format!("{}{:?}", tab, **cal)],
			Cmd::SetI(ref n, ref r) => vec![format!("{}SET INT {} => {:?}", tab, n, r)],
			Cmd::SetR(ref n, ref r) => vec![format!("{}SER REL {} => {:?}", tab, n, r)],
			Cmd::SetS(ref n, ref r) => vec![format!("{}SER STR {} => {:?}", tab, n, r)],
			Cmd::ItemVec(ref ar, ref ind, ref dst) => vec![format!("{}ITEM ARR {:?} [{:?}] => {:?}", tab, ar, ind, dst)],
			Cmd::ItemAsc(ref ar, ref ind, ref dst) => vec![format!("{}ITEM ASC {:?} [{:?}] => {:?}", tab, ar, ind, dst)],
			Cmd::ItemStr(ref ar, ref ind, ref dst) => vec![format!("{}ITEM STR {:?} [{:?}] => {:?}", tab, ar, ind, dst)],
			Cmd::Meth(ref obj, ref name, ref dst) => vec![format!("{}METHOD {} (self:{:?}) => {:?}", tab, name, obj, dst)],
			Cmd::MakeClos(ref cls) => vec![format!("{}{:?}", tab, **cls)],
			Cmd::Prop(ref obj, ref n, ref dst) => vec![format!("{}PROP {:?} [{:?}] => {:?}", tab, obj, n, dst)],
			Cmd::Conv(ref a, ref cnv, ref dst) => vec![format!("{}CONV {:?} : {:?} => {:?}", tab, a, cnv, dst)],
			Cmd::NewCls(ref cls) => vec![format!("{}{:?}", tab, cls)],
			Cmd::Throw(ref n, ref v) => vec![format!("{}THROW {:?} {:?}", tab, n, v)],
			Cmd::Ret(ref val) => vec![format!("{}RETURN {:?}", tab, val)],
			Cmd::Goto(ref lab) => vec![format!("{}GOTO {}", tab, lab)],
			Cmd::Label(ref lab) => vec![format!("{}LABEL {}", tab, lab)], 
			Cmd::If(ref cnd, ref good, ref bad) => {
				let mut acc = vec![format!("{}IF {:?}", tab, cnd)];
				for cmd in good.iter() {
					for val in cmd.show(layer + 1) {
						acc.push(val);
					}
				}
				acc.push(format!("{}ELSE", tab));
				for cmd in bad.iter() {
					for val in cmd.show(layer + 1) {
						acc.push(val);
					}
				}
				acc.push(format!("{}ENDIF", tab));
				acc
			},
			Cmd::Catch(ref lst) => {
				let mut acc = vec![format!("{}CATCH", tab)];
				for ctch in lst.iter() {
					acc.push(format!("{}CASE {}", tab, ctch.key));
					for cmd in ctch.code.iter() {
						for val in cmd.show(layer + 1) {
							acc.push(val);
						}
					}
				};
				acc
			}
		}
	}
}

#[derive(Debug)]
pub enum Convert {
	I2S,
	I2R,
	I2B,

	R2I,
	R2S,

	S2R,
	S2I,
	
	B2I
}

// int operation
pub struct Opr {
	pub a   : Reg,
	pub b   : Reg,
	pub dst : Reg,
	pub opr : char
}

pub struct Call {
	pub func        : Reg,
	pub args        : Vec<Reg>,
	pub dst         : Reg,
	pub can_throw   : bool,
	pub catch_block : Option<String>
}

pub struct MakeClos {
	pub func   : String,
	pub to_env : Vec<Reg>,
	pub dst    : Reg
}

pub struct NewCls {
	pub cls  : usize,
	pub args : Vec<Reg>,
	pub dst  : Reg
}

pub struct Catch {
	pub key  : usize,
	pub code : Vec<Cmd>
}

impl Debug for NewCls {
	fn fmt(&self, f : &mut Formatter) -> Result {
		write!(f, "NEWCLS {:?} {:?} => {:?}", self.cls, self.args, self.dst)
	}
}

impl Debug for MakeClos {
	fn fmt(&self, f : &mut Formatter) -> Result {
		write!(f, "CLOS: {:?} {:?} => {:?}", self.func, self.to_env, self.dst)
	}
}

impl Debug for Opr {
	fn fmt(&self, f : &mut Formatter) -> Result {
		write!(f, "OPR: {:?} {:?} {:?}' => {:?}", self.a, self.opr, self.b, self.dst)
	}
}

impl Debug for Call {
	fn fmt(&self, f : &mut Formatter) -> Result {
		let catch = match self.catch_block {
			Some(ref t) => t.to_string(),
			_ => "_".to_string()
		};
		write!(f, "CALL[TRY:{:?} CATCH: {}]: {:?} {:?} => {:?}", self.can_throw, catch, self.func, self.args, self.dst)
	}
}
