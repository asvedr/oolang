use syn::*;
use type_check::pack::*;

pub fn recalculate(mdl : &mut SynMod, pack : &mut Pack) {
    /*
     * 1 get all local funs of funs and meths
     * 2 check all funs. If found fun that change state to NOEXCEPT then
     *   replace all call of this to NOEXCEPT in other funs body
     * 3 calling of local fun or argument or var ALWAYS need exception-safety
     */
    // using *mut here must be safe because all operation used in lifetime of module
    macro_rules! for_all {
        (mut, $name:ident, $body:block) => {
            for $name in mdl.funs.iter_mut()
                $body   
            for c in mdl.classes.iter_mut() {
                for m in c.priv_fn.iter_mut() {
                    let $name = &mut m.func;
                    $body
                }
                for m in c.pub_fn.iter_mut() {
                    let $name = &mut m.func;
                    $body
                }
            }
        };
    }
    let mut locals  : Vec<*mut SynFn> = vec![];
    unsafe {
        for_all!(mut, f, {
            get_local_funs(&mut f.body, &mut locals);
        });
    }
    // CHECK GLOBAL FUNS
    let mut replaced = true;
    let mut new_free : Vec<*const String> = vec![];
    while replaced {
        replaced = false;
        for f in mdl.funs.iter_mut() {
            if f.no_except {
                continue;
            } else {
                if !can_throw_act(&f.body) {
                    f.no_except = true;
                    new_free.push(&f.name);
                }
            }
        }
        unsafe {
            for_all!(mut, f, {
                replaced = replaced || replace_to_noex(&mut f.body, &new_free);
            });
            for link in locals.iter_mut() {
                replace_to_noex(&mut (**link).body, &new_free);
            }
        }
        // SAVE TO PACK
        unsafe {
            for name in new_free {
                pack.fns_noex.insert((*name).clone());
            }
        }
        new_free = Vec::new();
    }
    // CHECK METHODS
    for c in mdl.classes.iter_mut() {
        for m in c.priv_fn.iter_mut() {
            if !can_throw_act(&m.func.body) {
                m.func.no_except = true;
            }
        }
        for m in c.pub_fn.iter_mut() {
            if !can_throw_act(&m.func.body) {
                m.func.no_except = true;
            }
        }
    }
    // CHECK LOCALS
    unsafe {
        for f in locals {
            let noexc = !can_throw_act(&(*f).body);
            (*f).no_except = noexc;
        }
    }
}

// some ways always returning false because it won't make function NOEXCEPT
unsafe fn replace_to_noex(code : &mut Vec<ActF>, names : &Vec<*const String>) -> bool {
    let mut res = false;
    let mut force_false = false;
    macro_rules! replace_expr {($e:expr) => {res = replace_expr($e, names) || res}; }
    for act in code.iter_mut() {
        match act.val {
            ActVal::Expr(ref mut e) => replace_expr!(e),
            ActVal::DVar(_,_,ref mut oe) =>
                match *oe {
                    Some(ref mut e) => replace_expr!(e),
                    _ => ()
                },
            ActVal::Asg(ref mut a, ref mut b) => {
                replace_expr!(a);
                replace_expr!(b);
            },
            ActVal::Ret(ref mut e) =>
                match *e {
                    Some(ref mut e) => replace_expr!(e),
                    _ => ()
                },
            ActVal::While(_, ref mut e, ref mut a) => {
                replace_expr!(e);
                res = replace_to_noex(a, names) || res;
            },
            ActVal::For(_,_,ref mut a,ref mut b,ref mut c) => {
                replace_expr!(a);
                replace_expr!(b);
                res = replace_to_noex(c, names) || res;
            },
            ActVal::Foreach(_,_,_,ref mut c, ref mut a) => {
                replace_expr!(c);
                res = replace_to_noex(a, names) || res;
                force_false = true
            },
            ActVal::If(ref mut e,ref mut a,ref mut b) => {
                let e = replace_expr(e, names);
                let a = replace_to_noex(a, names);
                res = replace_to_noex(b, names) || a || e || res
            },
            ActVal::Try(ref mut t, ref mut ctch) => {
                replace_to_noex(t, names);
                for c in ctch.iter_mut() {
                    replace_to_noex(&mut c.act, names);
                }
                force_false = true;
            },
            ActVal::Throw(_,_,ref mut e) => {
                match *e {
                    Some(ref mut e) => replace_expr!(e),
                    _ => ()
                };
                force_false = true;
            }
            _ => ()
        }
    }
    if force_false {
        false
    } else {
        res
    }
}

// some ways always returning false because it won't make function NOEXCEPT
//                     tree to change      new no-exc funcs in mod       is tree changed
unsafe fn replace_expr(expr : &mut Expr, names : &Vec<*const String>) -> bool {
    match expr.val {
        EVal::Call(_,ref mut fun,ref mut args,ref mut noexc) => {
            let mut f = false;
            for a in args.iter_mut() {
                f = replace_expr(a, names) || f;
            }
            if !*noexc {
                let yes = match fun.val {
                    EVal::Var(ref pref, ref name) => {
                        if pref.len() > 0 && pref[0] == "%mod" {
                            let mut found = false;
                            for n in names.iter() {
                                if **n == *name {
                                    found = true;
                                    break
                                }
                            }
                            found
                        } else {
                            false
                        }
                    },
                    _ => false
                };
                *noexc = yes; // CHANGING
                yes
            } else {
                f
            }
        },
        EVal::NewClass(_,_,_,ref mut args) => {
            for a in args.iter_mut() {
                replace_expr(a, names);
            }
            false
        },
        EVal::Item(ref mut cont, ref mut i) => {
            let a = replace_expr(&mut **cont, names);
            replace_expr(&mut **i, names) || a
        },
        EVal::Arr(ref mut items) => {
            let mut f = false;
            for i in items.iter_mut() {
                f = replace_expr(i, names) || f;
            }
            f
        },
        EVal::Asc(ref mut pairs) => {
            let mut f = false;
            for p in pairs.iter_mut() {
                let a = replace_expr(&mut p.a, names);
                f = replace_expr(&mut p.b, names) || f || a;
            }
            f
        },
        EVal::Attr(ref mut e, _, _) => {
            replace_expr(&mut **e, names);
            false
        },
        EVal::ChangeType(ref mut e, _) => {
            replace_expr(&mut **e, names)
        },
        _ => false
    }
}

unsafe fn get_local_funs(code : &mut Vec<ActF>, result : &mut Vec<*mut SynFn>) {
    for act in code.iter_mut() {
        match act.val {
            ActVal::DFun(ref mut df) => result.push(&mut **df),
            ActVal::While(_, _, ref mut body) =>
                get_local_funs(body, result),
            ActVal::For(_,_,_,_,ref mut body) =>
                get_local_funs(body, result),
            ActVal::Foreach(_,_,_,_,ref mut body) =>
                get_local_funs(body, result),
            ActVal::If(_,ref mut t_body,ref mut e_body) => {
                get_local_funs(t_body, result);
                get_local_funs(e_body, result);
            },
            ActVal::Try(ref mut body, ref mut ctchs) => {
                get_local_funs(body, result);
                for ctch in ctchs.iter_mut() {
                    get_local_funs(&mut ctch.act, result);
                }
            },
            _ => ()
        }
    }
}

fn can_throw_act(code : &Vec<ActF>/*, store : &Vec<SynFn>*/) -> bool {
    let mut res = false;
    macro_rules! can_throw {
        ($e:expr) => {res = res || can_throw_expr($e/*, store*/)};
        ($a:expr, ACT) => {res = res || can_throw_act($a/*, store*/)};
    }
    for act in code.iter() {
        match act.val {
            ActVal::Expr(ref e) => can_throw!(e),
            ActVal::DVar(_,_,ref oe) =>
                match *oe {
                    Some(ref e) => can_throw!(e),
                    _ => ()
                },
            ActVal::Asg(ref a, ref b) => {
                can_throw!(a);
                can_throw!(b);
            },
            ActVal::Ret(ref e) =>
                match *e {
                    Some(ref e) => can_throw!(e),
                    _ => ()
                },
            ActVal::While(_, ref e, ref a) => {
                can_throw!(e);
                can_throw!(a, ACT);
            },
            ActVal::For(_,_,ref a,ref b,ref c) => {
                can_throw!(a);
                can_throw!(b);
                can_throw!(c, ACT);
            },
            ActVal::Foreach(_,_,_,_,_) => res = true,
            ActVal::If(ref e,ref a,ref b) => {
                can_throw!(e);
                can_throw!(a, ACT);
                can_throw!(b, ACT);
            },
            ActVal::Try(_, _) => res = true,
            ActVal::Throw(_,_,_) => res = true,
            _ => ()
        }
        if res {
            return true;
        }
    }
    return res;
}

fn can_throw_expr(e : &Expr/*, store : &Vec<SynFn>*/) -> bool {
    macro_rules! rec {($e:expr) => {can_throw_expr($e/*, store*/)} }
    match e.val {
        EVal::Call(_, ref fun, ref args, ref noexc) => {
            let mut has = rec!(fun);
            for a in args.iter() {
                has = has || rec!(a);
            }
            !(*noexc) || has
        },
        EVal::NewClass(_,_,_,_) => true,
        EVal::Item(ref a, ref b) => rec!(a) || rec!(b),
        EVal::Arr(ref es) => {
            for e in es.iter() {
                if rec!(e) {
                    return true;
                }
            }
            return false;
        },
        EVal::Asc(ref pairs) => {
            for pair in pairs.iter() {
                if rec!(&pair.a) || rec!(&pair.b) {
                    return true;
                }
            }
            return false;
        },
        EVal::Attr(_,_,_) => true,
        EVal::ChangeType(ref e, ref t) => {
            let ech = rec!(e);
            !safe_coerse(&*e.kind, &**t) || ech
        },
        _ => false
    }
}

fn safe_coerse(a : &Type, b : &Type) -> bool {
    macro_rules! cmp {($at:ident, $bt:ident) => {
        match *a {
            Type::$at => match *b {
                Type::$bt => true,
                _ => false
            },
            Type::$bt => match *b {
                Type::$at => true,
                _ => false
            },
            _ => false
        }
    };}
    if a == b {
        true
    } else {
        cmp!(Int, Real) || cmp!(Int, Char) || cmp!(Int, Bool)
    }
}
