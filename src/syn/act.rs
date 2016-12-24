use syn::reserr::*;
use syn::utils::*;
use syn::expr::*;
use syn::type_sys::*;
//use std::fmt;

// DF is non declared struct for 'def function'
pub struct SynCatch<DF> {
	pub epref  : Vec<String>,
	pub ekey   : String,
	pub vname  : Option<String>,
	pub vtype  : Type,
	pub act    : Vec<Act<DF>>,
	pub addres : Cursor
}

pub enum ActVal<DF> {
	Expr(Expr),
	DFun(Box<DF>),
	//   name   var type     init val
	DVar(String,Type,Option<Expr>),
	//   a  =  b
	Asg(Expr,Expr),
	Ret(Option<Expr>),
	Break(Option<String>, usize), // label to loop. usize - count to skip(calculating on check-type)
	//     label          cond   actions
	While(Option<String>, Expr, Vec<Act<DF>>),
	//   label         vname  from  to   body
	For(Option<String>,String,Expr,Expr,Vec<Act<DF>>), // for i in range(a + 1, b - 2) {}
	//      label          vname  vtype cont  body
	Foreach(Option<String>,String,Type, Expr,Vec<Act<DF>>),  // for i in array {}
	// cond   then    else
	If(Expr,Vec<Act<DF>>,Vec<Act<DF>>),
	Try(Vec<Act<DF>>,Vec<SynCatch<DF>>), // try-catch
	Throw(Vec<String>,String,Option<Expr>)
}

pub struct Act<DF> {
	pub val       : ActVal<DF>,
	pub addres    : Cursor, 
//	pub exist_unk : bool // field for typecheck 'unknown type exist in this action'
}

macro_rules! act {
	($v:expr, $addr:expr) => {Act{val : $v, addres : $addr/*, exist_unk : true*/}};
}

impl<DF : Show> Show for Act<DF> {
	fn show(&self, layer : usize) -> Vec<String> {
		self.val.show(layer)
	}
}

impl<DF : Show> Show for ActVal<DF> {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		match *self {
			ActVal::Expr(ref e) => e.show(layer), 
			ActVal::DFun(ref df) => df.show(layer),
			ActVal::DVar(ref n, ref t, ref v) => {
				let tp = format!("{:?}", t);
				let mut res = vec![format!("{}DEF VAR '{}' : {:?}", tab, n, tp)];
				match *v {
					Some(ref e) =>
						for line in e.show(layer + 1) {
							res.push(line);
						},
					_ => ()
				};
				res
			},
			ActVal::Asg(ref var, ref val) => {
				let mut res = vec![format!("{}ASSIGN", tab)];
				for line in var.show(layer + 1) {
					res.push(line);
				}
				for line in val.show(layer + 1) {
					res.push(line);
				}
				res
			},
			ActVal::If(ref c, ref t, ref e) => {
				let cnd = format!("{}IF", tab);
				let thn = format!("{}THEN", tab);
				let els = format!("{}ELSE", tab);
				let mut res = vec![cnd];
				for line in c.show(layer + 1) {
					res.push(line);
				}
				res.push(thn);
				for cmd in t.iter() {
					for line in cmd.show(layer + 1) {
						res.push(line);
					}
				}
				res.push(els);
				for cmd in e.iter() {
					for line in cmd.show(layer + 1) {
						res.push(line);
					}
				}
				res
			},
			ActVal::Ret(ref e) => {
				let mut res = vec![format!("{}RET", tab)];
				match *e {
					Some(ref e) =>
						for line in e.show(layer + 1) {
							res.push(line);
						},
					_ => ()
				}
				res
			},
			ActVal::While(ref label, ref cnd, ref act) => {
				let mut res = match *label {
					Some(ref l) => vec![format!("{}WHILE {}", tab, l)],
					_ => vec![format!("{}WHILE", tab)]
				};
				for line in cnd.show(layer + 1) {
					res.push(line);
				}
				for cmd in act.iter() {
					for line in cmd.show(layer + 1) {
						res.push(line);
					}
				}
				res
			},
			ActVal::For(ref label, ref name, ref a, ref b, ref body) => {
				let mut res = match *label {
					Some(ref l) => vec![format!("{}for::{} {} in range", tab, l, name)],
					_ => vec![format!("{}for {} in range", tab, name)]
				};
				for line in a.show(layer + 1) {
					res.push(line);
				}
				for line in b.show(layer + 1) {
					res.push(line);
				}
				for act in body.iter() {
					for line in act.show(layer + 1) {
						res.push(line);
					}
				}
				res
			},
			ActVal::Foreach(ref label, ref name, ref vtype, ref expr, ref body) => {
				let mut res = match *label {
					Some(ref l) => vec![format!("{}foreach::{} {} : {:?}", tab, l, name, vtype)],
					_ => vec![format!("{}foreach {}", tab, name)]
				};
				for line in expr.show(layer + 1) {
					res.push(line);
				}
				for act in body.iter() {
					for line in act.show(layer + 1) {
						res.push(line);
					}
				}
				res
			},
			ActVal::Break(ref lab, ref cnt) => vec![format!("{}BREAK {:?} {:?}", tab, lab, cnt)],
			ActVal::Try(ref acts, ref ctchs) => {
				let mut res = vec![format!("{}TRY", tab)];
				for act in acts.iter() {
					for line in act.show(layer + 1) {
						res.push(line);
					}
				}
				for ctch in ctchs.iter() {
					res.push(format!("{}CATCH {:?}::{} {:?}:{:?}", tab, ctch.epref, ctch.ekey, ctch.vname, ctch.vtype));
					for act in ctch.act.iter() {
						for line in act.show(layer + 1) {
							res.push(line);
						}
					}
				}
				res
			},
			ActVal::Throw(ref p, ref n, ref e) => {
				let mut res = vec![format!("{}THROW {:?}::{}", tab, p, n)];
				match *e {
					Some(ref e) =>
						for line in e.show(layer + 1) {
							res.push(line);
						},
					_ => ()
				}
				res
			}
		}
	}
}

// {a; b; c; ..}
pub fn parse_act_list<DF>(lexer : &Lexer, curs : &Cursor, fparse : &Parser<DF>) -> SynRes<Vec<Act<DF>>> {
	let mut curs : Cursor = lex!(lexer, curs, "{");
	let mut acc = vec![];
	loop {
		let ans = lex!(lexer, &curs);
		if ans.val == "}" {
			syn_ok!(acc, ans.cursor);
		}
		let ans = syn_try!(parse_act(lexer, &curs, fparse));
		acc.push(ans.val);
		curs = ans.cursor;
		let ans = lex!(lexer, &curs);
		if ans.val == "}" {
			syn_ok!(acc, ans.cursor);
		} else if ans.val == ";" {
			curs = ans.cursor;
		} else {
			syn_throw!(format!("expected ';' or '{}', found: {}", "{", ans.val), curs)
		}
	}
}

// ask lexer for ';' or '}'
fn is_act_end(lexer : &Lexer, curs : &Cursor) -> bool {
	match lexer.lex(curs) {
		Ok(ans) => ans.val == ";" || ans.val == "}",
		_ => true
	}
}

fn parse_e_name(lexer : &Lexer, curs : &Cursor) -> SynRes<(Vec<String>, String)> {
	let mut acc  = vec![];
	let mut name = String::new();
	let sym = lex_type!(lexer, curs, LexTP::Id);
	name = sym.val;
	let mut curs = sym.cursor;
	loop {
		let sym = lex!(lexer, &curs);
		if sym.val == "::" {
			acc.push(name);
			let sym = lex_type!(lexer, &curs, LexTP::Id);
			curs = sym.cursor;
			name = sym.val;
		} else {
			syn_ok!( (acc, name), curs);
		}
	}
}

pub fn parse_act<DF>(lexer : &Lexer, curs : &Cursor, fparse : &Parser<DF>) -> SynRes<Act<DF>> {
	let ans = lex!(lexer, curs);
	let addr = curs.clone();
	macro_rules! make {($act:expr) => {act!($act, addr)}}
	match &*ans.val {
		"fn" => {
			let ans = try!(fparse(lexer, curs));
			syn_ok!(make!(ActVal::DFun(Box::new(ans.val))), ans.cursor)
		},
		"var" => {
			// name
			let vname = lex_type!(lexer, &ans.cursor, LexTP::Id);
			let mut curs = vname.cursor;
			let vname = vname.val;
			// try find type
			let tpflag = lex!(lexer, &curs);
			let tp = if tpflag.val == ":" {
				let tp = try!(parse_type(lexer, &tpflag.cursor));
				curs = tp.cursor;
				tp.val
			} else {
				Type::Unk
			};
			// try find init val
			if is_act_end(lexer, &curs) {
				syn_ok!(make!(ActVal::DVar(vname, tp, None)), curs)
			} else {
				let curs = lex!(lexer, &curs, "=");
				let e = try!(parse_expr(lexer, &curs));
				syn_ok!(make!(ActVal::DVar(vname, tp, Some(e.val))), e.cursor)
			}
		},
		"return" => {
			if is_act_end(lexer, &ans.cursor) {
				syn_ok!(make!(ActVal::Ret(None)), ans.cursor)
			} else {
				let expr = try!(parse_expr(lexer, &ans.cursor));
				syn_ok!(make!(ActVal::Ret(Some(expr.val))), expr.cursor)
			}
		},
		"while" => {
			// label
			let sym = lex!(lexer, &ans.cursor);
			let mut curs = ans.cursor;
			let label = if sym.val == "::" {
				let label = lex_type!(lexer, &sym.cursor, LexTP::Id);
				curs = label.cursor;
				Some(label.val)
			} else {
				None
			};
			// cond
			let cond = try!(parse_expr(lexer, &curs));
			// act
			let act = try!(parse_act_list(lexer, &cond.cursor, fparse));
			syn_ok!(make!(ActVal::While(label, cond.val, act.val)), act.cursor)
		},
		"for" => {
			// label
			let sym = lex!(lexer, &ans.cursor);
			let mut curs = ans.cursor;
			let label = if sym.val == "::" {
				let label = lex_type!(lexer, &sym.cursor, LexTP::Id);
				curs = label.cursor;
				Some(label.val)
			} else {
				None
			};
			// var name
			let var = lex_type!(lexer, &curs, LexTP::Id);
			let ans = lex!(lexer, &var.cursor);
			let mut tp = Type::Unk;
			if ans.val == ":" {
				let ans = try!(parse_type(lexer, &ans.cursor));
				curs = ans.cursor;
				tp = ans.val;
				curs = lex!(lexer, &curs, "in");
			} else if ans.val == "in" {
				//curs = lex!(lexer, &var.cursor, "in");
				curs = var.cursor;
			} else {
				syn_throw!("expected ':' or 'in'", var.cursor);
			}
			// 1 expr
			let a = try!(parse_expr(lexer, &curs));
			let sym = lex!(lexer, &a.cursor);
			if sym.val == ".." {
				// it's range
				let b = try!(parse_expr(lexer, &sym.cursor));
				let body = try!(parse_act_list(lexer, &b.cursor, fparse));
				syn_ok!(make!(ActVal::For(label, var.val, /*tp,*/ a.val, b.val, body.val)), body.cursor)
			} else {
				// it's single expr
				let body = try!(parse_act_list(lexer, &a.cursor, fparse));
				syn_ok!(make!(ActVal::Foreach(label, var.val, tp, a.val, body.val)), body.cursor)
			}
		},
		"break" => {
			if is_act_end(lexer, &ans.cursor) {
				syn_ok!(make!(ActVal::Break(None, 0)), ans.cursor)
			} else {
				let label = lex_type!(lexer, &ans.cursor, LexTP::Id);
				syn_ok!(make!(ActVal::Break(Some(label.val), 0)), label.cursor)
			}
		}
		"if" => {
			// cond
			let cond = try!(parse_expr(lexer, &ans.cursor));
			// then
			let thn = try!(parse_act_list(lexer, &cond.cursor, fparse));
			// else
			let sym = lex!(lexer, &thn.cursor);
			if sym.val == "else" {
				let els = try!(parse_act_list(lexer, &sym.cursor, fparse));
				syn_ok!(make!(ActVal::If(cond.val, thn.val, els.val)), els.cursor)
			} else {
				syn_ok!(make!(ActVal::If(cond.val, thn.val, vec![])), thn.cursor)
			}
		},
		"try" => {
			let act = try!(parse_act_list(lexer, &ans.cursor, fparse));
			let mut ctchs = vec![];
			let mut curs = act.cursor;
			let act = act.val;
			//println!("!");
			loop {
				let ans = lex!(lexer, &curs);
				if ans.val == "catch" {
					let addr = ans.cursor.clone();
					curs = ans.cursor;
					let ans = lex!(lexer, &curs);
					if ans.val == "{" {
						let al = parse_act_list(lexer, &curs, fparse)?;
						curs = al.cursor;
						ctchs.push(SynCatch{ekey : String::new(), epref : vec![], vtype : Type::Unk, vname : None, act : al.val, addres : addr});
					} else {
						// ans - EXCEPTON NAME
						//let key = lex_type!(lexer, &curs, LexTP::Id);
						let ename = parse_e_name(lexer, &curs)?;
						let (pref,name) = ename.val;
						curs = ename.cursor;
						let ecrs = curs.clone();
						//let tp = try!(parse_type(lexer, &curs));
						let sym = lex!(lexer, &curs);
						let var;
						if sym.kind == LexTP::Id {
							var = Some(sym.val);
							curs = sym.cursor;
						} else if sym.val == "{" {
							var = None;
							//curs = key.cursor;
						} else {
							syn_throw!(format!("excepcted var name of '{}', found '{}'", '{', sym.val), ecrs);
						}
						let al = parse_act_list(lexer, &curs, fparse)?;
						curs = al.cursor;
						ctchs.push(SynCatch{ekey : name, epref : pref, vname : var, vtype : Type::Unk, act : al.val, addres : addr});
						//ctchs.push(SynCatch{except : Some(tp.val), vname : Some(ans.val), act : al.val, addres : addr});
					}
				} else {
					if ctchs.len() == 0 {
						syn_throw!("'try' has no 'catch'", addr);
					} else {
						syn_ok!(make!(ActVal::Try(act, ctchs)), curs)
					}
				}
			}
		},
		"throw" => {
			//let name = lex_type!(lexer, &ans.cursor, LexTP::Id);
			let ename = parse_e_name(lexer, &ans.cursor)?;
			let (pref,name) = ename.val;
			let sym = lex!(lexer, &ename.cursor);
			if sym.val == ";" || sym.val == "}" {
				syn_ok!(make!(ActVal::Throw(pref, name, None)), ename.cursor)
			} else {
				let expr = try!(parse_expr(lexer, &ename.cursor));
				syn_ok!(make!(ActVal::Throw(pref, name, Some(expr.val))), expr.cursor);
			}
		},
		/* EXPR */
		_ => {
			let expr = try!(parse_expr(lexer, curs));
			let sym = lex!(lexer, &expr.cursor);
			// try make assign
			if sym.val == "=" {
				match expr.val.val {
					EVal::Var(_, _)  => (),
					EVal::Item(_, _) => (),
					EVal::Attr(_, _, _) => (),
					_ => syn_throw!(format!("assig allow only for vars, arr/asc items or obj props"), expr.cursor)
				}
				let val = try!(parse_expr(lexer, &sym.cursor));
				syn_ok!(make!(ActVal::Asg(expr.val, val.val)), val.cursor)
			} else {
				syn_ok!(make!(ActVal::Expr(expr.val)), expr.cursor)
			}
		}
	}
}
