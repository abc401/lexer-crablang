mod lexer;
mod parser;

use parser::Parser;

fn main() {
    let path = "code.toyl";
    let mut parser = Parser::from_file(path);
    println!("Started parsing.");
    parser.parse();
    println!("{:?}", parser.program)
}
