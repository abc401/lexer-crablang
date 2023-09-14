use crate::lexer::{LexingError as LErr, Location, Token, TokenType as TT};

use super::lexer::Lexer;
use std::{
    fmt::{write, Display},
    slice::Iter,
};

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

impl Display for IntLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
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

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
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
    LExp(LExpression),
    IntLit(IntLiteral),
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LExp(LExp::Ident(ident)) => write!(f, "{}", ident.lexeme),
            Self::IntLit(intlit) => write!(f, "{}", intlit.lexeme),
        }
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

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declare(ident) => write!(f, "Declare({})", ident),
            Self::Assign(lexp, rexp) => write!(f, "Assign({}, {})", lexp, rexp),
            Self::Initialize(ident, rexp) => write!(f, "Initialize({}, {})", ident, rexp),
            Self::RExp(rexp) => write!(f, "RExp({})", rexp),
        }
    }
}

#[derive(Debug)]
pub enum RExpression {
    Term(Term),
    Add(Box<RExpression>, Term),
    Sub(Box<RExpression>, Term),
}

impl Display for RExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RExp::Add(rexp, term) => {
                write!(f, "{} + {}", *rexp, term)
            }
            RExp::Sub(rexp, term) => {
                write!(f, "{} - {}", *rexp, term)
            }
            RExp::Term(term) => {
                write!(f, "{}", term)
            }
        }
    }
}

use RExpression as RExp;
impl TryInto<Term> for RExp {
    type Error = ();
    fn try_into(self) -> Result<Term, Self::Error> {
        match self {
            Self::Term(term) => Ok(term),
            _ => return Err(()),
        }
    }
}

impl From<LExpression> for RExpression {
    fn from(value: LExp) -> Self {
        return RExp::Term(Term::LExp(value));
    }
}

impl From<Term> for RExpression {
    fn from(value: Term) -> Self {
        return RExp::Term(value);
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
            RExp::Term(Term::LExp(lexp)) => Ok(lexp),
            _ => Err(()),
        };
    }
}

impl Display for LExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => write!(f, "{}", ident),
        }
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
            println!("[Parser] Parsing new statement.");
            println!("[Parser] Peek token: {:?}", token);
            if !self.program.statements.is_empty() {
                println!(
                    "[Parser] Statement: {}",
                    self.program.statements.last().unwrap()
                )
            }
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

    fn rexp(&mut self, errmsg: impl Into<String>) -> Result<RExpression, ParsingError> {
        let term1 = self.term(errmsg)?;
        println!("[Parser.rexp] term: {}", term1);
        let mut rexp = RExp::Term(term1);
        loop {
            let token = self.lexer.peek();
            match token.tokentype {
                TT::Plus => {
                    self.lexer.consume()?;
                    println!("[Parser] consumed: {:?}", token);
                    let term_next = self.term("Provide a second argument for the `+` operator.")?;
                    rexp = RExp::Add(Box::new(rexp), term_next);
                    println!("[Parser] new rexp: {}", rexp);
                }
                TT::Minus => {
                    self.lexer.consume()?;
                    println!("[Parser] consumed: {:?}", token);
                    let term_next = self.term("Provide a second argument for the `-` operator.")?;
                    rexp = RExp::Sub(Box::new(rexp), term_next);
                    println!("[Parser] new rexp: {}", rexp);
                }
                _ => {
                    return Ok(rexp);
                }
            }
        }
    }

    fn term(&mut self, errmsg: impl Into<String>) -> Result<Term, ParsingError> {
        let token = self.lexer.peek();
        let term = match token.tokentype {
            TT::Ident(_) => Term::LExp(LExp::Ident(token.into())),
            TT::IntLiteral(_) => Term::IntLit(token.into()),
            _ => return Err(PErr::unexpected(token, errmsg)),
        };
        self.lexer.consume()?;
        println!("[Parser] term: {}", term);
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
