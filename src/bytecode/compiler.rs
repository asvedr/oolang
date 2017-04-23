use bytecode::global_conf::*;
use bytecode::compile_fun as c_fun;
use bytecode::exc_keys::*;
use syn::{Show, SynFn, SynMod};
use preludelib::*;

/*
	name specifications:
		sampMod::fun : _sampleMod_F_fun
		
		in sampMod
		fn fun() {
			fn local() {
				...
			}
			...
		}
		_sampMod_fun_L_fun

		class cls {
			pub fn meth () {
				...
			}
		}

		_sampMod_cls_M_meth

*/

// STRUCT FOR COMPILATION
// FIELDS IS CONFIGURATION
pub struct Compiler {
	pub global_conf : GlobalConf,
	pub dest_dir    : String
}

// COMPILED MODULE
pub struct CMod {
	pub pub_fns  : Vec<c_fun::CFun>,
	pub priv_fns : Vec<c_fun::CFun>
}

struct FunQueueItem<'a> {
	fun  : &'a SynFn,
	pref : Option<String>
}

impl Compiler {
	pub fn new(std : &Prelude, exceptions : RExcKeys, mod_name : Vec<String>, dest_dir : String) -> Compiler {
		let mut global_conf = GlobalConf::new(exceptions, mod_name);
		for (k,v) in std.cfns.iter() {
			global_conf.fns.insert(k.clone(), v.clone());
		}
		for tcls in std.pack.cls.values() {
			let name = tcls.borrow().fname.clone();
			global_conf.classes.insert(name, tcls.clone());
		}
		for e in std.pack.exceptions.keys() {
			global_conf.exceptions.add(&std.pack.name, e);
		}
		Compiler {
			global_conf       : global_conf,
			dest_dir : dest_dir
		}
	}
	pub fn destroy(self) -> RExcKeys {
		self.global_conf.destroy()
	}
	pub fn compile_mod(&self, smod : &SynMod) -> CMod {
		let mut pub_f = vec![];
		let mut priv_f = vec![];
		
		let mut queue = vec![];
		let mut mod_name = String::new();
		for i in self.global_conf.mod_name.iter() {
			if mod_name.len() == 0 {
				mod_name.push_str(&**i);
			} else {
				mod_name.push('_');
				mod_name.push_str(&**i);
			}
		}

		for fun in smod.funs.iter() {
			queue.push(FunQueueItem{fun : fun, pref : None});
		}
		let mut loc_funs = vec![];
		while let Some(item) = queue.pop() {
			let f = c_fun::compile(item.fun, &self.global_conf, &mod_name, &item.pref, &mut loc_funs);
			match item.pref {
				None => pub_f.push(f),
				_    => priv_f.push(f)
			}
			let pref = match item.pref {
				Some(p) => format!("{}_{}", p, item.fun.name),
				_ => format!("{}_{}", mod_name, item.fun.name)
			};
			while loc_funs.len() > 0 {
				let f = loc_funs.pop().unwrap();
				queue.push(FunQueueItem{fun : f, pref : Some(pref.clone())});
			}
		}

		CMod {
			pub_fns  : pub_f,
			priv_fns : priv_f
		}
	}
}

impl Show for CMod {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		let mut res = vec![];
		res.push(format!("{}PRIVATE", tab));
		for f in self.priv_fns.iter() {
			for l in f.show(layer + 1) {
				res.push(l);
			}
		}
		res.push(format!("{}PUBLIC", tab));
		for f in self.pub_fns.iter() {
			for l in f.show(layer + 1) {
				res.push(l)
			}
		}
		res
	}
}
