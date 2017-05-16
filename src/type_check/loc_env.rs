use type_check::fun_env::*;
use type_check::pack::*;
use std::collections::BTreeMap;
use std::fmt::Write;
use syn::*;

type VMap = BTreeMap<String, MRType>;

pub struct LocEnv {
    pub fun_env   : FunEnv,
    pub block_env : Vec<VMap>,
    reduced_cells : Vec<MRType>
}

macro_rules! is_in_local {
    ($_self:expr, $name:expr) => {{
        let mut found = false;
        for env in $_self.block_env.iter().rev() {
            if env.contains_key($name) {
                found = true;
                break;
            }
        }
        found
    }}
}

impl LocEnv {
    // AT BEGINING OF FUNCTION
    pub fn new(
        pack : *const Pack,
        // ACIVE TEMPLATE
        tmpl : &Vec<String>,
        // OBJECT SELF FOR THIS CONTEXT
        _self : Option<RType>,
        // NAME FOR REC CALL
        rec_name : String,
        // TYPE OF REC FUNCTION
        rec_type : RType
    ) -> LocEnv {
        let mut env = FunEnv::new(pack, _self, rec_name, rec_type);
        for name in tmpl.iter() {
            env.templates.insert(name.clone());
        }
        LocEnv {
            fun_env       : env,
            block_env     : vec![BTreeMap::new()],
            reduced_cells : vec![]
        }
    }

    pub fn new_no_rec(pack : *const Pack, tmpl : &Vec<String>, _self : Option<RType>) -> LocEnv {
        let mut env = FunEnv::new(pack, _self, String::new(), Type::void());
        for name in tmpl.iter() {
            env.templates.insert(name.clone());
        }
        LocEnv {
            fun_env       : env,
            block_env     : vec![BTreeMap::new()],
            reduced_cells : vec![]
        }
    }

    // AT NEW BLOCK
    pub fn push_block(&mut self) {
        self.block_env.push(BTreeMap::new());
    }

    // AT END OF BLOCK
    pub fn pop_block(&mut self) {
        self.block_env.pop();
    }

    pub fn is_rec_call(&self, name : &String) -> bool {
        if is_in_local!(self, name) {
            return false;
        } else {
            return self.fun_env.rec_name == *name;
        }
    }

    pub fn set_rec_used(&mut self, val : bool) {
        self.fun_env.rec_used = val;
    }

    pub fn is_rec_used(&self) -> bool {
        self.fun_env.rec_used
    }

    pub fn pack(&self) -> &Pack {
        unsafe{ &* self.fun_env.global }
    }

    // labels only in fun_env
    pub fn add_loop_label(&mut self, name : &String) {
        self.fun_env.loop_labels.push(&*name);
    }

    // labels only in fun_env
    pub fn pop_loop_label(&mut self) {
        self.fun_env.loop_labels.pop();
    }

    // labels only in fun_env
    pub fn get_loop_label(&self, name : &String) -> Option<usize> {
        // getting count of loops which must skip to stop target
        // or 'None' if target not exist
        let loop_labels = &self.fun_env.loop_labels;
        let len = loop_labels.len();
        for i in 0 .. len {
            let val = unsafe { *loop_labels[len - i - 1] == *name };
            if val {
                return Some(i);
            }
        }
        return None;
    }

    // add outer env to current env
    pub fn add_outer(&mut self, out : &LocEnv) {
        let fun_env : &mut FunEnv = &mut self.fun_env;
        macro_rules! copy_pool {
            ($pool:expr) => {
                for (key,val) in $pool.iter() {
                    fun_env.outers.insert(key.clone(), val.clone());
                }
            }
        }
        copy_pool!(out.fun_env.outers);
        copy_pool!(out.fun_env.local);
        for env in out.block_env.iter() {
            copy_pool!(env);
        }
        for t in out.fun_env.templates.iter() {
            fun_env.templates.insert(t.clone());
        }
    }

    pub fn show(&self) -> String {
        let mut acc = self.fun_env.show();
        for env in self.block_env.iter() {
            let _ = write!(acc, "SUB: [");
            for name in env.keys() {
                let _ = write!(acc, "{},", name);
            }
            let _ = write!(acc, "]\n");
        }
        acc
    }

    pub fn replace_unk(&self, name : &String, tp : RType) {
        for env in self.block_env.iter().rev() {
            match env.get(name) {
                Some(ans) => {
                	*ans.borrow_mut() = tp.clone();
                	break
                },
                _ => ()
            }
        }
        // IF VAR FOUND IN BLOCK ENV THEN
        // THIS CODE WILL BE SKIPPED
        self.fun_env.replace_unk(name, tp)
    }

    pub fn get_local_var(&self, name : &String) -> RType {
        for env in self.block_env.iter().rev() {
            match env.get(name) {
                Some(v) => return v.borrow().clone(),
                _ => ()
            }
        }
        // IF VAR FOUND IN BLOCK ENV THEN
        // THIS CODE WILL BE SKIPPED
        self.fun_env.get_local_var(name)
    }

    // SET VAR TYPE TO tp_dst AND CHANGE prefix(%loc, %out, %rec, %mod or full path)
    // CURSOR USED FOR SEND ERR MESSAGE
    pub fn get_var(&self, pref : &mut Vec<String>, name : &String, tp_dst : &MRType, pos : &Cursor) -> CheckRes {
        // macro_rules! clone_type { ($t:expr) => { match *$t {Ok(ref t) => (*t).clone(), Err(ref t) => (**t).clone()} }; }
        macro_rules! clone_type {
        	($t:expr) => ($t.borrow().clone())
        }
        if pref.len() == 0 || pref[0] == "%loc" {
            // IF PREFIX NOT SETTED THEN TRY ALL LOCAL
            // THEN TRY FUN ENV
            // AFTER FOUND SET NEW PREFIX
            for env in self.block_env.iter().rev() {
                match env.get(name) {
                    Some(t) => {
                        // FOUND IN LOCAL
                        *tp_dst.borrow_mut() = clone_type!(t);
                        if pref.len() == 0 {
                            pref.push("%loc".to_string());
                        }
                        return Ok(())
                    },
                    _ => ()
                }
            }
            // IF RETURN NOT CALLED, THEN IN LOCAL NOT FOUND
            self.fun_env.get_var(pref, name, tp_dst, pos)
        } else {
            return self.fun_env.get_var(pref, name, tp_dst, pos);
        }
    }

    pub fn add_loc_var(&mut self, name : &String, tp : MRType, pos : &Cursor) -> CheckRes {
        // block_env never empty as container of env
        match self.block_env.last_mut().unwrap().insert(name.clone(), tp) {
            Some(_) => syn_throw!(format!("local var {} already exist", name), pos),
            _ => return Ok(())
        }
    }

    pub fn add_loc_var_const_t(&mut self, name : &String, tp : RType, pos : &Cursor) -> CheckRes {
        let cell = Type::mtype(tp);
        match self.block_env.last_mut().unwrap().insert(name.clone(), cell) {
            Some(_) => syn_throw!(format!("local var {} already exist", name), pos),
            _ => return Ok(())
        }
    }

    pub fn fun_env(&self) -> &FunEnv {
        &self.fun_env
    }

    pub fn fun_env_mut(&mut self) -> &mut FunEnv {
        &mut self.fun_env
    }
}
