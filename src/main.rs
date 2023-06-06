mod lexer;

use lexer::Lexer;

fn main() {
    let path = "src/lang.txt";
    let lexer = Lexer::from_file(path);
    for token in lexer {
        println!("{:?}", token);
    }
}
