use syn::reserr::*;
use syn::type_sys::*;
use std::fmt;

pub struct DefExcept {
    pub name : String,
    pub arg  : Option<RType>,
    pub addr : Cursor
}

pub fn parse_except(lexer : &Lexer, curs : &Cursor) -> SynRes<DefExcept> {
    let curs = lex!(lexer, &curs, "exception");
    let name = lex_type!(lexer, &curs, LexTP::Id);
    let ans = lex!(lexer, &name.cursor);
    if ans.val == ":" {
        let tp = parse_type(lexer, &ans.cursor)?;
        let curs = lex!(lexer, &tp.cursor, ";");
        syn_ok!(DefExcept{name : name.val, arg : Some(tp.val), addr : tp.cursor}, curs);
    } else if ans.val == ";" {
        syn_ok!(DefExcept{name : name.val, arg : None, addr : curs}, ans.cursor);
    } else {
        syn_throw!(format!("excpected ':' or ';', found {}", ans.val), name.cursor);
    }
}

impl fmt::Debug for DefExcept {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        match self.arg {
            Some(ref t) => write!(f, "exception {} : {:?}", self.name, t),
            _ => write!(f, "exception {}", self.name)
        }
    }
}
