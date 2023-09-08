use crate::lexer::{Token, TokenType as TT};

use super::lexer::Lexer;
use core::panic;

#[derive(Debug)]
pub struct Program {
    statements: Vec<Statement>,
}

#[derive(Debug)]
enum Statement {
    Declare(String),
    Initialize(String, RExpression),
    Assign(LExpression, RExpression),
}
use Statement as Stmt;

#[derive(Debug)]
enum RExpression {
    IntLiteral(i32),
    Ident(String),
}
use RExpression as RExp;

#[derive(Debug)]
enum LExpression {
    Ident(String),
}
use LExpression as LExp;

pub struct Parser {
    lexer: Lexer,
    pub program: Program,
}

impl Parser {
    pub fn new(source: String) -> Self {
        return Self {
            lexer: Lexer::new(source),
            program: Program {
                statements: Vec::new(),
            },
        };
    }

    pub fn from_file(path: &str) -> Self {
        return Self {
            lexer: Lexer::from_file(path),
            program: Program {
                statements: Vec::new(),
            },
        };
    }

    pub fn parse(&mut self) {
        loop {
            let token = self.lexer.peek();
            match token.tokentype {
                TT::EOF => {
                    println!("[Parser] EOF");
                    break;
                }
                TT::Let => {
                    self.lexer.consume();
                    println!("[Parser] Declaration or Initialization.");
                    self.decl_or_init();
                }
                TT::Ident(_) => {
                    println!("[Parser] Assignment.");
                    self.assign();
                }
                _ => {
                    unexpected_token(&token);
                }
            }
        }
    }

    fn assign(&mut self) {
        let lexp = self.lexp();
        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::Assign) {
            unexpected_token(&token)
        }
        self.lexer.consume();
        let rexp = self.rexp();
        self.program.statements.push(Stmt::Assign(lexp, rexp))
    }

    fn rexp(&mut self) -> RExpression {
        let token = self.lexer.peek();
        let rexp = match token.tokentype {
            TT::Ident(ident) => RExp::Ident(ident),
            TT::IntLiteral(intlit) => RExp::IntLiteral(intlit),
            _ => unexpected_token(&token),
        };
        self.lexer.consume();
        return rexp;
    }

    fn lexp(&mut self) -> LExpression {
        let token = self.lexer.peek();
        let TT::Ident(ident) = token.tokentype else {
            unexpected_token(&token);
        };
        self.lexer.consume();
        return LExpression::Ident(ident);
    }

    fn decl_or_init(&mut self) {
        let token = self.lexer.peek();
        let TT::Ident(l_identifier) = token.tokentype else {
            unexpected_token(&token);
        };
        self.lexer.consume();

        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::Assign) {
            let stmt = Stmt::Declare(l_identifier);
            log_parsed(&stmt);
            println!("[Parser] Next to parse: {:?}", token);
            self.program.statements.push(stmt);
            return;
        }
        self.lexer.consume();

        let token = self.lexer.peek();
        match token.tokentype {
            TT::Ident(r_identifier) => {
                let stmt = Stmt::Initialize(l_identifier, RExp::Ident(r_identifier));
                log_parsed(&stmt);
                self.program.statements.push(stmt);
            }
            TT::IntLiteral(int_lit) => {
                let stmt = Stmt::Initialize(l_identifier, RExp::IntLiteral(int_lit));
                log_parsed(&stmt);
                self.program.statements.push(stmt);
            }
            _ => {
                unexpected_token(&token);
            }
        }
        self.lexer.consume();
    }
}

fn unexpected_token(token: &Token) -> ! {
    println!(
        "[Parser] {}:{} Unexpected token: {:?}",
        token.row, token.col, token.tokentype
    );
    panic!()
}

fn log_parsed(stmt: &Statement) {
    println!("[Parser] Parsed: {:?}", stmt);
}
