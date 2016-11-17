//use syn_expr::*;
//use syn_act::*;
use syn_utils::*;
//use type_sys::*;
use syn_reserr::*;
use syn_fn::*;
use syn_class::*;
use syn_ext_c::*;
use std::fmt;

pub struct Import {
	pub path   : Vec<String>,
	pub alias  : Option<String>
//	pub getall : bool
}

impl fmt::Debug for Import {
	fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
		for n in self.path.iter() {
			try!(write!(f, "{}::", n));
		}
		match self.alias {
			Some(ref a) => write!(f, ". as {}", a),
			_ => write!(f, "*")
		}
	}
}

impl Import {
	pub fn get_all(&self) -> bool {
		match self.alias {
			None => true,
			_ => false
		}
	}
}

pub struct SynMod {
	pub imports : Vec<Import>,
	pub funs    : Vec<SynFn>,
	pub classes : Vec<Class>,
	pub c_fns   : Vec<CFun>,
	pub c_types : Vec<CType>
}

impl Show for SynMod {
	fn show(&self, layer : usize) -> Vec<String> {
		let mut tab = String::new();
		for _ in 0 .. layer {
			tab.push(' ');
		}
		let mut res = vec![format!("{}IMPORTS", tab)];
		for imp in self.imports.iter() {
			if imp.get_all() {
				res.push(format!("{} => {:?} all", tab, imp.path));
			} else {
				res.push(format!("{} => {:?} as {}", tab, imp.path, imp.alias.as_ref().unwrap()));
			}
		}
		res.push(format!("{}EXTERN", tab));
		for f in self.c_types.iter() {
			res.push(format!("ctype {}", f));
		}
		for f in self.c_fns.iter() {
			res.push(format!("{:?}", f));
		}
		res.push(format!("{}CLASSES", tab));
		for cls in self.classes.iter() {
			for line in cls.show(layer + 1) {
				res.push(line);
			}
		}
		res.push(format!("{}FUNS", tab));
		for fun in self.funs.iter() {
			for line in fun.show(layer + 1) {
				res.push(line);
			}
		}
		res.push(format!("{}END", tab));
		return res;
	}
}

pub fn parse_mod(lexer : &Lexer) -> Result<SynMod,Vec<SynErr>> {
	let curs = Cursor::new();
	match parse_mod_syn(lexer, &curs) {
		Ok(ans) => Ok(ans.val),
		Err(e)  => Err(e)
	}
}

fn parse_mod_syn(lexer : &Lexer, curs : &Cursor) -> SynRes<SynMod> {
	let mut imps = vec![];
	let mut funs = vec![];
	let mut clss = vec![];
	let mut cfns = vec![];
	let mut ctps = vec![];

	let mut curs = curs.clone();

	loop {
		match lexer.lex(&curs) {
			Err(_) =>
				syn_ok!(SynMod{imports : imps, funs : funs, classes : clss, c_fns : cfns, c_types : ctps}, curs),
			Ok(ans) =>
				match &*ans.val {
					"class"  => {
						let cls = try!(parse_class(lexer, &curs));
						clss.push(cls.val);
						curs = cls.cursor;
						//println!("AFTER CLASS: {:?}", curs);
					},
					"use"    => {
						let imp = try!(parse_import(lexer, &curs));
						imps.push(imp.val);
						curs = imp.cursor;
					}
					"fn"     => {
						let fnc = try!(parse_fn_full(lexer, &curs));
						funs.push(fnc.val);
						curs = fnc.cursor;
					},
					"extern" => {
						let sym = lex!(lexer, &ans.cursor);
						if sym.val == "fn" {
							let cfun = try!(parse_c_fn(lexer, &ans.cursor));
							cfns.push(cfun.val);
							curs = cfun.cursor;
						} else if sym.val == "type" {
							let ctype = try!(parse_c_type(lexer, &ans.cursor));
							ctps.push(ctype.val);
							curs = ctype.cursor;
						} else {
							syn_throw!(format!("after 'extern' must be 'fn' or 'type', found '{}'", sym.val), curs);
						}
					},
					"c_type" => panic!(),
					"c_func" => panic!(),
					_ => syn_throw!(format!("unexpected expression on toplevel: '{}'", ans.val), curs)
				}
		}
	}
}

fn parse_import(lexer : &Lexer, curs : &Cursor) -> SynRes<Import> {
	let mut curs = lex!(lexer, curs, "use");
	let mut acc = vec![];
	loop {
		let name = lex!(lexer, &curs);
		if name.val == "*" {
			if acc.len() == 0 {
				syn_throw!(format!("before '*' must be a pack"), curs);
			} else {
				curs = lex!(lexer, &name.cursor, ";");
				syn_ok!(Import{path : acc, alias : None/*, getall : true*/}, curs);
			}
		} else if name.kind == LexTP::Id {
			let ans = lex!(lexer, &name.cursor);
			if !is_high(&*name.val) {
				syn_throw!(format!("pack name must start with high: {}", name.val), curs);
			}
			curs = ans.cursor;
			acc.push(name.val.clone());
			match &*ans.val {
				"::" => {},
				"as" => {
					let alias = lex_type!(lexer, &curs, LexTP::Id);
					if !is_high(&*alias.val) {
						syn_throw!(format!("pack name must start with high: {}", alias.val), curs);
					} else {
						curs = lex!(lexer, &alias.cursor, ";");
						let res = Import{path : acc, alias : Some(alias.val)/*, getall : false*/};
						syn_ok!(res, curs);
					}
				},
				";" => {
					let res = Import{path : acc, alias : Some(name.val.clone())/*, getall : false*/};
					syn_ok!(res, curs);
				},
				_ => syn_throw!(format!("expected "), name.cursor)
			}
		} else {
			syn_throw!(format!("expected pack name or '*'"), curs);
		}
	}
}

#[inline(always)]
fn is_high(s : &str) -> bool {
	match s.chars().next() {
		Some(c) => c >= 'A' && c <= 'Z',
		_ => false
	}
}
