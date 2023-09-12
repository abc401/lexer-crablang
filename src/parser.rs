use crate::lexer::{LexingError as LErr, Location, Token, TokenType as TT};

use super::lexer::Lexer;
use std::{fmt::Display, slice::Iter};

#[derive(Debug)]
pub enum ParsingError {
    UnexpectedToken { unexpected: Token, msg: String },
    IllegalToken(Token),
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
        let LErr::IllegalToken(token) = value;
        return PErr::IllegalToken(token);
    }
}

#[derive(Debug)]
pub struct Program {
    statements: Vec<Statement>,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program {{\n")?;
        for stmt in self.statements.iter() {
            write!(f, "\t{:?}\n", stmt)?;
            write!(f, "---------------------------------------------------\n")?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug)]
pub struct IntLiteral {
    pub file: Option<String>,
    pub start: Location,
    pub end: Location,
    pub lexeme: String,
}
use IntLiteral as ILit;

impl IntLiteral {
    fn new(token: Token) -> Self {
        let TT::IntLiteral(lexeme) = token.tokentype else {
            panic!("Non integer literal token passed to `IntLiteral` constructor.");
        };
        return Self {
            file: token.file,
            start: token.start,
            end: token.end,
            lexeme,
        };
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub file: Option<String>,
    pub start: Location,
    pub end: Location,
    pub lexeme: String,
}
use Identifier as Id;

impl Identifier {
    fn new(token: Token) -> Self {
        let TT::Ident(lexeme) = token.tokentype else {
            panic!("Non-identifier token passed to `Identifier` constructor.");
        };
        return Self {
            file: token.file,
            start: token.start,
            end: token.end,
            lexeme,
        };
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
    RExp(RExpression),
}
use Statement as Stmt;

#[derive(Debug)]
pub enum RExpression {
    LExp(LExpression),
    IntLiteral(IntLiteral),
}
use RExpression as RExp;

impl From<LExp> for RExp {
    fn from(value: LExp) -> Self {
        return RExp::LExp(value);
    }
}

#[derive(Debug)]
pub enum LExpression {
    Ident(Identifier),
}
use LExpression as LExp;
impl TryFrom<RExpression> for LExpression {
    type Error = ();
    fn try_from(value: RExpression) -> Result<Self, Self::Error> {
        return match value {
            RExp::LExp(lexp) => Ok(lexp),
            _ => Err(()),
        };
    }
}

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
                TT::StartOfFile | TT::NewLine => {
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
                TT::Ident(_) | TT::IntLiteral(_) => {
                    self.assign_stmt_or_rexp()?;
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

    fn assign(&mut self, errmsg: impl Into<String>) -> Result<(), ParsingError> {
        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::Assign) {
            return Err(PErr::unexpected(token, errmsg));
        }
        self.lexer.consume()?;
        return Ok(());
    }

    fn newline(&mut self, errmsg: impl Into<String>) -> Result<(), ParsingError> {
        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::NewLine) {
            return Err(PErr::unexpected(token, errmsg));
        }
        self.lexer.consume()?;
        return Ok(());
    }

    fn assign_stmt_or_rexp(&mut self) -> Result<(), ParsingError> {
        let exp = self.rexp("");
        if exp.is_ok() && self.newline("").is_ok() {
            let stmt = Stmt::RExp(exp.unwrap());
            self.program.statements.push(stmt);
            return Ok(());
        }

        let lexp = LExp::try_from(exp.unwrap());
        if lexp.is_err() {
            return Err(PErr::unexpected(
                self.lexer.peek(),
                "Expected an lexpression.",
            ));
        }
        let lexp = lexp.unwrap();

        let res = self.assign("");
        match res {
            Ok(_) => {}
            Err(_) => {
                self.newline("Expected a newline.")?;
                let stmt = Stmt::RExp(lexp.into());
                self.program.statements.push(stmt);
                return Ok(());
            }
        }

        println!("[Parser] next token: {:?}", self.lexer.peek());
        let rexp = self.rexp("What do I assign it to?")?;
        self.program.statements.push(Stmt::Assign(lexp, rexp));
        return Ok(());
    }

    fn rexp(&mut self, errmsg: impl Into<String>) -> Result<RExpression, ParsingError> {
        let lexp = self.lexp();
        if lexp.is_ok() {
            return Ok(lexp.unwrap().into());
        }
        let token = self.lexer.peek();
        let rexp = match token.tokentype {
            TT::IntLiteral(_) => RExp::IntLiteral(ILit::new(token)),
            _ => return Err(PErr::unexpected(token, errmsg)),
        };
        self.lexer.consume()?;
        return Ok(rexp);
    }

    fn lexp(&mut self) -> Result<LExpression, ParsingError> {
        let token = self.lexer.peek();
        let TT::Ident(_) = token.tokentype else {
            return Err(PErr::unexpected(token, "An lexpression can only consist of an identifier."));
        };
        self.lexer.consume()?;
        return Ok(LExp::Ident(Identifier::new(token)));
    }

    fn ident(&mut self) -> Result<Identifier, ParsingError> {
        let token = self.lexer.peek();
        let TT::Ident(_) = token.tokentype else {

            return Err(PErr::unexpected(token, "Expected an identifier."));
        };
        self.lexer.consume()?;
        return Ok(Id::new(token));
    }

    fn decl_or_init(&mut self) -> Result<(), ParsingError> {
        let l_ident = self.ident()?;
        if self.newline("").is_ok() {
            let stmt = Stmt::Declare(l_ident);
            self.program.statements.push(stmt);
            return Ok(());
        }

        self.assign("Expected an `=` sign.")?;

        let rexp = self.rexp("What do I initialize it to?")?;

        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::NewLine) {
            return Err(PErr::unexpected(token, "Expected a newline."));
        }
        self.lexer.consume()?;

        let stmt = Stmt::Initialize(l_ident, rexp);
        self.program.statements.push(stmt);

        return Ok(());
    }
}
