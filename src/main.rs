mod asmgen;
mod lexer;
mod parser;
mod semantic_anal;

use crate::asmgen::AsmCode;
use lexer::Token;
use parser::{Identifier, Parser};
use semantic_anal::analyze;

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
    let anal_result = analyze(&mut parser.program);
    let env = match anal_result {
        Err(err) => {
            println!("{:?}", err);
            exit(1)
        }
        Ok(env) => env,
    };
    println!("[Semantic Analysis] {:?}", env);
    println!(
        "-------------------[AST]-----------------\n{}",
        parser.program
    );
    let mut asmcode = AsmCode::default();
    asmcode.genasm(&parser.program, &env);
    asmcode.compile(&path)?;
    return Ok(());
}
