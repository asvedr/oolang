use syn_reserr::*;
use syn_utils::*;
use syn_expr::*;
use type_sys::*;
//use std::fmt;

pub struct SynCatch<DF> {
	except : Option<Type>,
	vname  : Option<String>,
	act    : Vec<Act<DF>>
}

pub enum ActVal<DF> {
	Expr(Expr),
	DFun(Box<DF>),
	//   name    var type     init val
	DVar(String,Option<Type>,Option<Expr>),
	//   a  =  b
	Asg(Expr,Expr),
	Ret(Option<Expr>),
	Break(Option<String>), // label to loop
	//     label          cond   actions
	While(Option<String>, Expr, Vec<Act<DF>>),
	//For(String,)
	// cond   then    else
	If(Expr,Vec<Act<DF>>,Vec<Act<DF>>),
	Try(Vec<Act<DF>>,Vec<SynCatch<DF>>),
	Throw(Expr)
}

pub struct Act<DF> {
	pub val    : ActVal<DF>,
	pub addres : Cursor 
}

macro_rules! act {
	($v:expr, $addr:expr) => {Act{val : $v, addres : $addr}};
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
				let tp = match *t {
					Some(ref t) => format!("{:?}", t),
					_ => "(?)".to_string()
				};
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
			ActVal::Break(ref lab) => vec![format!("{}BREAK {:?}", tab, lab)],
			ActVal::Try(ref acts, ref ctchs) => {
				let mut res = vec![format!("{}TRY", tab)];
				for act in acts.iter() {
					for line in act.show(layer + 1) {
						res.push(line);
					}
				}
				for ctch in ctchs.iter() {
					res.push(format!("{}CATCH {:?}:{:?}", tab, ctch.except, ctch.vname));
					for act in ctch.act.iter() {
						for line in act.show(layer + 1) {
							res.push(line);
						}
					}
				}
				res
			},
			ActVal::Throw(ref e) => {
				let mut res = vec![format!("{}THROW", tab)];
				for line in e.show(layer + 1) {
					res.push(line);
				}
				res
			}
		}
	}
}

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
			syn_throw!(format!("expected ';' or '{}'", "{"), curs)
		}
	}
}

fn is_act_end(lexer : &Lexer, curs : &Cursor) -> bool {
	match lexer.lex(curs) {
		Ok(ans) => ans.val == ";" || ans.val == "}",
		_ => true
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
				Some(tp.val)
			} else {
				None
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
				let label = lex_type!(lexer, &curs, LexTP::Id);
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
		//"for"
		"break" => {
			if is_act_end(lexer, &ans.cursor) {
				syn_ok!(make!(ActVal::Break(None)), ans.cursor)
			} else {
				let label = lex_type!(lexer, &ans.cursor, LexTP::Id);
				syn_ok!(make!(ActVal::Break(Some(label.val))), label.cursor)
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
			println!("!");
			loop {
				let ans = lex!(lexer, &curs);
				if ans.val == "catch" {
					curs = ans.cursor;
					let ans = lex!(lexer, &curs);
					if ans.val == "{" {
						let al = try!(parse_act_list(lexer, &curs, fparse));
						curs = al.cursor;
						ctchs.push(SynCatch{except : None, vname : None, act : al.val});
					} else {
						let ans = lex_type!(lexer, &curs, LexTP::Id);
						curs = lex!(lexer, &ans.cursor, ":");
						let tp = try!(parse_type(lexer, &curs));
						let al = try!(parse_act_list(lexer, &tp.cursor, fparse));
						curs = al.cursor;
						ctchs.push(SynCatch{except : Some(tp.val), vname : Some(ans.val), act : al.val});
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
			let expr = try!(parse_expr(lexer, &ans.cursor));
			syn_ok!(make!(ActVal::Throw(expr.val)), expr.cursor);
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
					EVal::Prop(_, _) => (),
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
