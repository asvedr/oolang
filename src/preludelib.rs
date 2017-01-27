use type_check::tclass::*;
use type_check::pack::*;
use syn::type_sys::*;
use std::collections::HashMap;

pub struct Prelude {
	pub pack : Box<Pack>,
	pub cfns : HashMap<String,String>
}

impl Prelude {
	pub fn new() -> Prelude {
		let mut pack = Box::new(Pack::new());
		let mut cfns = HashMap::new();
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
			pack.cls.insert($name.to_string(), c);
			pack.cls.get($name).unwrap().borrow_mut()
		}}; }
		macro_rules! newf {
			($name:expr, $cname:expr, $t:expr) => {{
				pack.fns.insert($name.to_string(), $t);
				cfns.insert($name.to_string(), $cname.to_string());
			}};
			($name:expr, $cname:expr, $t:expr, $noexc:expr) => {{
				pack.fns.insert($name.to_string(), $t);
				pack.fns_noex.insert($name.to_string());
				cfns.insert($name, $cname);
			}};
		}
		macro_rules! meth {($cls:expr, $name:expr, $t:expr, $noexc:expr) => {{
			(*$cls).pubs.insert($name.to_string(), Attr::method($t, $noexc));
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
		/*{
			let mut except = newc!("Exception", "_std_exc", vec![], vec![Type::str()]);
			meth!(except, "param", type_fn!(vec![], Type::str()), true);
		}*/
		{
			newf!("print", "_std_print", type_fn!(vec![Type::str()], Type::void()));
			newf!("readln", "_std_readln", type_fn!(vec![], Type::str()));
			newe!("Exception", Type::str());
			newe!("IndexError");
			newe!("NullPtr");
		}
		Prelude{pack : pack, cfns : cfns}
	}
}
