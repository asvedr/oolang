use syn::*;
use pack::

pub fn set_noexc(fun : &mut SynFn, pack : &Pack) -> bool {

	for act in fun.body.iter() {
		match act.val {
			ActVal::Expr(ref e) => can_throw!(e),
			ActVal::DVar(_,_,)
		}
	}
}

fn can_throw(e : &Expr, pack : &Pack) -> bool {
}
