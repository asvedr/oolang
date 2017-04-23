use type_check::tclass::*;
use bytecode::exc_keys::*;
pub use std::cell::Ref;
use std::collections::HashMap;

// global values for module
pub struct GlobalConf {
    pub exceptions  : RExcKeys,
    pub classes  : HashMap<String,RTClass>,
    pub mod_name : Vec<String>,
    pub fns      : HashMap<String,String> // map of full names
}

impl GlobalConf {
    pub fn new(exc : RExcKeys, mname : Vec<String>) -> GlobalConf {
        GlobalConf{
            exceptions  : exc,//ExcKeys::new(0),
            classes  : HashMap::new(),
            fns      : HashMap::new(),
            mod_name : mname
        }
    }
    pub fn add_class(&mut self, class : RTClass) {
        let name = {
            let c = class.borrow_mut();
            // XXX cause of info about #NoExcept
            //c.prepare_to_translation();
            c.fname.clone()
        };
        self.classes.insert(name, class);
    }
    // 'cause of on translation can't get class out of table
    /*
        use .get(name).get_virt_i  - to get slot of virtual or check 'is it virtual'
        use .get(name).method2name - to get fname of regular method
        use .get(name).prop_i      - to get slot of prop
    */
    #[inline(always)]
    pub fn get_class(&self, name : &String) -> Ref<TClass> {
        match self.classes.get(name) {
            Some(val) => val.borrow(),
            _ => panic!()
        }
    }
    #[inline(always)]
    pub fn get_fun(&self, name : &String) -> &String {
        match self.fns.get(name) {
            Some(val) => val,
            _ => panic!()
        }
    }
    #[inline(always)]
    pub fn get_exc(&self, pref : &Vec<String>, name : &String) -> usize {
        if pref.len() == 0 || pref[0] == "%mod" {
            self.exceptions.get(&self.mod_name, name)
        } else {
            self.exceptions.get(pref,name)
        }
    }
    #[inline(always)]
    pub fn destroy(self) -> RExcKeys {
        self.exceptions
    }
}

