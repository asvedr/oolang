use type_check::tclass::*;
use type_check::pack::*;
//use std::rc::Rc;
use syn::type_sys::*;
//use std::collections::BTreeMap;

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
		macro_rules! newc {($name:expr, $fname:expr, $p:expr, $acnt:expr) => {{
			let c = TClass::new($fname.to_string());
			{
				let mut c = c.borrow_mut(); 
				c.params = $p;
				c.args = $acnt;
			}
			//let lnk : *mut TClass = &mut clss[clss.len() - 1];
			pack.cls.insert($name.to_string(), c);
			pack.cls.get($name).unwrap().borrow_mut()
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
			(*$cls).pubs.insert($name.to_string(), Attr::method(/*newf_loc!($t)*/$t, $noexc));
		}};}
		{
			let mut arr = newc!("%arr", "_std_arr", vec!["a".to_string()], vec![]);
			meth!(arr, "len", type_fn!(vec![], Type::int()), true);
			meth!(arr, "get", type_fn!(vec![Type::int()], type_c!("a".to_string())), false);
		}
		{
			let mut asc = newc!("Asc", "_std_asc", vec!["a".to_string(),"b".to_string()], vec![]);
			meth!(asc, "len", type_fn!(vec![], Type::int()), true);
			meth!(asc, "keys", type_fn!(vec![], Type::arr(type_c!("a".to_string()))), true);
			meth!(asc, "get", type_fn!(vec![type_c!("a".to_string())], type_c!("b".to_string())), false);
			meth!(asc, "set", type_fn!(vec![type_c!("a".to_string()), type_c!("b".to_string())], Type::void()), false);
			meth!(asc, "has_key", type_fn!(vec![type_c!("a".to_string())], Type::bool()), true);
		}
		{
			let mut str_s = newc!("%str", "_std_str", vec![], vec![]);
			meth!(str_s, "len", type_fn!(vec![], Type::int()), true);
			meth!(str_s, "get", type_fn!(vec![Type::int()], Type::char()), false);
			meth!(str_s, "set", type_fn!(vec![Type::int(),Type::char()], Type::void()), false);
			meth!(str_s, "add", type_fn!(vec![Type::str()], Type::void()), false);
		}
		{
			let mut except = newc!("Exception", "_std_exc", vec![], vec![Type::str()]);
			meth!(except, "param", type_fn!(vec![], Type::str()), true);
		}
		{
			newf!("print",  type_fn!(vec![Type::str()], Type::void()));
			newf!("readln", type_fn!(vec![], Type::str()));
			newe!("IndexError");
			newe!("NullPtr");
		}
		Prelude{pack : pack}
	}
}
