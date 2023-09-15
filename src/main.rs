mod lexer;
mod parser;
mod semantic_anal;
mod asmgen;

use std::process::exit;

use parser::Parser;

use semantic_anal::analyze;

use crate::asmgen::AsmCode;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let mut parser = Parser::from_file(path);
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
    let anal_result = analyze(&parser.program);
    let symtable = match anal_result {
        Err(err) => {
            println!("{:?}", err);
            exit(1)
        }
        Ok(symtable) => symtable,
    };
    println!("[Semantic Analysis] {:?}", symtable);
    let mut asmcode = AsmCode::default();
    asmcode.genasm(&parser.program, symtable);
    asmcode.compile(path)?;
    return Ok(());
    // asmcode.write_to_file()
}
