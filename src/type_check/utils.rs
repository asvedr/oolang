use syn::*;
use std::collections::BTreeMap;
use std::rc::Rc;
pub use type_check::loc_env::*;
pub use type_check::fun_env::CheckRes;
pub use type_check::fun_env::CheckAns;

#[macro_export]
macro_rules! throw {
    ($mess:expr, $curs:expr) => {syn_throw!($mess, $curs)};
}

#[macro_export]
macro_rules! ok {() => {return Ok(())};}

pub type VMap = BTreeMap<String, Result<RType, *mut RType>>;
// Ok  (WE TRULY KNOW WHAT IT IS)
// Err (WE CALCULATED THIS AND WE CAN MISTAKE)

#[macro_export]
macro_rules! add_loc_unk {
    ($loc_e:expr, $name:expr, $tp:expr, $pos:expr) => { try!($loc_e.add_loc_var($name, Err($tp), &$pos)) };
}
#[macro_export]
macro_rules! add_loc_knw {
    ($loc_e:expr, $name:expr, $tp:expr, $pos:expr) => { try!($loc_e.add_loc_var($name, Ok($tp), &$pos)) };
}

pub fn find_unknown(body : &Vec<ActF>) -> &Cursor {    
    macro_rules! go_e {($e:expr) => {match check($e) {Some(p) => return Some(p) , _ => ()}};}
    macro_rules! go_a {($e:expr) => {match rec($e) {Some(p) => return Some(p) , _ => ()}};}
    fn check(e : &Expr) -> Option<&Cursor> {    
        if e.kind.is_unk() {
            Some(&e.addres)
        } else {
            match e.val {
                EVal::Call(_, ref f, ref a, _) => {
                    go_e!(f);
                    for i in a.iter() {
                        go_e!(i);
                    }
                },
                EVal::NewClass(_,_,_,ref args) => {
                    for a in args.iter() {
                        go_e!(a);
                    }
                },
                EVal::Item(ref a, ref b) => {
                    go_e!(a);
                    go_e!(b);
                },
                EVal::Arr(ref items) =>
                    for i in items {
                        go_e!(i);
                    },
                EVal::Asc(ref pairs) => {
                    for pair in pairs {
                        go_e!(&pair.a);
                        go_e!(&pair.b);
                    }
                },
                EVal::Attr(ref a, _, _) => go_e!(a),
                EVal::ChangeType(ref a, _) => go_e!(a),
                _ => ()
            }
            None
        }
    }
    fn rec(body : &Vec<ActF>) -> Option<&Cursor> {
        for act in body.iter() {
            match act.val {
                ActVal::Expr(ref e) => go_e!(e),
                ActVal::DFun(ref dfun) => go_a!(&dfun.body),
                ActVal::DVar(_,_,ref oe) => for e in oe.iter() { go_e!(e) },
                ActVal::Asg(ref a, ref b) => {
                    go_e!(a);
                    go_e!(b);
                },
                ActVal::Ret(ref oe) => for e in oe.iter() { go_e!(e) },
                ActVal::While(_, ref e, ref a) => {
                    go_e!(e);
                    go_a!(a);
                },
                ActVal::For(_,_,ref e1,ref e2,ref a) => {
                    go_e!(e1);
                    go_e!(e2);
                    go_a!(a);
                },
                ActVal::Foreach(_,_,ref t,ref e,ref a) => {
                    if t.is_unk() {
                        return Some(&act.addres);
                    } else {
                        go_e!(e);
                        go_a!(a);
                    }
                },
                ActVal::If(ref e, ref a, ref b) => {
                    go_e!(e);
                    go_a!(a);
                    go_a!(b);
                },
                ActVal::Try(ref a, ref ctchs) => {
                    go_a!(a);
                    for c in ctchs.iter() {
                        go_a!(&c.act);
                    }
                },
                ActVal::Throw(_, _, ref e) =>
                    match *e {
                        Some(ref e) => go_e!(e),
                        _ => ()
                    },
                _ => ()
            }
        }
        None
    }
    match rec(body) {
        Some(a) => a,
        _ => panic!()
    }
}

// pub empty parent constructor
pub fn put_inherit_init(seq : &mut Vec<ActF>, pos : Cursor) {
    let mut new_seq = Vec::new();
    new_seq.reserve(seq.len() + 1);
    let p1 = pos.clone();
    let p2 = p1.clone();
    new_seq.push(Act{
        val    : ActVal::Expr(Expr{
            val     : EVal::Call(
                None,
                Box::new(Expr {
                    val     : EVal::Var(vec!["%parent".to_string()], "%init".to_string()),
                    kind    : Type::unk(),
                    addres  : p2,
                    op_flag : 0
                }),
                vec![],
                false
            ),
            kind    : Type::void(),
            addres  : pos,
            op_flag : 0
        }),
        addres : p1
    });
    new_seq.append(seq);
    *seq = new_seq;
}

pub fn replace_inherit_init(seq : &mut Vec<ActF>) -> bool {
    if seq.len() > 0 {
        match seq[0].val {
            ActVal::Expr(ref mut e) =>
                match e.val {
                    EVal::Call(_, ref mut fnc, _, _) => {
                        let change = match fnc.val {
                            EVal::Var(_, ref name) => {
                                if *name == "init_parent" {
                                    true
                                } else if *name == "%init" {
                                    return true;
                                } else {
                                    false
                                }
                            },
                            _ => false
                        };
                        if change {
                            fnc.val = EVal::Var(vec!["%parent".to_string()], "%init".to_string());
                            return true;
                        }
                    },
                    _ => ()
                },
            _ => ()
        }
    }
    return false;
}

pub fn gen_default_init(has_parent : bool, addr : Cursor) -> Method {
    macro_rules! parent {() => {
        Act {
            val    : ActVal::Expr(Expr{
                val     : EVal::Call(
                    None,
                    Box::new(Expr {
                        val     : EVal::Var(vec!["%parent".to_string()], "%init".to_string()),
                        kind    : Type::unk(),
                        addres  : addr.clone(),
                        op_flag : 0
                    }),
                    vec![],
                    false
                ),
                kind    : Type::void(),
                addres  : addr.clone(),
                op_flag : 0
            }),
            addres : addr.clone()
        }
    };}
    let body = if has_parent {vec![parent!()]} else {vec![]};
    let t = type_fn!(vec![], Type::void());
    let f = SynFn {
        name        : "init".to_string(),
        tmpl        : vec![],
        args        : vec![],
        rettp       : Type::void(),
        body        : body,
        addr        : addr,
        can_be_clos : false,
        has_named   : false,
        ftype       : t.clone(),
        outers      : BTreeMap::new(),
        no_except   : false,
        rec_used    : false
    };
    Method {
        is_virt : false,
        func : f,
        ftype : t
    }
}
