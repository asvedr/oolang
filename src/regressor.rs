// regress type calculation system

use syn_common::*;
use type_check_utils::*;

// TODO: Prop
pub fn regress_expr(env : &mut LocEnv, expr : &mut Expr, e_type : &Type) -> CheckRes {
	macro_rules! regress {($e:expr, $t:expr) => {try!(regress_expr(env, $e, $t))}; }
	match expr.val {

	/*
		Call(ref mut tmpl, ref mut fun, ref mut args) => {
			match expr.op_flag {
				IROP if e_type.is_int() => {
				},
				IROP if e_type.is_real() => {
				},
				IROPB if e_type.is_int() => {
				},
				IROPB if e_type.is_real() => {
				},
				IOP   => {
				},
				ROP   => {
				}
				AOP   => {
				}
				BOP   => {
				},
				_     => { // NOP
				}
			}
		},*/

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
		//Prop(Box<Expr>,String),
		_ => ()
	}
	expr.kind = e_type.clone();
	Ok(())
}
