use bytecode::registers::*;
use std::fmt::*;
use syn::utils::Show;

pub enum Cmd {
    //  from, to
    Mov(Reg,Reg),
    IOp(Box<Opr>), // int operation
    ROp(Box<Opr>), // real operation
    VOp(Box<Opr>), // oper for object. type: (Obj,Obj) -> int
    Call(Box<Call>),
    SetI(isize,Reg),
    SetR(f64,Reg),
    SetS(String,Reg),
    WithItem(Box<WithItem>),
    //       obj meth dst
    MethMake(Reg,Reg,Reg), // it works like make-clos
    MethCall(Box<Call>, Reg), // call.func - ptr to self. Reg - register with func
    MakeClos(Box<MakeClos>),
    //   obj  ind  dst
    Prop(Reg,usize,Reg),
    //      obj  ind  val
    SetProp(Reg,usize,Reg),
    //ObjToObj(),
    Conv(Reg,Convert,Reg),
    //NewCls(Box<NewCls>),
    NewObj(usize,usize,Reg), // NewObj(prop_count, virt_count, out)

    //   (err-code, value, goto-to-this-label)
    Throw(usize,Option<Reg>,String),
    Ret(Reg),
    Goto(String), // used by break, loops, try-catch
    If(Reg,Vec<Cmd>),//,Vec<Cmd>),
    Else(Vec<Cmd>),
    ReRaise, // if exception can't be catched. making reraise and return from function

    // NOT EXECUTABLE
    Noop,
    Label(String), // for goto
    Catch(Vec<Catch>,String) // translated to switch(ex_type){case ...}. Second field is link to next 'catch' if all this was failed
}

impl Cmd {
    pub fn is_mov(&self) -> bool {
        match *self {
            Cmd::Mov(_,_) => true,
            _ => false
        }
    }
    // WARNING can panic
    pub fn mov_in(&self) -> &Reg {
        match *self {
            Cmd::Mov(ref a, _) => a,
            _ => panic!()
        }
    }
    // WARNING can panic
    pub fn mov_out(&self) -> &Reg {
        match *self {
            Cmd::Mov(_, ref a) => a,
            _ => panic!()
        }
    }
    pub fn reg_in_use(&self, reg : &Reg/*store : &mut Vec<*const Reg>*/) -> bool {
        //store.clear();
        //macro_rules! add {($e:expr) => {store.push(&$e)}; }
        macro_rules! add {($e:expr) => {if $e == *reg {return true}}; }
        match *self {
            Cmd::NewObj(_,_,ref a) => add!(*a),
            Cmd::Mov(ref a, _) => add!(*a),
            Cmd::IOp(ref opr) => {
                add!(opr.a);
                add!(opr.b);
            },
            Cmd::ROp(ref opr) => {
                add!(opr.a);
                add!(opr.b);
            },
            Cmd::VOp(ref opr) => {
                add!(opr.a);
                add!(opr.b);
            },
            Cmd::Call(ref cal) => {
                add!(cal.func);
                for a in cal.args.iter() {
                    add!(*a);
                }
            },
            Cmd::WithItem(ref obj) => {
                add!(obj.container);
                add!(obj.index);
                if obj.is_get {
                    add!(obj.value);
                }
            },
            Cmd::MethMake(ref obj, ref m, _) => {
                add!(*obj);
                add!(*m);
            },
            Cmd::MethCall(ref cal, ref r) => {
                add!(cal.func);
                add!(*r);
                for a in cal.args.iter() {
                    add!(*a);
                }
            },
            Cmd::MakeClos(ref cls) =>
                for r in cls.to_env.iter() {
                    add!(*r);
                },
            Cmd::Prop(ref obj, _, _) => add!(*obj),
            Cmd::SetProp(ref obj, _, ref val) => {
                add!(*obj);
                add!(*val)
            },
            Cmd::Conv(ref a, _, _) => add!(*a),
            Cmd::Ret(ref val) => add!(*val),
            _ => ()
        }
        return false;
    }
    pub fn get_in(&self) -> Option<&Reg> {
        match *self {
            Cmd::Mov(ref a, _) => Some(a),
            //Cmd::IOp(Box<Opr>),
            //Cmd::ROp(Box<Opr>), // real operation
            //Cmd::VOp(Box<Opr>), // oper for object
            Cmd::Call(ref c) => Some(&c.func),
            Cmd::WithItem(ref i) =>
                if !i.is_get {
                    Some(&i.value)
                } else {
                    None
                },
            Cmd::MethMake(_, ref a, _) => Some(a),
            Cmd::MethCall(_, ref a) => Some(a),
            Cmd::Prop(ref a, _, _) => Some(a),
            Cmd::SetProp(_, _, ref a) => Some(a),
            Cmd::Conv(ref a,_,_) => Some(a),
            Cmd::Throw(_, ref a, _) =>
                match *a {
                    Some(ref r) => Some(r),
                    _ => None
                },
            Cmd::Ret(ref a) => Some(a),
            Cmd::If(ref a, _) => Some(a),
            _ => None
        }
    }
    pub fn set_in(&mut self, val : Reg) {
        match *self {
            Cmd::Mov(ref mut a, _) => *a = val,
            Cmd::Call(ref mut c) => c.func = val,
            Cmd::WithItem(ref mut i) =>
                if !i.is_get {
                    i.value = val
                } else {
                    panic!("CMD has no in-slot")
                },
            Cmd::MethMake(_, ref mut a, _) => *a = val,
            Cmd::MethCall(_, ref mut a) => *a = val,
            Cmd::Prop(ref mut a, _, _) => *a = val,
            Cmd::SetProp(_, _, ref mut a) => *a = val,
            Cmd::Conv(ref mut a,_,_) => *a = val,
            Cmd::Throw(_, ref mut a, _) =>
                match *a {
                    Some(ref mut r) => *r = val,
                    _ => panic!("CMD has no in-slot")
                },
            Cmd::Ret(ref mut a) => *a = val,
            Cmd::If(ref mut a, _) => *a = val,
            _ => panic!("CMD has no in-slot")
        }
    }
    pub fn get_out(&self) -> Option<&Reg> {
        match *self {
            Cmd::NewObj(_,_,ref a) => Some(a),
            Cmd::Mov(_,ref a) => Some(a),
            Cmd::IOp(ref o) => Some(&o.dst),
            Cmd::ROp(ref o) => Some(&o.dst),
            Cmd::VOp(ref o) => Some(&o.dst),
            Cmd::Call(ref c) => Some(&c.dst),
            Cmd::SetI(_, ref a) => Some(a),
            Cmd::SetR(_, ref a) => Some(a),
            Cmd::SetS(_, ref a) => Some(a),
            Cmd::WithItem(ref i) =>
                if i.is_get {
                    Some(&i.value)
                } else {
                    None
                },
            Cmd::MethMake(_,_,ref a) => Some(a),
            Cmd::MethCall(ref a, _) => Some(&a.dst),
            Cmd::MakeClos(ref c) => Some(&c.dst),
            Cmd::Prop(_,_,ref a) => Some(a),
            Cmd::Conv(_,_,ref a) => Some(a),
            _ => None
        }
    }
    pub fn set_out(&mut self, out : Reg) {
        match *self {
            Cmd::NewObj(_,_,ref mut a) => *a = out,
            Cmd::Mov(_,ref mut a) => *a = out,
            Cmd::IOp(ref mut o) => o.dst = out,
            Cmd::ROp(ref mut o) => o.dst = out,
            Cmd::VOp(ref mut o) => o.dst = out,
            Cmd::Call(ref mut c) => c.dst = out,
            Cmd::SetI(_, ref mut a) => *a = out,
            Cmd::SetR(_, ref mut a) => *a = out,
            Cmd::SetS(_, ref mut a) => *a = out,
            Cmd::WithItem(ref mut i) =>
                if i.is_get {
                    i.value = out
                } else {
                    panic!("CMD has no out-slot")
                },
            Cmd::MethMake(_,_,ref mut a) => *a = out,
            Cmd::MethCall(ref mut a, _) => a.dst = out,
            Cmd::MakeClos(ref mut c) => c.dst = out,
            Cmd::Prop(_,_,ref mut a) => *a = out,
            Cmd::Conv(_,_,ref mut a) => *a = out,
            _ => panic!("CMD has no out-slot")
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
            Cmd::SetR(ref n, ref r) => vec![format!("{}SET REL {} => {:?}", tab, n, r)],
            Cmd::SetS(ref n, ref r) => vec![format!("{}SET STR {:?} => {:?}", tab, n, r)],
            Cmd::WithItem(ref obj) =>
                if obj.is_get {
                    vec![format!("{}GET ITEM<{:?}> {:?} [{:?}] => {:?}", tab, obj.cont_type, obj.container, obj.index, obj.value)]
                } else {
                    vec![format!("{}SET ITEM<{:?}> {:?} [{:?}] <= {:?}", tab, obj.cont_type, obj.container, obj.index, obj.value)] 
                },
            Cmd::MethMake(ref obj, ref name, ref dst) =>
                vec![format!("{}MAKE_M {:?} self:{:?} => {:?}", tab, name, obj, dst)],
            Cmd::MethCall(ref cal, ref meth) => {
                let ctch = match cal.catch_block {
                    Some(ref c) => c.clone(),
                    _ => "_".to_string()
                };
                vec![format!("{}CALL_M {:?} [catch:{}] self:{:?} {:?} => {:?}", tab, meth, ctch, cal.func, cal.args, cal.dst)]
            },
            Cmd::MakeClos(ref cls) => vec![format!("{}{:?}", tab, **cls)],
            Cmd::Prop(ref obj, ref n, ref dst) => vec![format!("{}PROP {:?} [{}] => {:?}", tab, obj, n, dst)],
            Cmd::SetProp(ref obj, ref n, ref val) =>
                vec![(format!("{}SET PROP {:?} [{}] <= {:?}", tab, obj, n, val))],
            Cmd::Conv(ref a, ref cnv, ref dst) => vec![format!("{}CONV {:?} : {:?} => {:?}", tab, a, cnv, dst)],
            //Cmd::NewCls(ref cls) => vec![format!("{}{:?}", tab, cls)],
            Cmd::NewObj(ref cnt, ref virt, ref out) =>
                vec![format!("{}NEW OBJ {} {} => {:?}", tab, cnt, virt, out)],
            Cmd::Throw(ref n, ref v, ref lab) => vec![format!("{}THROW {:?} {:?} '{}'", tab, n, v, lab)],
            Cmd::Ret(ref val) => vec![format!("{}RETURN {:?}", tab, val)],
            Cmd::Goto(ref lab) => vec![format!("{}GOTO {}", tab, lab)],
            Cmd::Label(ref lab) => vec![format!("{}LABEL {}", tab, lab)], 
            Cmd::If(ref cnd, ref good/*, ref bad*/) => {
                let mut acc = vec![format!("{}IF {:?}", tab, cnd)];
                for cmd in good.iter() {
                    for val in cmd.show(layer + 1) {
                        acc.push(val);
                    }
                }
                /*acc.push(format!("{}ELSE", tab));
                for cmd in bad.iter() {
                    for val in cmd.show(layer + 1) {
                        acc.push(val);
                    }
                }*/
                acc.push(format!("{}ENDIF", tab));
                acc
            },
            Cmd::Else(ref v) => {
                let mut acc = vec![format!("{}ELSE", tab)];
                for cmd in v.iter() {
                    for val in cmd.show(layer + 1) {
                        acc.push(val);
                    }
                }
                acc.push(format!("{}ENDELSE", tab));
                acc
            },
            Cmd::Noop => vec![format!("{}NOOP", tab)],
            Cmd::ReRaise => vec![format!("{}RERAISE", tab)],
            Cmd::Catch(ref lst, ref next) => {
                let mut acc = vec![format!("{}CATCH next:'{}'", tab, next)];
                for ctch in lst.iter() {
                    acc.push(format!("{}CASE {:?}", tab, ctch.key));
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
//    pub can_throw   : bool,
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
    pub key  : Option<usize>,
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
        write!(f, "CALL[CATCH: {}]: {:?} {:?} => {:?}", catch, self.func, self.args, self.dst)
    }
}
