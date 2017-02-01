use std::collections::{HashMap, BTreeMap};
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{Read, Write, Result, Error, ErrorKind};
use std::fs::File;
use std::str::FromStr;

// all exception keys are the numbers in C
// must make one NAME-CODE map for all modules in application
pub struct ExcKeys {
	map : HashMap<String,usize>,
	cnt : usize
}
pub struct Prepare {
	map : BTreeMap<String,usize>,
	cnt : usize
}
pub type RExcKeys = Box<ExcKeys>;

macro_rules! make_name{($pref:expr, $name:expr, $res:expr) => {{
	for i in $pref.iter() {
		$res = format!("{}{}_", $res, i);
	}
	$res = format!("{}{}_", $res, $name);
}};}

impl ExcKeys {
	#[inline(always)]
	pub fn get(&self, pref : &Vec<String>, name : &String) -> usize {
		let mut res = String::new();
		make_name!(pref, name, res);
		match self.map.get(&res) {
			Some(a) => *a,
			_ => panic!("bad exception key: {}", name)
		}
	}
	pub fn add(&mut self, pref : &Vec<String>, name : &String) {
		let mut res = String::new();
		make_name!(pref, name, res);
		self.map.insert(res, self.cnt);
		self.cnt += 1;
	}
	pub fn new(c : usize) -> RExcKeys {
		Box::new(ExcKeys {
			cnt : c,
			map : HashMap::new()
		})
	}
	pub fn from_stream<In : Read>(input : &mut In) -> Result<RExcKeys> {
		let mut buf = String::new();
		input.read_to_string(&mut buf);
		macro_rules! err {() => { return Err(Error::new(ErrorKind::InvalidData, "ExcKeys reading stream")) }}
		let mut lines = buf.split('\n');
		let cnt = match lines.next() {
			Some(l) =>
				match usize::from_str(&*buf) {
					Ok(n) => n,
					_ => err!()
				},
			_ => err!()
		};
		let mut keys = Box::new(ExcKeys {
			map : HashMap::new(),
			cnt : cnt
		});
		keys.map.reserve(cnt);
		for _ in 0 .. cnt {
			//buf.clear();
			//input.read_line(&mut buf);
			//read_line(&mut seq, &mut buf);
			match lines.next() {
				Some(line) => {
					let split : Vec<&str> = line.split(' ').collect();
					if split.len() != 2 {
						err!()
					}
					let key = split[0].to_string();
					let val = match usize::from_str(split[1]) {
						Ok(n) => n,
						_ => err!()
					};
					keys.map.insert(key, val);
				},
				_ => err!()
			}
		}
		return Ok(keys);
	}
	pub fn from_file(path : &String) -> Result<RExcKeys> {
		let mut file = File::open(path)?;
		ExcKeys::from_stream(&mut file)
	}
}

impl Prepare {
	pub fn new(&self) -> Prepare {
		Prepare {
			map : BTreeMap::new(),
			cnt : 0
		}
	}
	pub fn add(&mut self, pref : &Vec<String>, name : &String) {
		let mut res = String::new();
		make_name!(pref, name, res);
		self.map.insert(res, self.cnt);
		self.cnt += 1;
	}
	pub fn to_stream<Out : Write>(&self, out : &mut Out) -> Result<()> {
		write!(out, "{}\n", self.cnt)?;
		for (name, n) in self.map.iter() {
			write!(out, "{} {}\n", name, n)?;
		}
		out.flush()
	}
	pub fn to_file(&self, name : &String) -> Result<()> {
		let mut file = File::create(name)?;
		self.to_stream(&mut file)
	}
	pub fn exc_keys(self) -> RExcKeys {
		let cnt = self.cnt;
		let mut keys = Box::new(ExcKeys {
			cnt : cnt,
			map : HashMap::new()
		});
		keys.map.reserve(cnt);
		for (k,v) in self.map {
			keys.map.insert(k,v);
		}
		keys
	}
}
