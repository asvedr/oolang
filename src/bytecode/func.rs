use bytecode::cmd::*;
use syn::utils::Show;

pub struct CodeFn {
    pub args      : usize,
    pub name      : String,
    // FLAGS
    pub can_throw : bool,
    // ENV
    pub opt_i_len : u8,
    pub opt_r_len : u8,
    pub unopt_len : u8,
    pub use_std_i : bool,
    pub use_std_r : bool,
    // CODE
    pub code      : Vec<Cmd>
}

impl Show for CodeFn {
    fn show(&self, _ : usize) -> Vec<String> {
        let mut acc = vec![format!("FUNC {}, {}", self.name, self.args)];
        acc.push(format!(" CAN THROW EXCEPTION {}", self.can_throw));
        acc.push(format!(" USE STDI {} USE STDR {}", self.use_std_i, self.use_std_r));
        acc.push(format!(" INT {} REAL {} VAR {}", self.opt_i_len, self.opt_r_len, self.unopt_len));
        acc.push(" {".to_string());
        for cmd in self.code.iter() {
            let cmd : &Cmd = cmd;
            for line in cmd.show(2) {
                acc.push(line);
            }
        }
        acc.push(" }".to_string());
        acc
    }
}

