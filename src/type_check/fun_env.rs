use syn::*;
use type_check::pack::*;
use type_check::tclass::*;
use std::collections::{HashSet, BTreeMap, BTreeSet};
use std::fmt::Write;

macro_rules! throw {
    ($mess:expr, $curs:expr) => {syn_throw!($mess, $curs)};
}

macro_rules! ok {() => {return Ok(())};}

type VMap = BTreeMap<String, MRType>;

pub type CheckRes    = Result<(),Vec<SynErr>>;
pub type CheckAns<A> = Result<A,Vec<SynErr>>;

pub struct FunEnv {
    pub global      : *const Pack,
    pub local       : VMap, 
    pub outers      : VMap,
    pub used_outers : BTreeSet<String>,
    pub templates   : HashSet<String>,      // local templates
    pub ret_type    : Option<RType>,
    pub loop_labels : Vec<*const String>,   // for 'break' cmd
    pub self_val    : Option<RType>,        // if you check method, then Class in this val. if it's global fun then None
    pub rec_name    : String,
    pub rec_used    : bool,
    pub rec_type    : RType
}

impl FunEnv {
    pub fn new(pack : *const Pack, _self : Option<RType>, rec_name : String, rec_type : RType) -> FunEnv {
        FunEnv {
            global      : pack,
            local       : BTreeMap::new(),
            outers      : BTreeMap::new(),
            templates   : HashSet::new(),
            ret_type    : None,
            loop_labels : Vec::new(),
            used_outers : BTreeSet::new(),
            self_val    : _self,
            rec_name    : rec_name,
            rec_used    : false,
            rec_type    : rec_type
        }
    }

    // getting outers for result
    // WARNING ref-cells will be lost
    pub fn outers(&self) -> Vec<(String, RType)> {
        let mut res = Vec::new();
        for (name,lnk) in self.outers.iter() {
            res.push((name.clone(), lnk.borrow().clone()));
        }
        return res;
    }

    pub fn parent_init(&self) -> RType {
        match self.self_val {
            None => panic!(),
            Some(ref tp) =>
                match **tp {
                    Type::Class(ref pref, ref name, _) => unsafe {
                        match (*self.global).get_cls_rc(pref, name){
                            Some(cls) =>
                                match cls.borrow().parent {
                                    Some(ref par) => par.class.borrow().initer.clone(),
                                    _ => panic!()
                                },
                            _ => panic!()
                        }
                    },
                    _ => panic!()
                }
        }
    }
    pub fn add_outer(&mut self, out : &FunEnv) {
        for name in out.local.keys() {
            self.outers.insert(name.clone(), out.local.get(name).unwrap().clone());
        }
        for name in out.outers.keys() {
            if !self.outers.contains_key(name) {
                self.outers.insert(name.clone(), out.outers.get(name).unwrap().clone());
            }
        }
        for name in out.templates.iter() {
            self.templates.insert(name.clone());
        }
    }
    pub fn set_ret_type(&mut self, t : RType) {
        self.ret_type = Some(t);
    }
    pub fn check_ret_type(&self, t : &RType) -> bool {
        match self.ret_type {
            Some(ref t1) => *t1 == *t,
            _ => false
        }
    }
    pub fn ret_type(&self) -> &RType {
        match self.ret_type {
            Some(ref t) => t,
            _ => panic!()
        }
    }
    pub fn show(&self) -> String {
        let mut out = String::new();
        let _ = write!(out, "LocEnv:\ntempls: [");
        for name in self.templates.iter() {
            let _ = write!(out, "{}, ", name);
        }
        let _ = write!(out, "]\nlocal: [");
        for name in self.local.keys() {
            let _ = write!(out, "{}, ", name);
        }
        let _ = write!(out, "]\nouter: [");
        for name in self.local.keys() {
            let _ = write!(out, "{}, ", name);
        }
        let _ = write!(out, "]\n");
        out

    }
    pub fn replace_unk(&self, name : &String, tp : RType) {
        match self.local.get(name) {
            Some(ref rtype) => *rtype.borrow_mut() = tp,
            _ =>
                match self.outers.get(name) {
                    Some(rtype) => *rtype.borrow_mut() = tp,
                    _ => panic!("replace_unk: var out: {}", name)
                }
        }
    }
    pub fn get_local_var(&self, name : &String) -> RType {
        match self.local.get(name) {
            Some(v) => v.borrow().clone(),
            _ if *name == self.rec_name => self.rec_type.clone(),
            _ => panic!()
        }
    }
    pub fn get_var(&self, pref : &mut Vec<String>, name : &String, tp_dst : &MRType, pos : &Cursor) -> CheckRes {
        macro_rules! LOCAL   { () => { pref.push(("%loc").to_string()) }; }
        macro_rules! REC     { () => { pref.push(("%rec").to_string()) }; }
        macro_rules! OUTER   { () => { pref.push(("%out").to_string()) }; }
        macro_rules! THISMOD { () => { pref.push(("%mod").to_string()) }; }
        macro_rules! clone_type { ($t:expr) => ($t.borrow().clone()); }
        if pref.len() == 0 {
            match self.local.get(name) {
                Some(t) => {
                    *tp_dst.borrow_mut() = clone_type!(t);
                    LOCAL!();
                    ok!()
                },
                None =>
                    if *name == self.rec_name {
                        *tp_dst.borrow_mut() = self.rec_type.clone();
                        REC!();
                        ok!();
                    } else {
                        match self.outers.get(name) {
                            Some(t) => {
                                *tp_dst.borrow_mut() = clone_type!(t);
                                OUTER!();
                                ok!()
                            },
                            None => 
                                match unsafe {(*self.global).get_fn(pref, name)} {
                                    Some(t) => {
                                        *tp_dst.borrow_mut() = t;
                                        match unsafe{(*self.global).pack_of_fn(name)} {
                                            Some(p) => {
                                                //println!("PACK FOUND {:?}", p);
                                                *pref = p
                                            },
                                            _ => THISMOD!()
                                        };
                                        ok!()
                                    },
                                    None => {
                                        throw!(format!("var {} not found", name), pos)
                                    }
                                }
                        }
                    }
            }
        } else {
            if pref[0] == "%loc" {
                *tp_dst.borrow_mut() = clone_type!(self.local.get(name).unwrap());
                ok!()
            } else if pref[0] == "%out" {
                *tp_dst.borrow_mut() = clone_type!(self.outers.get(name).unwrap());
                ok!()
            } else if pref[0] == "%mod" {
                let p = Vec::new();
                *tp_dst.borrow_mut() = unsafe{(*self.global).get_fn(&p, name).unwrap()};
                ok!()
            }
            match unsafe{(*self.global).get_fn(pref, name)} {
                Some(t) => {
                    unsafe{(*self.global).open_pref(pref)};
                    *tp_dst.borrow_mut() = t.clone();
                    ok!()
                }
                None => {
                    let mut fname = String::new();
                    for p in pref.iter() {
                        fname.push_str(&*p);
                        fname.push_str("::");
                    }
                    fname.push_str(name);
                    throw!(format!("var {} not found", fname), pos)
                }
            }
        }
    }
    pub fn check_exception(&self, pref : &mut Vec<String>, name : &String, pos : &Cursor) -> CheckAns<Option<RType>> {
        unsafe {
            if pref.len() == 0 {
                match (*self.global).exceptions.get(name) {
                    Some(arg) => {
                        pref.push("%mod".to_string());
                        return Ok(arg.clone());
                    },
                    None => {
                        match (*self.global).out_exc.get(name) {
                            Some(pack) => {
                                *pref = (**pack).name.clone();
                                match (**pack).exceptions.get(name) {
                                    Some(arg) => return Ok(arg.clone()),
                                    None => panic!()
                                }
                            },
                            _ => throw!(format!("exception {} not found", name), pos)
                        }
                    }
                }
            } else {
                match (*self.global).get_exception(pref, name) {
                    None => {
                        throw!(format!("exception {:?}::{} not found", pref, name), pos)
                    },
                    Some(arg) => {
                        (*self.global).open_pref(pref);
                        return Ok(arg);
                    }
                }
            }
        }
    }
    pub fn check_class(
        &self,
        pref : &Vec<String>,
        name : &String,
        params : &Option<Vec<RType>>,
        pos : &Cursor
    ) -> CheckAns<Option<Vec<String>>> {
        // RESULT IS NONE IF PREF AND NAME NOT CHANGED
        let mut new_pref = Vec::new();
        if pref.len() == 0 {
            // PREFIX NOT EXIST OR IT'S A TEMPLATE TYPE
            if self.templates.contains(name) {
            // IT'S TEMPLATE
                new_pref.push("%tmpl".to_string())
            } else {
            // IT'S IN IMPORTED SPACE
                unsafe {
                    match (*self.global).get_cls(pref, name) {
                        Some(cls) => {
                            let pcnt = match *params {Some(ref vec) => vec.len(), _ => 0};
                            if (*cls).params.len() != pcnt {
                                throw!(format!("class {:?}{} need {} params, given {}", pref, name, (*cls).params.len(), pcnt), pos)
                            }
                        },
                        None => {
                            println!("!1");
                            throw!(format!("class {} not found", name), pos)
                        }
                    }
                    match (*self.global).pack_of_cls(name) {
                        None => new_pref.push("%mod".to_string()),
                        Some(path) => new_pref = path
                    }
                }
            }
        } else {
        // IT'S IN AVAILABLE MODULES
            unsafe {
                match (*self.global).get_cls(pref, name) {
                    None => {
                        throw!(format!("class {} not found", name), pos)
                    },
                    Some(cls) => {
                        (*self.global).open_pref(pref);
                        let pcnt = match *params {Some(ref vec) => vec.len(), _ => 0};
                        if (*cls).params.len() != pcnt {
                            throw!(format!("class {:?}{} need {} params, given {}", pref, name, (*cls).params.len(), pcnt), pos)
                        }
                    }
                }
            }
        }
        if new_pref.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(new_pref))
        }
    }
    /*pub fn get_class(&self, pref : &Vec<String>, name : &String) -> *const TClass {
        unsafe {
            if pref.len() == 0 || pref[0] == "%mod" {
                match (*self.global).get_cls(None, name) {
                    Some(cls) => cls,
                    _ => panic!()
                }
            } else {
                match (*self.global).get_cls(Some(pref), cname) {
                    Some(cls) => cls,
                    _ => panic!()
                }
            }
        }
    }*/
    // return Option<(methodType, isMethod)>
    pub fn get_cls(&self, cls : &Type) -> Option<*const TClass> {
        unsafe {
            match *cls {
                Type::Class(ref pref, ref cname, _) => {
                    if pref.len() == 0 || pref[0] == "%mod" {
                        // PREFIX NOT EXIST OR IT'S A TEMPLATE TYPE
                        if self.templates.contains(cname) {
                            None
                        } else {
                        // IT'S IN IMPORTED SPACE
                            let p = Vec::new();
                            (*self.global).get_cls(&p, cname)
                        }
                    } else {
                        // IT'S IN AVAILABLE MODULES
                        (*self.global).get_cls(pref, cname)
                    }
                },
                Type::Arr(_) => {
                    let cname = "%arr".to_string();
                    let p = Vec::new();
                    (*self.global).get_cls(&p, &cname)
                },
                Type::Str => {
                    let cname = "%str".to_string();
                    let p = Vec::new();
                    (*self.global).get_cls(&p, &cname)
                },
                _ => return None
            }
        }
    }
    pub fn get_attrib(&self, cls : &RType, mname : &String, priv_too : bool) -> Option<(RType,bool)> {
        unsafe {
            match **cls {
                Type::Class(ref pref, ref cname, ref params) => {
                    let cls : *const TClass =
                        if pref.len() == 0 || pref[0] == "%mod" {
                            // PREFIX NOT EXIST OR IT'S A TEMPLATE TYPE
                            if self.templates.contains(cname) {
                                return None
                            } else {
                            // IT'S IN IMPORTED SPACE
                                let p = Vec::new();
                                match (*self.global).get_cls(&p, cname) {
                                    Some(cls) => cls,
                                    None => return None
                                }
                            }
                        } else {
                            // IT'S IN AVAILABLE MODULES
                            match (*self.global).get_cls(pref, cname) {
                                None => return None,
                                Some(cls) => cls
                            }
                        };
                    let params = match *params {
                        None => None,
                        Some(ref p) => Some(p)
                    };
                    let m = if priv_too { (*cls).look_in_all(mname, params) } else { (*cls).look_in_pub(mname, params) };
                    match m {
                        Some(res) => {
                            let flag = (*cls).is_method(mname);
                            return Some( (res, flag) )
                        },
                        None => return None
                    }
                },
                Type::Arr(ref params) => {
                    let cname = "%arr".to_string();
                    let p = Vec::new();
                    let cls = match (*self.global).get_cls(&p, &cname) { Some(c) => c, _ => panic!() };
                    let m = if priv_too { (*cls).look_in_all(mname, Some(params)) } else { (*cls).look_in_pub(mname, Some(params)) };
                    match m {
                        Some(res) => {
                            let flag = (*cls).is_method(mname);
                            return Some( (res, flag) )
                        },
                        None => return None
                    }
                },
                Type::Str => {
                    let cname = "%str".to_string();
                    let p = Vec::new();
                    let cls = match (*self.global).get_cls(&p, &cname) { Some(c) => c, _ => panic!() };
                    let m = if priv_too { (*cls).look_in_all(mname, None) } else { (*cls).look_in_pub(mname, None) };
                    match m {
                        Some(res) => {
                            let flag = (*cls).is_method(mname);
                            return Some( (res, flag) )
                        },
                        None => return None
                    }
                },
                _ => return None
            }
        }
    }
}
