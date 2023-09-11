mod asmgen;
mod lexer;
mod parser;
mod semantic_anal;

use std::{
    path::Path,
    process::{exit, Command},
};

use parser::Parser;

use crate::{asmgen::genasm, semantic_anal::analyze};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    println!("{:?}", args);
    let mut parser = Parser::from_file(path);
    println!("Started parsing.");
    parser.parse();
    let anal_result = analyze(&parser.program);
    let symtable = match anal_result {
        Err(err) => {
            println!("{:?}", err);
            exit(1)
        }
        Ok(symtable) => symtable,
    };
    println!("[Semantic Analysis] {:?}", symtable);
    let asmcode = genasm(&parser.program, symtable);
    asmcode.compile(path)?;
    return Ok(());
    // asmcode.write_to_file()
}
