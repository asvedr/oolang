use std::rc::Rc;
use std::mem;

#[macro_export]
macro_rules! rc_eq {
	($a:expr, $b:expr) => (unsafe{ rc_eq($a, $b) })
}

pub unsafe fn rc_eq<A>(a : &Rc<A>, b : &Rc<A>) -> bool {
	let a : & *const A = mem::transmute(a);
	let b : & *const A = mem::transmute(b);
	return *a == *b;
}