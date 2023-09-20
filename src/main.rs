mod codegen;
mod lexer;
mod parser;

use crate::codegen::{Asm, Env};
use lexer::Token;
use parser::{Identifier, Parser};

use std::{process::exit, rc::Rc};

#[derive(Debug)]
pub enum CompileError {
    IllegalToken(Token),
    UnexpectedToken { unexpected: Token, msg: String },
    RedeclareIdent(Identifier),
    UseOfUndeclaredIdent(Identifier),
    UseOfUninitializedIdent(Identifier),
}

impl CompileError {
    fn unexpected(unexpected: Token, msg: impl Into<String>) -> Self {
        return Self::UnexpectedToken {
            unexpected,
            msg: msg.into(),
        };
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path: Rc<str> = Rc::from(args[1].clone());
    let mut parser = Parser::from_file(path.clone());
    let res = parser.parse();
    match res {
        Err(err) => {
            println!("Error: {:?}", err);
            exit(1);
        }
        _ => {
            println!(
                "-------------------[AST]-----------------\n{}",
                parser.program
            )
        }
    }
    println!(
        "-------------------[AST]-----------------\n{}",
        parser.program
    );
    let mut asm = Asm::default();
    let mut env = Env::new();
    let res = asm.gen(&parser.program, &mut env);
    match res {
        Err(err) => {
            println!("Error: {:?}", err);
            exit(1);
        }
        _ => (),
    }
    asm.compile(path)?;
    return Ok(());
}
