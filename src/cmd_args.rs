use getopts::*;
use std::env;
use std::result::Result as StdRes;

//pub struct 

const SRC_TP       : &'static str = "code";
const BYTE_TEXT_TP : &'static str = "bt";
const BYTE_BIN_TP  : &'static str = "bb";
const C_TP         : &'static str = "c";  // c source
const OBJ_TP       : &'static str = "o";  // c obj file
const LIB_TP       : &'static str = "so"; // dll
const EXE_TP1      : &'static str = "";
const EXE_TP2      : &'static str = "exe";

#[derive(Debug)]
pub enum FType {
	Exe,
	ByteText,
	ByteBin,
	C,
	CObj,
	DLL,
	Source
}

impl FType {
	pub fn to_str(&self) -> &'static str {
		match *self {
			FType::Exe      => EXE_TP1,
			FType::Source   => SRC_TP,
			FType::ByteText => BYTE_TEXT_TP,
			FType::ByteBin  => BYTE_BIN_TP,
			FType::C        => C_TP,
			FType::CObj     => OBJ_TP,
			FType::DLL      => LIB_TP
		}
	}
}

pub struct File {
	pub path  : String,
	pub ftype : FType 
}

pub struct Quest {
	pub input    : Vec<File>,
	pub output   : File
}

pub fn parse() -> StdRes<Quest, String> {
	let mut opts = Options::new();
	opts.optopt("o", "", "output name and format. Default is \"<srcname>.exe\"", "NAME");
	opts.optflag("h", "help", "this message");
	opts.optopt("C", "c_dir", "include directory with .c or .o files", "DIR");
	opts.optopt("c", "", "include .c file or .o file", "NAME");
	let args : Vec<String> = env::args().collect();
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(m) => return Err(m.to_string())
	};
	if matches.opt_present("h") {
		let vi = vec![SRC_TP, BYTE_TEXT_TP, BYTE_BIN_TP, C_TP, OBJ_TP, LIB_TP];
		let vo = vec![BYTE_TEXT_TP, BYTE_BIN_TP, C_TP, EXE_TP1, EXE_TP2];
		let brief = format!("help for compiler\navailable input: {:?}\navailable output: {:?}", vi, vo);
		return Err(opts.usage(&brief));
	}
	if matches.free.is_empty() {
		return Err(format!("no input files"));
	}
	let mut inp = vec![];
	for i in matches.free.iter() {
		let tp = match file_type(i) {
			Ok(a) => a,
			Err(a) => return Err(format!("bad filetype: {}", a))
		};
		inp.push(File{path : i.clone(), ftype : tp})
	}
	let out = match matches.opt_str("o") {
		Some(val) => {
			let tp = match file_type(&val) {
				Ok(a) => a,
				Err(a) => return Err(format!("bad filetype: {}", a))
			};
			File{path : val, ftype : tp}
		},
		_ => {
			let val = &inp[0].path;
			File{path : format!("{}/{}.{}", file_dir(val), file_name(val), FType::Exe.to_str()), ftype : FType::Exe}
		}
	};
	Ok(Quest{
		input  : inp,
		output : out
	})

}

fn file_dir(path : &String) -> String {
	let text : Vec<char> = path.chars().collect();
	let mut dir = String::new();
	let mut slash = 0;
	for i in (0 .. path.len()).rev() {
		if text[i] == '/' {
			slash = i;
			break;
		}
	}
	for i in 0 .. slash {
		dir.push(text[i]);
	}
	return dir;
}

fn file_name(path : &String) -> String {
	let text : Vec<char> = path.chars().collect();
	let mut name = String::new();
	let mut dot = 0;
	let mut dot_found = false;
	let mut slash = 0;
	for i in (0 .. text.len()).rev() {
		if !dot_found && text[i] == '.' {
			dot = i;
			dot_found = true;
		}
		if text[i] == '/' {
			slash = i;
			break;
		}
	}
	macro_rules! make{($a:expr, $b:expr) => {{
		for i in $a .. $b {
			name.push(text[i]);
		}
		return name;
	}};}
	if dot_found {
		make!(slash, dot);
	} else {
		make!(slash, text.len());
	}
}

fn file_type(path : &String) -> StdRes<FType,String> {
	let name : Vec<char> = path.chars().collect();
	let mut ftp = String::new();
	let mut i = name.len() - 1;
	while i >= 0 {
		if name[i] == '.' {
			for j in i+1 .. name.len() {
				ftp.push(name[j]);
			}
			if ftp == SRC_TP {
				return Ok(FType::Source)
			} else if ftp == BYTE_TEXT_TP {
				return Ok(FType::ByteText)
			} else if ftp == BYTE_BIN_TP {
				return Ok(FType::ByteBin)
			} else if ftp == C_TP {
				return Ok(FType::C)
			} else if ftp == OBJ_TP {
				return Ok(FType::CObj)
			} else if ftp == LIB_TP {
				return Ok(FType::DLL)
			} else if ftp == EXE_TP1 {
				return Ok(FType::Exe)
			} else if ftp == EXE_TP2 {
				return Ok(FType::Exe)
			} else {
				return Err(ftp)
			}
		}
		if name[i] == '/' {
			return Ok(FType::Exe);
		}
		i -= 1;
	}
	return Ok(FType::Exe);
}
