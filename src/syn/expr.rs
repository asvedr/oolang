use syn::utils::*;
use syn::reserr::*;
//use syn::lexer::*;
use syn::type_sys::*;
use std::str::FromStr;
use std::mem;

#[derive(Clone)]
pub enum EVal {
	Int        (i64),
	Real       (f64),
	Str        (String),
	Char       (char),
	Bool       (bool),
	Call       (Option<Vec<Type>>,Box<Expr>,Vec<Expr>),
	NewClass   (Option<Vec<Type>>,Vec<String>,String,Vec<Expr>),
	Item       (Box<Expr>,Box<Expr>),     // item of array or asc
	Var        (Vec<String>, String),     // namespace, name
	Arr        (Vec<Expr>),               // new arr
	Asc        (Vec<Pair<Expr,Expr>>),    // new Asc. Only strings, chars and int allowed for key
	//          obj       pname  is_meth
	Attr       (Box<Expr>,String,bool),   // geting class attrib: 'object.prop' or 'object.fun()'
	ChangeType (Box<Expr>, Type),         // type coersing
	TSelf,
	Null
}

pub const NOP   : u8 = 0; // not operation
pub const IROP  : u8 = 1; // int real op
pub const IOP   : u8 = 2; // int op
pub const ROP   : u8 = 3; // real op
pub const AOP   : u8 = 4; // all op
pub const BOP   : u8 = 5; // bool op
pub const IROPB : u8 = 6; // int real op => bool

#[derive(Clone)]
pub struct Expr {
	pub val     : EVal,
	pub kind    : Type,
	pub addres  : Cursor,
	pub op_flag : u8      // field for regress type calculation
}

impl Show for Expr {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ')
		}
		let tp = format!(":{:?}",self.kind);
		match self.val {
			EVal::Int(ref a)  => vec![format!("{}{}{}",tab,a,tp)],
			EVal::Real(ref a) => vec![format!("{}{}{}",tab,a,tp)],
			EVal::Str(ref a)  => vec![format!("{}\"{}\"{}",tab,a,tp)],
			EVal::Char(ref a) => vec![format!("{}\'{}\'{}",tab,a,tp)],
			EVal::Bool(ref a) => vec![format!("{}{}{}",tab,a,tp)],
			EVal::Call(_,ref fun,ref args) => {
				let mut res = vec![format!("{}CALL{}", tab, tp)/*, format!("{}FUN", tab)*/];
				for line in fun.show(layer + 1) {
					res.push(line);
				}
				//res.push(format!("{}ARGS", tab));
				for arg in args.iter() {
					for line in arg.show(layer + 1) {
						res.push(line);
					}
				}
				res
			},
			EVal::Item(ref a, ref i) => {
				let mut res = vec![format!("{}ITEM{}", tab, tp)];
				for line in a.show(layer + 1) {
					res.push(line);
				}
				for line in i.show(layer + 1) {
					res.push(line);
				}
				res
			},
			EVal::Var(ref p, ref a) => {
				let mut pref = String::new();
				if p.len() > 0 {
					for s in p.iter() {
						pref.push_str(&*s);
						pref.push_str("::");
					}
				} else {
					pref.push_str("_::")
				}
				vec![format!("{}{}{}{}",tab,pref,a,tp)]
			},
			EVal::Arr(ref v) => {
				let mut res = vec![format!("{}ARR{}", tab, tp)];
				for item in v.iter() {
					for line in item.show(layer + 1) {
						res.push(line)
					}
				}
				res
			},
			EVal::Asc(ref v) => {
				let mut res = vec![format!("{}ASC{}", tab, tp)];
				for item in v.iter() {
					res.push(format!("{}PAIR", tab));
					for line in item.a.show(layer + 1) {
						res.push(line)
					}
					for line in item.b.show(layer + 1) {
						res.push(line)
					}
				}
				res
			},
			EVal::NewClass(_, ref p, ref n, ref a) => {
				let mut pref = String::new();
				if p.len() == 0 {
					pref.push_str("_::")
				} else {
					for n in p.iter() {
						pref.push_str(&*n);
						pref.push_str("::");
					}
				}
				let mut res = vec![format!("{}NEWC {}{}{}", tab, pref, n, tp)];
				for arg in a.iter() {
					for line in arg.show(layer + 1) {
						res.push(line)
					}
				}
				res
			},
			EVal::Attr(ref obj, ref fld, ref is_m) => {
				let mut res = vec![format!("{}ATTR {}{}({})", tab, fld, tp, if *is_m {"meth"} else {"prop"})];
				for line in obj.show(layer + 1) {
					res.push(line)
				}
				res
			},
			EVal::ChangeType(ref obj, ref tp) => {
				let mut res = vec![format!("{}CHTP {:?}", tab, tp)];
				for line in obj.show(layer + 1) {
					res.push(line)
				}
				res
			},
			EVal::TSelf => vec![format!("{}self{}", tab, tp)],
			EVal::Null => vec![format!("{}null{}", tab, tp)]
		}
	}
}

macro_rules! expr {
	($v:expr, $addr:expr, $k:expr) => {Expr{val : $v, kind : $k,        addres : $addr, op_flag : 0}};
	($v:expr, $addr:expr)          => {Expr{val : $v, kind : Type::Unk, addres : $addr, op_flag : 0}};
}

// search for prefix part of id : ModA::ModB::var => [ModA::ModB::]
fn parse_prefix(lexer : &Lexer, curs : &Cursor) -> Option<SynAns<Vec<String>>> {
	let mut acc = vec![];
	let mut curs = curs.clone();
	macro_rules! finalize { () => {{
		if acc.len() > 0 {
			return Some(SynAns{val : acc, cursor : curs});
		} else {
			return None;
		}
	}};}
	loop {
		match lexer.lex(&curs) {
			Ok(ans) => {
				let pack;
				if ans.kind == LexTP::Id {
					pack = ans.val;
				} else {
					finalize!()
				}
				match lexer.lex(&ans.cursor) {
					Ok(ans) => {
						if ans.val == "::" {
							acc.push(pack);
							curs = ans.cursor;
						} else {
							finalize!()
						}
					},
					_ => finalize!()
				}
			},
			_ => finalize!()
		}
	}
}

/* getting single elems like '1', 'a' or '[a,b,c]'
   and getting single elems with post modifs, like
   'fun(a,b,c)' or 'a.b' or 'arr[index]' and application of this elems.
   this fun can parse this composition:
   		[a,b,c][2].asc['key'](a,b,c).len()
   but, it can't parse this:
   		a + b
*/
fn parse_operand(lexer : &Lexer, curs : &Cursor) -> SynRes<Expr> {
	let mut curs : Cursor = curs.clone();
	let ans = lex!(lexer, &curs);
	let mut obj;
	//println!("PARSE OPER");
	match parse_prefix(&lexer, &curs) {
		Some(pans) => {
			let prefix = pans.val;
			curs = pans.cursor;
			let id = lex_type!(lexer, &curs, LexTP::Id);
			obj = expr!(EVal::Var(prefix, id.val), curs);
			curs = id.cursor;
		},
		None =>
			match ans.kind {
				LexTP::Br if ans.val == "(" => {
					let expr = try!(parse_expr(lexer, &ans.cursor));
					curs = lex!(lexer, &expr.cursor, ")");
					obj = expr.val;
				},
				LexTP::Int  => {
					//println!("INT");
					//println!("VAL: '{}'", ans.val);
					obj = expr!(EVal::Int(i64::from_str(&*ans.val).unwrap()), curs, Type::Int);
					curs = ans.cursor;
					//println!("INTOK");
				},
				LexTP::Real => {
					obj = expr!(EVal::Real(f64::from_str(&*ans.val).unwrap()), curs, Type::Real);
					curs = ans.cursor;
				},
				LexTP::Str  => {
					obj = expr!(EVal::Str(ans.val), curs, Type::Str);
					curs = ans.cursor;
				},
				LexTP::Char => {
					obj = expr!(EVal::Char(ans.val.chars().next().unwrap()), curs, Type::Char);
					curs = ans.cursor;
				},
				LexTP::Id if ans.val == "self" => {
					obj = expr!(EVal::TSelf, curs);
					curs = ans.cursor;
				},
				LexTP::Id if ans.val == "null" => {
					obj = expr!(EVal::Null, curs);
					curs = ans.cursor;
				},
				LexTP::Id if ans.val == "true"  => {
					obj = expr!(EVal::Bool(true), curs, Type::Bool);
					curs = ans.cursor;
				},
				LexTP::Id if ans.val == "false" => {
					obj = expr!(EVal::Bool(false), curs, Type::Bool);
					curs = ans.cursor;
				},
				LexTP::Id if ans.val == "new" => {
					let orig_c = curs;
					curs = ans.cursor;
					// PREFIX
					let pref = match parse_prefix(lexer, &curs) {
						None => vec![],
						Some(v) => {
							curs = v.cursor;
							v.val
						}
					};
					// NAME
					let ans = lex_type!(lexer, &curs, LexTP::Id);
					let name = ans.val;
					curs = ans.cursor;
					// TMPL
					let tmpl = {
						let ans = lex!(lexer, &curs);
						if ans.val == "<" {
							let tmpl = parse_list(lexer, &curs, &parse_type, "<", ">")?;
							curs = tmpl.cursor;
							Some(tmpl.val)
						} else {
							None
						}
					};
					// ARGS
					let args = try!(parse_list(lexer, &curs, &parse_expr, "(", ")"));
					curs = args.cursor;
					let args = args.val;
					obj = expr!(EVal::NewClass(tmpl,pref,name,args), orig_c);
				},
				LexTP::Id   => {
					obj  = expr!(EVal::Var(Vec::new(), ans.val), curs);
					curs = ans.cursor;
				},
				LexTP::Br if ans.val == "[" => {
					let ans = try!(parse_list(lexer, &curs, &parse_expr, "[", "]"));
					obj  = expr!(EVal::Arr(ans.val), curs);
					curs = ans.cursor;
				},
				LexTP::Br if ans.val == "{" => {
					let parser = pair_parser!(&parse_expr, &parse_expr, ":");
					let ans = try!(parse_list(lexer, &curs, &parser, "{", "}"));
					obj = expr!(EVal::Asc(ans.val), curs);
					curs = ans.cursor;
				},
				_ => syn_throw!("can't read expr", curs)
			}
	}
	// adding modifs (props, items, calls)
	// cur object in 'obj'
	// cur cursor in 'curs'
	loop {
		match lexer.lex(&curs) {
			Ok(ans) => 
				// CALL
				if ans.val == "(" {
					let args = try!(parse_list(lexer, &curs, &parse_expr, "(", ")"));
					//let opos = obj.addres.clone();
					obj = expr!(EVal::Call(None,Box::new(obj), args.val), curs);
					curs = args.cursor;
				// INDEXING
				} else if ans.val == "[" {	
					let index_ans = try!(parse_expr(lexer, &ans.cursor));
					//let opos = obj.addres.clone();
					obj = expr!(EVal::Item(Box::new(obj), Box::new(index_ans.val)), curs);
					curs = lex!(lexer,&index_ans.cursor,"]");
				// FIELD
				} else if ans.val == "." {
					let fld = lex_type!(lexer, &ans.cursor, LexTP::Id);
					obj = expr!(EVal::Attr(Box::new(obj), fld.val, false), curs);
					curs = fld.cursor;
				// TYPE COERSING
				} else if ans.val == "as" {
					let tp = try!(parse_type(lexer, &ans.cursor));
					let tpc = tp.val.clone();
					obj = expr!(EVal::ChangeType(Box::new(obj), tp.val), curs, tpc);
					curs = tp.cursor;
				} else {
					syn_ok!(obj, curs)
				},
			_ => syn_ok!(obj, curs)
		}
	}
}

static OPERS  : &'static [&'static str] = &["&&","||","<",">",">=","<=","==","!=","+","-","*","/","%","**"];
static PRIORS : &'static [u8]           = &[0,    0,  1,  1,  1,   1,   1,   1,   2,  2,  3,  3,  3,   4];

fn parse_operator(lexer : &Lexer, curs : &Cursor) -> SynRes<usize> {
	let ans = lex_type!(lexer,curs,LexTP::Opr);
	for i in 0 .. OPERS.len() {
		if ans.val == OPERS[i]
			{ syn_ok!(i, ans.cursor); }
	}
	syn_throw!("");
}

// build expr tree from list of operands and operators
fn build(seq : &mut Vec<Result<Box<Expr>,usize>>, addr : &Vec<Cursor>) -> Expr {
	/*
		build_local:
			seq : ['1', '+', '2', '-', '3', '*', '4']
			call : (seq, addr, 2, 7)
				['1', '+' ! '2', '-', '3', '*', '4' ! ] ==> call('-', 2, call('*', 3, 4))
	*/
	fn build_local(seq : &mut Vec<Result<Box<Expr>,usize>>, addr : &Vec<Cursor>, left : usize, right : usize) -> Expr {
		if right - left == 1 {
			let a = mem::replace(&mut seq[left], Err(0));
			match a {
				Ok(e) => return *e,
				_ => panic!()
			}
		} else {
			let mut min_p_ind = 0;
			let mut min_p : Option<u8> = None;
			for i in left .. right {
				match seq[i] {
					Err(ref p) =>
						match min_p {
							None => {
								min_p_ind = i;
								min_p = Some(PRIORS[*p]);
							}
							Some(p1) =>
								if p1 >= PRIORS[*p] {
									min_p_ind = i;
									min_p = Some(PRIORS[*p]);
								}
						},
					_ => ()
				}
			}
			match min_p {
				None => panic!(),
				_ => ()
			}
			let fun_id = match seq[min_p_ind] {Err(ref i) => OPERS[*i], _ => panic!()};
			let left  = build_local(seq, addr, left, min_p_ind);
			let right = build_local(seq, addr, min_p_ind + 1, right);
			let fun = expr!(EVal::Var(vec!["%opr".to_string()], fun_id.to_string()), addr[min_p_ind].clone());
			return expr!(EVal::Call(None, Box::new(fun), vec![left,right]), addr[min_p_ind].clone());
		}
	}
	let len = seq.len();
	build_local(seq, addr, 0, len)
}

// parse_operand + parse_operator
// parse seq of [expr, operator, expr, operator, expr, ...]
// and build it to tree
pub fn parse_expr(lexer : &Lexer, curs : &Cursor) -> SynRes<Expr> {
	let mut curs : Cursor = curs.clone();
	let mut acc  : Vec<Result<Box<Expr>,usize>> = vec![]; // seq of src [expr, operator, ...]
	let mut addr : Vec<Cursor> = vec![]; // cursors for all items in 'acc'
	macro_rules! finalize{() => {{
		let res = build(&mut acc, &addr);
		syn_ok!(res, curs);
	}};}
	loop {
		let obj = parse_operand(lexer, &curs)?;
		/* let ans = lex!(lexer, &curs);
		let obj;
		// CHECK FOR '(a + b) or (a)'
		if ans.val == "(" {
			curs = ans.cursor;
			let ans = try!(parse_expr(lexer, &curs));
			addr.push(curs);
			curs = lex!(lexer, &ans.cursor, ")");
			obj = Box::new(ans.val);
		} else {
			let ans = try!(parse_operand(lexer, &curs));
			obj = Box::new(ans.val);
			addr.push(curs);
			curs = ans.cursor;
		} */
		// operand found
		acc.push(Ok(Box::new(obj.val)));
		addr.push(curs);
		curs = obj.cursor;
		// check for operator
		// if not exist then build what was parsed
		match parse_operator(lexer, &curs) {
			Err(_) => finalize!(),
			Ok(ans) => {
				acc.push(Err(ans.val));
				addr.push(curs);
				curs = ans.cursor;
			}
		}
	}
}
