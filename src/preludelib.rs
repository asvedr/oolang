use type_check::tclass::*;
use type_check::pack::*;
use std::rc::Rc;
use syn::type_sys::*;
use std::collections::BTreeMap;

pub struct Prelude {
	pub pack : Box<Pack>
}

impl Prelude {
	pub fn new() -> Prelude {
		//let mut fns  = Box::new(vec![]);
		//let mut clss = vec![];
		let mut pack = Box::new(Pack::new());
		//let mut res = Box::new(Prelude{fns : Rc::new(RefCell::new(vec![])), pack : Pack::new()}); 
		//res.pack.name.push("%std".to_string());
		/*macro_rules! newf_loc {($t:expr) => {{
			res.fns.borrow_mut().push($t);
			&res.fns.borrow()[res.fns.borrow().len() - 1]
		}}; }*/
		pack.name.push("%std".to_string());
		macro_rules! newe {
			($n:expr, $arg:expr) => {pack.excepts.insert($n.to_string(), Some($arg))};
			($n:expr) => {pack.excepts.insert($n.to_string(), None)};
		}
		macro_rules! newc {($name:expr, $p:expr, $acnt:expr) => {{
			let c = TClass{source : None, parent : None, privs : BTreeMap::new(), pubs : BTreeMap::new(), params : $p, args : $acnt};
			//let lnk : *mut TClass = &mut clss[clss.len() - 1];
			pack.cls.insert($name.to_string(), c);
			pack.cls.get_mut($name).unwrap()
		}}; }
		macro_rules! newf {
			($name:expr, $t:expr) => {{
				pack.fns.insert($name.to_string(), $t);
			}};
			($name:expr, $t:expr, $noexc:expr) => {{
				pack.fns.insert($name.to_string(), $t);
				pack.fns_noex.insert($name.to_string());
			}};
		}
		macro_rules! meth {($cls:expr, $name:expr, $t:expr, $noexc:expr) => {{
			unsafe { (*$cls).pubs.insert($name.to_string(), Attr::method(/*newf_loc!($t)*/$t, $noexc)); }
		}};}
		{
		let arr : *mut TClass = newc!("%arr", vec!["a".to_string()], vec![]);
		meth!(arr, "len", type_fn!(vec![], Type::int()), true);
		meth!(arr, "get", type_fn!(vec![Type::int()], type_c!("a".to_string())), false);
		let asc : *mut TClass = newc!("Asc", vec!["a".to_string(),"b".to_string()], vec![]);
		meth!(asc, "len", type_fn!(vec![], Type::int()), true);
		meth!(asc, "keys", type_fn!(vec![], Type::arr(type_c!("a".to_string()))), true);
		meth!(asc, "get", type_fn!(vec![type_c!("a".to_string())], type_c!("b".to_string())), false);
		meth!(asc, "set", type_fn!(vec![type_c!("a".to_string()), type_c!("b".to_string())], Type::void()), false);
		meth!(asc, "has_key", type_fn!(vec![type_c!("a".to_string())], Type::bool()), true);
		let str_s : *mut TClass = newc!("%str", vec![], vec![]);
		meth!(str_s, "len", type_fn!(vec![], Type::int()), true);
		meth!(str_s, "get", type_fn!(vec![Type::int()], Type::char()), false);
		meth!(str_s, "set", type_fn!(vec![Type::int(),Type::char()], Type::void()), false);
		meth!(str_s, "add", type_fn!(vec![Type::str()], Type::void()), false);
		let except : *mut TClass = newc!("Exception", vec![], vec![Type::str()]);
		meth!(except, "param", type_fn!(vec![], Type::str()), true);
		newf!("print",  type_fn!(vec![Type::str()], Type::void()));
		newf!("readln", type_fn!(vec![], Type::str()));
		//newf!("add_str", type_fn!(vec![Type::str(), Type::str()], Type::str()), true);
		//newf!("add_i", type_fn!(vec![Type::int(), Type::int()], Type::int()));
		newe!("IndexError");
		newe!("NullPtr");
		}
		Prelude{pack : pack}
	}
}
