use std::result::*;
//use std::str::{Chars};
use std::mem;

// error for 'lex' function
#[derive(Debug)]
pub struct LexErr {
	pub line   : usize,
	pub column : usize,
	pub data   : String
}

// answer for 'lex' function
#[derive(Debug)]
pub struct LexRes {
	pub val     : String,
	pub kind    : LexTP,
	pub cursor  : Cursor
}

// 'lex' function return lexem and type of lexem
// this enum is this type
#[derive(Debug,Clone,PartialEq)]
pub enum LexTP {
	Id,			// ok q
	Str,		// ok q
	Int,		// ok q
	Real,		// ok q
	Char,		// ok q
	Br,			// ok q
	Dot,		// ok q
	//Comma,		// ok q
	//DCM,        // ok
	Opr,		// ok q
	DecType,	// ok q
	NSpace,		// ok q
}

/*
impl LexTP {
	pub fn to_string(&self) -> String {
		format!("{:?}", self)
	}
}
*/

/*
// this struct don't used in this module but using in others
#[derive(Debug,Clone)]
pub struct Lexem {
	pub val  : String,
	pub kind : LexTP
}

impl LexRes {
	pub fn compare_with(&self, lexem : &Lexem) -> bool {
		if self.kind == lexem.kind
			{ return self.val == lexem.val }
		else
			{ return false }
	}
}
*/

// return lex err
macro_rules! lexerr {
	($l:expr, $c:expr, $d:expr) => { return Err(LexErr{line : $l, column : $c, data: $d.to_string()}) };
}

#[derive(Clone,Debug)]
pub struct Cursor {
	pub line   : usize,
	pub column : usize,
	//pub module : usize
}

impl Cursor {
	pub fn new() -> Cursor {
		Cursor {
			line   : 0,
			column : 0,
			//module : 0
		}
	}
}

// Lexer::lex is target function
#[derive(Debug)]
pub struct Lexer {
	alphs  : Vec<Vec<char>>, // alphabets for specific machines
	text   : Vec<Vec<char>>  // text in which we looking for 
}

// state of lex machine. Using for stra, ida, chara, etc
#[derive(Debug)]
struct State {
	fin : bool, // flag
	err : bool, // flag
	num : u8    // state
}

// machine for choose specific lexem from text
struct Machine {
	state : State, // current state
	key   : LexTP, // name of machine
	func  : Box<Fn(char,&Lexer,&mut State) -> ()>, // looker
	reader: Option<Box<Fn(&str) -> Result<String,String>>> // finalizer for building
}

macro_rules! mach { // macro for build machine
	($fun:ident, $lt:ident) => { Machine{state : State{fin : false, err : false, num : 0}, func : Box::new($fun), key : LexTP::$lt, reader : None} };
	($fun:expr, $lt:ident, $r:expr) => { Machine{state : State{fin : false, err : false, num : 0}, func : Box::new($fun), key : LexTP::$lt, reader : Some(Box::new($r))} };
}

#[allow(mutable_transmutes)]
impl Lexer {
	// create lexer from text
	pub fn new(src : &str) -> Lexer {
		// alphabets
		let ops     : Vec<char> = ("+-*/=<>?!\\@%$^&#").chars().collect();
		let sings   : Vec<char> = (".,;").chars().collect();
		let brs     : Vec<char> = ("()[]{}").chars().collect();
		let slashed : Vec<char> = ("nt\\'\"").chars().collect();
		// splitting for [[char]]
		let mut acc = vec![];
		let mut cur = vec![];
		for c in src.chars() {
			if c == '\n' {
				acc.push(cur);
				cur = vec![];
			} else {
				cur.push(c);
			}
		}
		acc.push(cur);
		Lexer {
			alphs : vec![ops, sings, brs, slashed],
			text : acc
		}
	}
	pub fn lex(&self, curs : &Cursor) -> Result<LexRes,LexErr> {
		let mut line   = curs.line;
		let mut column = curs.column;
		let mut p_line; // line on prev step
		let mut p_column; // column on prev step
		let mut comm_count = 0; // for '/*'
		let mut comm_line  = false; // for '//'
		let mut prev_slash = false; // flag for symbol '\' enter for comment
		let mut prev_star  = false; // flag for symbol '*' exit for comment
		let machines : Vec<Machine> = vec![
			mach!(id, Id), mach!(numi, Int), mach!(numr, Real), mach!(stra, Str, read_str), mach!(chara, Char, read_char),
			mach!(br, Br), mach!(opr, Opr), mach!(dectp, DecType), mach!(namesp, NSpace),
			mach!(dot, Dot)//, mach!(comma, Comma)
		];
		let mut any_on = false; // flag for any machine is activated
		let mut last_true = None; // last machine which was in final state
		let mut acc = String::new(); // word is here
		macro_rules! finalizer {() => {{
			match last_true {
				None => lexerr!(line, column, format!("from line:{} column:{} bad lexem:'{}'", curs.line, curs.column, acc)),
				Some(i) => {
					let mach : &Machine = &machines[i];
					let res = match mach.reader {
						Some(ref f) =>
							match f(&*acc) {
								Ok(res) => res,
								Err(err) => lexerr!(curs.line, curs.column, err)
							},
						_ => acc
					};
					let curs = Cursor{line : p_line, column : p_column, /*module : curs.module*/};
					return Ok(LexRes{val : res, kind : mach.key.clone(), cursor : curs})
				}
			}
		}};}
		loop {
			let mut sym; // current symbol
			p_line = line;
			p_column = column;
			// checing for EOL and EOF
			if line >= self.text.len() {
				break;
			} else if column >= self.text[line].len() {
				sym = '\n';
			} else {
				sym = self.text[line][column];
			}
			// turn line/column if EOL found
			if sym == '\n' {
				line += 1;
				column = 0;
				if comm_line {
					comm_line = false;
				}
				sym = ' '
			} else {
				column += 1;
			}
			// comment sys
			if sym == '/' && prev_slash {
				comm_line = true;
			} else if sym == '*' && prev_slash && !comm_line {
				comm_count += 1;
			} else if sym == '/' && prev_star && !comm_count > 0 {
				comm_count -= 1;
				sym = ' ';
			}
			if sym == '/' {
				prev_slash = true;
				prev_star  = false;
			} else if sym == '*' {
				prev_star  = true;
				prev_slash = false;
			} else {
				prev_star  = false;
				prev_slash = false;
			}
			if sym == '\t' || comm_line || comm_count > 0
				{sym = ' '}
			// lexers
			if sym == ' ' && !any_on {
				continue
			} else {
				// checking machines for new symbol
				let mut first_fin = None; // first machine which final on current symbol
				let mut all_err = true; // flag for all machines fail
				for i in 0 .. machines.len() {
					let f = &*machines[i].func;
					if !machines[i].state.err {
						unsafe {f(sym, &self, mem::transmute(&machines[i].state))};
						if !machines[i].state.err {
							if machines[i].state.fin {
								match first_fin {
									None => first_fin = Some(i),
									_    => ()
								}
							}
							all_err = false;
						}
					}
				}
				match first_fin {
					Some(i) => last_true = Some(i),
					_ => ()
				}
				if all_err {
					finalizer!()
				} else {
					any_on = true;
					acc.push(sym);
				}
			}
		}
		// this block for EOF.
		// if any machines was activated before EOF then we try to finalize
		// else throw 'EOF' error
		if any_on
			{finalizer!()}
		else
			{lexerr!(line, column, "EOF")}
	}
}

macro_rules! islow  { ($c:expr) => {($c >= 'a' && $c <= 'z')}; }
macro_rules! ishigh { ($c:expr) => {($c >= 'A' && $c <= 'Z')}; }
macro_rules! islit  { ($c:expr) => {(islow!($c) || ishigh!($c) || $c == '_')}; }
macro_rules! isnum  { ($c:expr) => {($c >= '0' && $c <= '9')}; }
macro_rules! sstate { ($s:expr, $f:expr, $n:expr) => {{$s.fin = $f; $s.num = $n}};
                      ($s:expr) => {$s.err = true}; }
macro_rules! cond   { ($s:expr, $c:expr, $a:expr) => {{
							if $c { $a }
							else { sstate!($s)};
					  }};
                      ($s:expr, $c:expr, $a:expr, $e:expr) => {{
							if $c { $a }
							else { $e }
					  }};
					}

#[allow(unused_parens)]
fn id(c : char, _ : &Lexer, s : &mut State) {
	cond!(s, s.num == 0,
		cond!(s, islit!(c), sstate!(s, true, 1)),
		cond!(s, islit!(c) || isnum!(c), sstate!(s, true, 1))
	);
}

#[allow(unused_parens)]
fn numi(c : char, _ : &Lexer, s : &mut State) {
	cond!(s, isnum!(c), sstate!(s, true, 1))
}

#[allow(unused_parens)]
fn numr(c : char, _ : &Lexer, s : &mut State) {
	match s.num {
		0 if isnum!(c) => sstate!(s, false, 1),
		1 if isnum!(c) => sstate!(s, false, 1),
		1 if c == '.'  => sstate!(s, false, 2),
		2 if isnum!(c) => sstate!(s, true, 2),
		_ => sstate!(s)
	}
}

#[allow(unused_parens)]
fn opr(c : char, lexer : &Lexer, s : &mut State) {
	cond!(s, is_in(c, &lexer.alphs[0]), sstate!(s, true, 0))
}

#[allow(unused_parens)]
fn br(c : char, lexer : &Lexer, s : &mut State) {
	cond!(s, s.num == 0 && is_in(c, &lexer.alphs[2]), sstate!(s,true,1))
}

#[allow(unused_parens)]
fn dot(c : char, _ : &Lexer, s : &mut State) {
	cond!(s, s.num == 0 && (c == '.' || c == ',' || c == ';'), sstate!(s,true,1))
}
/*
#[allow(unused_parens)]
fn comma(c : char, _ : &Lexer, s : &mut State) {
	cond!(s, s.num == 0 && c == ',', sstate!(s,true,1))
}

#[allow(unused_parens)]
fn dcm(c : char, _ : &Lexer, s : &mut State) {
	cond!(s, s.num == 0 && c == )
}
*/

#[allow(unused_parens)]
fn dectp(c : char, _ : &Lexer, s : &mut State) {
	cond!(s, s.num == 0 && c == ':', sstate!(s,true,1))
}

#[allow(unused_parens)]
fn namesp(c : char, _ : &Lexer, s : &mut State) {
	cond!(s, s.num == 0 && c == ':',
			sstate!(s,false,1),
			cond!(s, s.num == 1 && c == ':', sstate!(s,true,2))
		)
}

#[allow(unused_parens)]
fn chara(c : char, lexer : &Lexer, s : &mut State) {
	match s.num {
		0 if c == '\'' => sstate!(s,false,1),
		1 if c == '\\' => sstate!(s,false,10),
		1 if c == '\'' => sstate!(s),
		1 => sstate!(s,false,2),
		2 if c == '\'' => sstate!(s,true,3),
		10 if is_in(c, &lexer.alphs[3]) => sstate!(s,false,2),
		10 if isnum!(c) => sstate!(s,false,11),
		11 if isnum!(c) => sstate!(s,false,12),
		12 if isnum!(c) => sstate!(s,false,2),
		_ => sstate!(s)
	}
}

#[allow(unused_parens)]
fn stra(c : char, lexer : &Lexer, s : &mut State) {
	match s.num {
		0 if c == '"' => sstate!(s,false,1),
		1 if c == '"' => sstate!(s,true,100),
		1 if c == '\\' => sstate!(s,false,10),
		1 => sstate!(s,false,1),
		10 if is_in(c, &lexer.alphs[3]) => sstate!(s,false,1),
		10 if isnum!(c) => sstate!(s,false,11),
		11 if isnum!(c) => sstate!(s,false,12),
		12 if isnum!(c) => sstate!(s,false,1),
		_ => sstate!(s)
	}
}

fn is_in(c : char, v : &Vec<char>) -> bool {
	for c1 in v.iter() {
		if c == *c1
			{ return true }
	}
	return false;
}

fn read_char(s : &str) -> Result<String,String> {
	let chrs : Vec<char> = s.chars().collect();
	if chrs[1] == '\\' {
		match chrs[2] {
			'\\' => Ok(("\\").to_string()),
			't'  => Ok(("\t").to_string()),
			'n'  => Ok(("\n").to_string()),
			'\'' => Ok(("'").to_string()),
			'\"' => Ok(("\"").to_string()),
			_    => {
					let i0 = chrs[4] as usize - 48;
					let i1 = chrs[3] as usize - 48;
					let i2 = chrs[2] as usize - 48;
					let sum = i2 * 100 + i1 * 10 + i0;
					if sum > 255
						{ return Err(format!("Bad char: {}", sum)) }
					else
						{ return Ok(((sum as u8) as char).to_string()) }
				}
		}
	} else {
		Ok(chrs[1].to_string())
	}
}

fn read_str(s : &str) -> Result<String,String> {
	let mut acc = String::new();
	let mut estate = 0;
	let mut sum : usize = 0;
	macro_rules! push0 { ($c:expr) => {{acc.push($c); estate = 0}} }
	let chars : Vec<char> = s.chars().collect();
	for i in 1 .. chars.len() - 1 {
		let c = chars[i];
		match estate {
			0 if c == '\\' => estate = 1,
			0 => acc.push(c),
			1 if c == '\\' => push0!('\\'),
			1 if c == 'n'  => push0!('\n'),
			1 if c == 't'  => push0!('\t'),
			1 if c == '\'' => push0!('\''),
			1 if c == '\"' => push0!('\"'),
			1 => { // num
				sum = (c as usize - 48) * 100;
				estate = 2;
			},
			2 => {
				sum += (c as usize - 48) * 10;
				estate = 3;
			},
			3 => {
				sum += c as usize - 48;
				if sum > 255
					{ return Err(format!("Bad char: {}", sum)) }
				else
					{ acc.push((sum as u8) as char) }
				estate = 0
			},
			_ => unreachable!()
		}
	}
	Ok(acc)
}

