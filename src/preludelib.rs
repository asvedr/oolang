use type_check::tclass::*;
use type_check::pack::*;
use syn::type_sys::*;
use std::collections::BTreeMap;

pub struct Prelude {
//	tcls : Vec<TClass>,
	fns  : Vec<Type>,
	pub pack : Pack
}

impl Prelude {
	pub fn new() -> Prelude {
		let mut fns  = vec![];
		//let mut clss = vec![];
		let mut pack = Pack::new();/*Pack {
			name     : vec![format!("%std")],
			packs    : HashMap::new(),
			out_cls  : HashMap::new(),
			out_fns  : BTreeMap::new(),
			cls      : HashMap::new(),
			fns      : BTreeMap::new(),
			out_exc  : BTreeMap::new(),
			excepts  : BTreeMap::new()
		};*/
		pack.name.push("%std".to_string());
		macro_rules! newf_loc {($t:expr) => {{
			fns.push($t);
			&fns[fns.len() - 1]
		}}; }
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
				pack.out_fns.add($name.to_string());
			}};
		}
		macro_rules! meth {($cls:expr, $name:expr, $t:expr, $noexc:expr) => {{
			unsafe { (*$cls).pubs.insert($name.to_string(), Attr::method(newf_loc!($t), $noexc)); }
		}};}
		{
		let arr : *mut TClass = newc!("%arr", vec!["a".to_string()], vec![]);
		meth!(arr, "len", type_fn!(vec![], Type::Int), true);
		meth!(arr, "get", type_fn!(vec![Type::Int], type_c!("a".to_string())), false);
		let asc : *mut TClass = newc!("Asc", vec!["a".to_string(),"b".to_string()], vec![]);
		meth!(asc, "len", type_fn!(vec![], Type::Int), true);
		meth!(asc, "keys", type_fn!(vec![], Type::Arr(/*Box::new*/vec![type_c!("a".to_string())])), true);
		meth!(asc, "get", type_fn!(vec![type_c!("a".to_string())], type_c!("b".to_string())), false);
		meth!(asc, "set", type_fn!(vec![type_c!("a".to_string()), type_c!("b".to_string())], Type::Void), false);
		meth!(asc, "has_key", type_fn!(vec![type_c!("a".to_string())], Type::Bool), true);
		let str_s : *mut TClass = newc!("%str", vec![], vec![]);
		meth!(str_s, "len", type_fn!(vec![], Type::Int), true);
		meth!(str_s, "get", type_fn!(vec![Type::Int], Type::Char), false);
		meth!(str_s, "set", type_fn!(vec![Type::Int,Type::Char], Type::Void), false);
		let except : *mut TClass = newc!("Exception", vec![], vec![Type::Str]);
		meth!(except, "param", type_fn!(vec![], Type::Str), true);
		newf!("print",  type_fn!(vec![Type::Str], Type::Void));
		newf!("readln", type_fn!(vec![], Type::Str));
		newe!("IndexError");
		newe!("NullPtr");
		}
		//println!("{}",pack.show());
		Prelude {
			fns  : fns,
			pack : pack
		}
	}
}
