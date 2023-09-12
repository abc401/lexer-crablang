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
impl From<Token> for IntLiteral {
    fn from(value: Token) -> Self {
        let TT::IntLiteral(lexeme) = value.tokentype else {
            panic!("Non integer literal token passed to `IntLiteral` constructor.");
        };
        return Self {
            file: value.file,
            start: value.start,
            end: value.end,
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

impl From<Token> for Identifier {
    fn from(value: Token) -> Self {
        let TT::Ident(lexeme) = value.tokentype else {
            panic!("Non-identifier token passed to `Identifier` constructor.");
        };
        return Self {
            file: value.file,
            start: value.start,
            end: value.end,
            lexeme,
        };
    }
}

#[derive(Debug)]
pub enum Term {
    Ident(Identifier),
    IntLit(IntLiteral),
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
    BinExp(BinExp),
}

#[derive(Debug)]
pub enum BinExp {
    Term(Term),
    Add(Box<BinExp>, Term),
}
impl From<Term> for BinExp {
    fn from(value: Term) -> Self {
        return BinExp::Term(value);
    }
}
use RExpression as RExp;

impl From<LExpression> for RExpression {
    fn from(value: LExp) -> Self {
        return RExp::LExp(value);
    }
}

impl From<Term> for RExpression {
    fn from(value: Term) -> Self {
        match value {
            Term::Ident(ident) => RExp::LExp(LExp::Ident(ident)),
            Term::IntLit(intlit) => RExp::IntLiteral(intlit),
        }
    }
}

impl From<BinExp> for RExpression {
    fn from(value: BinExp) -> Self {
        match value {
            BinExp::Term(Term::Ident(ident)) => RExp::LExp(LExp::Ident(ident)),
            BinExp::Term(Term::IntLit(intlit)) => RExp::IntLiteral(intlit),
            bin_exp => RExp::BinExp(bin_exp),
        }
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
                self.newline("[assign_stmt_or_rexp] Expected a newline.")?;
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

    fn plus(&mut self, errmsg: impl Into<String>) -> Result<(), ParsingError> {
        let token = self.lexer.peek();
        if matches!(token.tokentype, TT::Plus) {
            self.lexer.consume()?;
            return Ok(());
        }
        return Err(PErr::unexpected(token, errmsg));
    }

    fn rexp(&mut self, errmsg: impl Into<String>) -> Result<RExpression, ParsingError> {
        let bin_exp = self.bin_exp(errmsg)?;
        return Ok(bin_exp.into());
    }

    fn bin_exp(&mut self, errmsg: impl Into<String>) -> Result<BinExp, ParsingError> {
        println!("Next token: {:?}", self.lexer.peek());
        let term1 = self.term(errmsg)?;
        println!("[bin_exp] found a term.");
        if self.plus("").is_err() {
            println!("Next token: {:?}", self.lexer.peek());
            println!("[bin_exp] no plus.");

            return Ok(term1.into());
        }
        let term2 = self.term("Expected a term for binary expression.")?;
        let mut bin_exp = BinExp::Add(Box::new(term1.into()), term2);
        while self.plus("").is_ok() {
            let term_next = self.term("Expected a term for binary expression.")?;
            bin_exp = BinExp::Add(Box::new(bin_exp), term_next);
        }
        return Ok(bin_exp);
    }

    fn term(&mut self, errmsg: impl Into<String>) -> Result<Term, ParsingError> {
        let token = self.lexer.peek();
        let term = match token.tokentype {
            TT::Ident(_) => Term::Ident(token.into()),
            TT::IntLiteral(_) => Term::IntLit(token.into()),
            _ => return Err(PErr::unexpected(token, errmsg)),
        };
        self.lexer.consume()?;
        return Ok(term);
    }

    fn lexp(&mut self) -> Result<LExpression, ParsingError> {
        let token = self.lexer.peek();
        let TT::Ident(_) = token.tokentype else {
            return Err(PErr::unexpected(token, "An lexpression can only consist of an identifier."));
        };
        self.lexer.consume()?;
        return Ok(LExp::Ident(token.into()));
    }

    fn ident(&mut self) -> Result<Identifier, ParsingError> {
        let token = self.lexer.peek();
        let TT::Ident(_) = token.tokentype else {

            return Err(PErr::unexpected(token, "Expected an identifier."));
        };
        self.lexer.consume()?;
        return Ok(token.into());
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

        self.newline("[decl_or_init] Expected a newline.")?;

        let stmt = Stmt::Initialize(l_ident, rexp);
        self.program.statements.push(stmt);

        return Ok(());
    }
}
