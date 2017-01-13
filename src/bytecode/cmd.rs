use bytecode::registers::*;
use std::fmt::*;
use syn::utils::Show;

pub enum Cmd {
	//  from, to
	Mov(Reg,Reg),
	IOp(Box<Opr>), // int operation
	ROp(Box<Opr>), // real operation
	VOp(Box<Opr>), // oper for object
	Call(Box<Call>),
	SetI(isize,Reg),
	SetR(f64,Reg),
	SetS(String,Reg),
	WithItem(Box<WithItem>),
	//   obj  name  dst
	Meth(Reg,String,Reg), // it works like make-clos
	MakeClos(Box<MakeClos>),
	//   obj  ind  dst
	Prop(Reg,usize,Reg),
	//ObjToObj(),
	Conv(Reg,Convert,Reg),
	//NewCls(Box<NewCls>),

	Throw(usize,Option<Reg>), // try optimize it: if catch in this function, just use simple goto
	Ret(Reg),
	Goto(String), // used by break, loops, try-catch
	If(Reg,Vec<Cmd>,Vec<Cmd>),
	ReRaise, // if exception can't be catched. making reraise and return from function

	// NOT EXECUTABLE
	Noop,
	Label(String), // for goto
	Catch(Vec<Catch>,String) // translated to switch(ex_type){case ...}. Second field is link to next 'catch' if all this was failed
}

impl Cmd {
	pub fn get_out(&self) -> Option<&Reg> {
		match *self {		
			Reg::Mov(_,ref a) => Some(a),
			Reg::IOp(ref o) => Some(&o.dst),
			Reg::ROp(ref o) => Some(&o.dst),
			Reg::VOp(ref o) => Some(&o.dst),
			Reg::Call(ref c) => Some(&c.dst),
			Reg::SetI(_, ref a) => Some(a),
			Reg::SetR(_, ref a) => Some(a),
			Reg::SetS(_, ref a) => Some(a),
			Reg::WithItem(ref i) =>
				if i.is_get {
					Some(&i.value)
				} else {
					None
				},
			Reg::Meth(_,_,ref a) => Some(a),
			Reg::MakeClos(ref c) => Some(&c.dst),
			Reg::Prop(_,_,ref a) => Some(a),
			Reg::Conv(_,_,ref a) => Some(a),
			_ => None
		}
	}
	pub fn set_out(&mut self, out : Reg) {
		match *self {		
			Reg::Mov(_,ref mut a) => *a = out,
			Reg::IOp(ref mut o) => o.dst = out,
			Reg::ROp(ref mut o) => o.dst = out,
			Reg::VOp(ref mut o) => o.dst = out,
			Reg::Call(ref mut c) => c.dst = out,
			Reg::SetI(_, ref mut a) => *a = out,
			Reg::SetR(_, ref mut a) => *a = out,
			Reg::SetS(_, ref mut a) => *a = out,
			Reg::WithItem(ref i) =>
				if i.is_get {
					i.value = out
				},
			Reg::Meth(_,_,ref a) => *a = out,
			Reg::MakeClos(ref c) => c.dst = out,
			Reg::Prop(_,_,ref a) => *a = out,
			Reg::Conv(_,_,ref a) => *a = out,
			_ => ()
		}
	}
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
			Cmd::VOp(ref opr) => vec![format!("{}object {:?}", tab, opr)], // object operation
			Cmd::Call(ref cal) => vec![format!("{}{:?}", tab, **cal)],
			Cmd::SetI(ref n, ref r) => vec![format!("{}SET INT {} => {:?}", tab, n, r)],
			Cmd::SetR(ref n, ref r) => vec![format!("{}SER REL {} => {:?}", tab, n, r)],
			Cmd::SetS(ref n, ref r) => vec![format!("{}SER STR {} => {:?}", tab, n, r)],
			Cmd::WithItem(ref obj) =>
				if obj.is_get {
					vec![format!("{}GET ITEM<{:?}> {:?} [{:?}] => {:?}", tab, obj.cont_type, obj.container, obj.index, obj.value)]
				} else {
					vec![format!("{}SET ITEM<{:?}> {:?} [{:?}] <= {:?}", tab, obj.cont_type, obj.container, obj.index, obj.value)] 
				},
			Cmd::Meth(ref obj, ref name, ref dst) => vec![format!("{}METHOD {} (self:{:?}) => {:?}", tab, name, obj, dst)],
			Cmd::MakeClos(ref cls) => vec![format!("{}{:?}", tab, **cls)],
			Cmd::Prop(ref obj, ref n, ref dst) => vec![format!("{}PROP {:?} [{:?}] => {:?}", tab, obj, n, dst)],
			Cmd::Conv(ref a, ref cnv, ref dst) => vec![format!("{}CONV {:?} : {:?} => {:?}", tab, a, cnv, dst)],
			//Cmd::NewCls(ref cls) => vec![format!("{}{:?}", tab, cls)],
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
			Cmd::Noop => vec![format!("{}NOOP", tab)],
			Cmd::ReRaise => vec![format!("{}RERAISE", tab)],
			Cmd::Catch(ref lst, ref next) => {
				let mut acc = vec![format!("{}CATCH next:'{}'", tab, next)];
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
	I2R,
	I2B,
	R2I
}

pub struct WithItem {
	pub container : Reg,
	pub index     : Reg,
	pub is_get    : bool, // true - get, false - set
	pub value     : Reg,  // if get then destination else source
	pub cont_type : ContType
}

#[derive(Debug,PartialEq)]
pub enum ContType {
	Vec,
	Asc,
	Str
}

// int operation
pub struct Opr {
	pub a   : Reg,
	pub b   : Reg,
	pub dst : Reg,
	pub opr : String,
	pub is_f: bool
}

pub struct Call {
	pub func        : Reg,
	pub args        : Vec<Reg>,
	pub dst         : Reg,
//	pub can_throw   : bool,
	pub catch_block : Option<String> // NONE ONLY OF CAN'T THROW
}

pub struct MakeClos {
	pub func   : String,
	pub to_env : Vec<Reg>,
	pub dst    : Reg
}

/*pub struct NewCls {
	pub cls  : usize,
	pub args : Vec<Reg>,
	pub dst  : Reg
}*/

pub struct Catch {
	pub key  : usize,
	pub code : Vec<Cmd>
}

/*impl Debug for NewCls {
	fn fmt(&self, f : &mut Formatter) -> Result {
		write!(f, "NEWCLS {:?} {:?} => {:?}", self.cls, self.args, self.dst)
	}
}*/

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
