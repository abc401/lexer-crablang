use crate::lexer::{LexingError as LErr, Location, Token, TokenType as TT};

use super::lexer::Lexer;
use std::slice::Iter;

#[derive(Debug)]
pub enum ParsingError {
    UnexpectedToken { unexpected: Token, msg: String },
    IllegalToken(Location, String),
}
impl ParsingError {
    fn unexpected(unexpected: Token, msg: impl Into<String>) -> Self {
        return Self::UnexpectedToken {
            unexpected,
            msg: msg.into(),
        };
    }
}
use ParsingError as PErr;

impl From<LErr> for PErr {
    fn from(value: LErr) -> Self {
        let LErr::IllegalToken(loc, lexeme) = value;
        return PErr::IllegalToken(loc, lexeme);
    }
}

#[derive(Debug)]
pub struct Program {
    statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct IntLiteral {
    pub loc: Location,
    pub lexeme: String,
}

impl IntLiteral {
    fn new(loc: Location, lexeme: String) -> Self {
        return Self { loc, lexeme };
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub loc: Location,
    pub lexeme: String,
}
use Identifier as Id;

impl Identifier {
    fn new(loc: Location, lexeme: String) -> Self {
        return Self { loc, lexeme };
    }
}

impl Program {
    pub fn iter(&self) -> Iter<'_, Statement> {
        self.statements.iter()
    }
}

#[derive(Debug)]
pub enum Statement {
    Declare(Identifier),
    Initialize(Identifier, RExpression),
    Assign(LExpression, RExpression),
}
use Statement as Stmt;

#[derive(Debug)]
pub enum RExpression {
    IntLiteral(IntLiteral),
    Ident(Identifier),
}
use RExpression as RExp;

#[derive(Debug)]
pub enum LExpression {
    Ident(Identifier),
}
use LExpression as LExp;

pub struct Parser {
    lexer: Lexer,
    pub program: Program,
}

impl Parser {
    pub fn from_file(path: &str) -> Self {
        return Self {
            lexer: Lexer::from_file(path),
            program: Program {
                statements: Vec::new(),
            },
        };
    }

    pub fn parse(&mut self) -> Result<(), ParsingError> {
        loop {
            let token = self.lexer.peek();
            match token.tokentype {
                TT::StartOfFile => {
                    self.lexer.consume()?;
                    continue;
                }

                TT::EndOfFile => {
                    break;
                }
                TT::Let => {
                    self.lexer.consume()?;
                    self.decl_or_init()?;
                }
                TT::Ident(_) => {
                    self.assign()?;
                }
                _ => {
                    return Err(PErr::unexpected(
                        token,
                        "Invalid token for starting a statement",
                    ));
                }
            }
        }
        return Ok(());
    }

    fn assign(&mut self) -> Result<(), ParsingError> {
        let lexp = self.lexp()?;
        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::Assign) {
            return Ok(());
        }
        self.lexer.consume()?;
        let rexp = self.rexp()?;
        self.program.statements.push(Stmt::Assign(lexp, rexp));
        return Ok(());
    }

    fn rexp(&mut self) -> Result<RExpression, ParsingError> {
        let token = self.lexer.peek();
        let rexp = match token.tokentype {
            TT::Ident(lexeme) => RExp::Ident(Identifier::new(token.loc, lexeme)),
            TT::IntLiteral(lexeme) => RExp::IntLiteral(IntLiteral::new(token.loc, lexeme)),
            _ => {
                return Err(PErr::unexpected(
                    token,
                    "An rexpression can only be an int literal or an identifier",
                ))
            }
        };
        self.lexer.consume()?;
        return Ok(rexp);
    }

    fn lexp(&mut self) -> Result<LExpression, ParsingError> {
        let token = self.lexer.peek();
        let TT::Ident(ident) = token.tokentype else {
            return Err(PErr::unexpected(token, "An lexpression can only consist of an identifier."));
        };
        self.lexer.consume()?;
        return Ok(LExp::Ident(Identifier {
            loc: token.loc,
            lexeme: ident,
        }));
    }

    fn decl_or_init(&mut self) -> Result<(), ParsingError> {
        let token = self.lexer.peek();
        let TT::Ident(l_ident_lexeme) = token.tokentype else {
            return Err(PErr::unexpected(token, "The `let` keyword can only be followed by an identifier"));
        };
        let l_ident_loc = token.loc;
        self.lexer.consume()?;

        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::Assign) {
            let stmt = Stmt::Declare(Identifier::new(l_ident_loc, l_ident_lexeme));
            self.program.statements.push(stmt);
            return Ok(());
        }
        self.lexer.consume()?;

        let token = self.lexer.peek();
        match token.tokentype {
            TT::Ident(r_ident_lexeme) => {
                let stmt = Stmt::Initialize(
                    Id::new(l_ident_loc, l_ident_lexeme),
                    RExp::Ident(Id::new(token.loc, r_ident_lexeme)),
                );
                self.program.statements.push(stmt);
            }
            TT::IntLiteral(int_lit_lexeme) => {
                let stmt = Stmt::Initialize(
                    Id::new(l_ident_loc, l_ident_lexeme),
                    RExp::IntLiteral(IntLiteral::new(token.loc, int_lit_lexeme)),
                );
                self.program.statements.push(stmt);
            }
            _ => {
                return Err(PErr::unexpected(
                    token,
                    "An rexpression can only be an int literal or an identifier",
                ))
            }
        }
        self.lexer.consume()?;
        return Ok(());
    }
}
