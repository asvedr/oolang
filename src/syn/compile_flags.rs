use syn::reserr::*;
//use syn::lexer::*;

#[derive(PartialEq,Debug)]
pub enum CompFlag {
    NoExcept,
    Inline // NOT REALIZED
}

pub fn parse_comp_flag(lexer : &Lexer, curs : &Cursor) -> SynRes<CompFlag> {
    let ans = lex!(lexer, curs);
    if ans.kind == LexTP::Hash {
        let key = lex_type!(lexer, &ans.cursor, LexTP::Id);
        if key.val == "NoExcept" {
            syn_ok!(CompFlag::NoExcept, key.cursor)
        }
        syn_throw!(format!("bad flag: {}", key.val), ans.cursor)
    }
    syn_throw!("", curs)
}
