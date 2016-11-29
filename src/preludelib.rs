use type_check_utils::*;
use type_sys::*;
use std::collections::{HashMap, HashSet, BTreeMap};

pub struct Prelude {
//	tcls : Vec<TClass>,
	fns  : Vec<Type>,
	pub pack : Pack
}

impl Prelude {
	pub fn new() -> Prelude {
		let mut fns  = vec![];
		//let mut clss = vec![];
		let mut pack = Pack {
			name    : vec![format!("%std")],
			packs   : HashMap::new(),
			out_cls : HashMap::new(),
			out_fns : BTreeMap::new(),
			cls     : HashMap::new(),
			fns     : BTreeMap::new()
		};
		macro_rules! newf {($t:expr) => {{
			fns.push($t);
			&fns[fns.len() - 1]
		}}; }
		macro_rules! newc {($name:expr, $p:expr, $acnt:expr) => {{
			let c = TClass{parent : None, privs : BTreeMap::new(), pubs : BTreeMap::new(), params : $p, args : $acnt};
			//let lnk : *mut TClass = &mut clss[clss.len() - 1];
			pack.cls.insert($name.to_string(), c);
			pack.cls.get_mut($name).unwrap()
		}}; }
		macro_rules! meth {($cls:expr, $name:expr, $t:expr) => {{
			unsafe { (*$cls).pubs.insert($name.to_string(), newf!($t) ); }
		}};}
		{
		let arr : *mut TClass = newc!("%arr", vec!["a".to_string()], vec![]);
		meth!(arr, "len", type_fn!(vec![], Type::Int));
		meth!(arr, "get", type_fn!(vec![Type::Int], type_c!("a".to_string())));
		let asc : *mut TClass = newc!("Asc", vec!["a".to_string(),"b".to_string()], vec![]);
		meth!(asc, "len", type_fn!(vec![], Type::Int));
		meth!(asc, "keys", type_fn!(vec![], Type::Arr(/*Box::new*/vec![type_c!("a".to_string())])));
		meth!(asc, "get", type_fn!(vec![type_c!("a".to_string())], type_c!("b".to_string())));
		meth!(asc, "has_key", type_fn!(vec![type_c!("a".to_string())], Type::Bool));
		let str_s : *mut TClass = newc!("%str", vec![], vec![]);
		meth!(str_s, "len", type_fn!(vec![], Type::Int));
		meth!(str_s, "get", type_fn!(vec![Type::Int], Type::Char));
		meth!(str_s, "set", type_fn!(vec![Type::Int,Type::Char], Type::Void));
		}
		Prelude {
//			tcls : clss,
			fns  : fns,
			pack : pack
		}
	}
}
