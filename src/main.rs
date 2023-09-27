mod codegen;
mod lexer;
mod parser;

use crate::codegen::{Asm, Env};
use lexer::{Location, Token};
use parser::{Identifier, Parser, RExp};

use std::{process::exit, rc::Rc};

#[derive(Debug)]
pub enum CompileError {
    // Lexer
    IllegalToken(Token),

    // Parser
    UnexpectedToken(Token),
    RExpOnLHS(RExp),
    ExpectedExpression(Location),
    ExpectedIdent(Location),
    ExpectedEBrace(Location),
    ExpectedECurly(Location),
    ExpectedBlock(Location),
    ExpectedNewline(Location),
    // This error is only used internally in the parser and is not intended to reach the user.
    // It is used to signify that the parser couldn't find the terminals
    // that appear at the start of the requested language construct
    NotFound,

    // Analyzer
    UndeclaredIdent(Identifier),
    UninitializedIdent(Identifier),
}

trait HandleNotFound {
    fn handle_not_found(self, err: CompileError) -> Self;
}

impl<T> HandleNotFound for Result<T, CompileError> {
    fn handle_not_found(self, err: CompileError) -> Self {
        match self {
            Err(CompileError::NotFound) => Err(err),
            res => res,
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path: Rc<str> = Rc::from(args[1].clone());
    let mut parser = Parser::from_file(path.clone());
    let res = parser.parse_program();
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
    // println!(
    //     "-------------------[AST]-----------------\n{}",
    //     parser.program
    // );
    let mut asm = Asm::default();
    let res = asm.gen(&parser.program.stmts);
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
