// regress type calculation system

use syn_common::*;
use type_check_utils::*;

// TODO: Prop
pub fn regress_expr(env : &mut LocEnv, expr : &mut Expr, e_type : &Type) -> CheckRes {
	expr.kind = e_type.clone();
	/*match expr.val {
		Call(ref mut tmpl, ref mut fun, ref mut args) => {
			
		},
		NewClass(Option<Vec<Type>>,Option<Vec<String>>,String,Vec<Expr>),
		Item(Box<Expr>,Box<Expr>),
		Var(Option<Vec<String>>, String), // namespace, name
		Arr(Vec<Expr>),
		Asc(Vec<Pair<Expr,Expr>>), // only strings, chars and int allowed for key
		//Prop(Box<Expr>,String),
		_ => ()
	}*/
	Ok(())
}
