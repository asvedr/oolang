// regress type calculation system

use syn::*;
use type_check::utils::*;
use std::mem;

// TODO: Prop
pub fn regress_expr(env : &mut LocEnv, expr : &mut Expr, e_type : &Type) -> CheckRes {
	macro_rules! regress {($e:expr, $t:expr) => {try!(regress_expr(env, $e, $t))}; }
	match expr.val {
		EVal::Call(ref mut tmpl, ref mut fun, ref mut args) => {
			macro_rules! a { () => {args[0]}; }
			macro_rules! b { () => {args[1]}; }
			macro_rules! coerse { ($e:expr, $from:expr, $to:expr) => {{
				let val  = mem::replace(&mut $e.val, EVal::Null);
				let expr = Expr{val : val, kind : $from, addres : $e.addres.clone(), op_flag : 0};
				$e.val   = EVal::ChangeType(Box::new(expr), $to);
				$e.kind  = $to;
			}}; }
			macro_rules! apply {($e:expr, $f:expr, $t:expr) => {{
				if $e.kind.is_unk() {
					regress!(&mut $e, &$t);
				} else if $e.kind == $f {
					coerse!($e, $f, $t)
				} else if !($e.kind.is_real()) {
					throw!(format!("expected int or real, found {:?}", $e.kind), $e.addres)
				}
			}};}
			macro_rules! happly {($e:expr, $t:expr) => {{
				if $e.kind.is_unk() {
					regress!(&mut $e, &$t)
				} else if $e.kind != $t {
					throw!(format!("expected {:?}, found {:?}", $t, $e.kind), $e.addres)
				}
			}};}
			match fun.op_flag {
				IROP if e_type.is_int() => {
					apply!(a!(), Type::Real, Type::Int);
					apply!(b!(), Type::Real, Type::Int);
					fun.kind = type_fn!(vec![Type::Int, Type::Int], Type::Int);
				},
				IROP if e_type.is_real() => {
					apply!(a!(), Type::Int, Type::Real);
					apply!(b!(), Type::Int, Type::Real);
					fun.kind = type_fn!(vec![Type::Real, Type::Real], Type::Real);
				},
				IROP => throw!(format!("expected {:?} found num operation", e_type), expr.addres),
				IROPB if e_type.is_bool() => {
					if a!().kind.is_real() || b!().kind.is_real() {
						apply!(a!(), Type::Int, Type::Real);
						apply!(b!(), Type::Int, Type::Real);
						fun.kind = type_fn!(vec![Type::Real, Type::Real], Type::Bool);
					} else {
						apply!(a!(), Type::Real, Type::Int);
						apply!(b!(), Type::Real, Type::Int);
						fun.kind = type_fn!(vec![Type::Int, Type::Int], Type::Bool);
					}
				},
				IROPB => throw!(format!("expected {:?} found bool", e_type), expr.addres),
				IOP   => {
					happly!(a!(), Type::Int);
					happly!(b!(), Type::Int);
					fun.kind = type_fn!(vec![Type::Int,Type::Int],Type::Int);
				},
				ROP   => {
					happly!(a!(), Type::Real);
					happly!(b!(), Type::Real);
					fun.kind = type_fn!(vec![Type::Real,Type::Real],Type::Real);
				}
				AOP   => {
					if a!().kind.is_unk() && b!().kind.is_unk() {
						// CAN'T SOLUTE THIS
						// return;
					} else if a!().kind.is_unk() {
						let t : *const Type = &b!().kind;
						unsafe { regress!(&mut a!(), &*t) }
					} else if b!().kind.is_unk() {
						let t : *const Type = &b!().kind;
						unsafe { regress!(&mut a!(), &*t) }
					} else if a!().kind != b!().kind {
						throw!(format!("expected {:?}, found {:?}", a!().kind, b!().kind), b!().addres)
					}
				}
				BOP   => {
					happly!(a!(), Type::Bool);
					happly!(b!(), Type::Bool);
					fun.kind = type_fn!(vec![Type::Bool,Type::Bool], Type::Bool);
				},
				_     => { // NOP
				}
			}
		},
		// TYPE OF 'NEW CLASS' IS ALWAYS KNOWN
		//NewClass(ref tmpl,ref mut pref, ref mut name, ref mut args) => ,
		EVal::Item(ref mut cont, ref mut index) => {
			// IF INDEX IS INT OR UNKNOWN TRY TO FOLD TO ARRAY ELSE ASSOC
			// 'CAUSE OF REGRESS CALLED CONT TYPE EXACTLY UNKNOWN
			if index.kind.is_unk() {
				// ARRAY
				regress!(cont, &Type::Arr(/*Box::new*/vec![e_type.clone()]));
				regress!(index, &Type::Int);
			} else if index.kind.is_int() {
				regress!(cont, &Type::Arr(/*Box::new*/vec!(e_type.clone())));
			} else if index.kind.is_char() || index.kind.is_str() {
				// ASSOC
				let tp = Type::Class(vec!["%std".to_string()], "Asc".to_string(), Some(vec![index.kind.clone(), e_type.clone()]));
				regress!(cont, &tp);
			} else {
				throw!(format!("can't use {:?} for indexing", index.kind), index.addres.clone());
			}
		},
		EVal::Var(_, ref name) => { // namespace, name
			if expr.kind.is_unk() {
				env.replace_unk(name, e_type);
			}
		},
		EVal::Arr(ref mut items) => {
			match *e_type {
				Type::Arr(ref itp) => {
					for i in items.iter_mut() {
						try!(regress_expr(env, i, &itp[0]))
					}
				},
				ref a => throw!(format!("expected {:?}, found array", a), expr.addres)
			}
		},
		EVal::Asc(ref mut pairs) => { // only strings, chars and int allowed for key
			//expr.kind = e_type.clone();
			match *e_type {
				Type::Class(ref pref, ref name, ref params) => {
					if !(pref.len() == 0 && pref[0] == "%std" && name == "Asc") {
						let pars = match *params { Some(ref p) => p, _ => panic!() };
						for pair in pairs.iter_mut() {
							regress!(&mut pair.a, &pars[0]);
							regress!(&mut pair.b, &pars[1]);
						}
					} else {
						throw!(format!("expected {:?}, found asc", e_type), expr.addres)
					}
				},
				_ => throw!(format!("expected {:?}, found asc", e_type), expr.addres)
			}
		},
		// CAN'T REGRESS PROPERTY GET
		//Prop(Box<Expr>,String),
		_ => ()
	}
	expr.kind = e_type.clone();
	Ok(())
}
