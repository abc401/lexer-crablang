use crate::{
    lexer::{Location, Token, TokenType as TT},
    CompileError,
};

use super::lexer::Lexer;
use std::{
    fmt::Display,
    rc::Rc,
    slice::{Iter, IterMut},
};

#[derive(Debug)]
pub struct Program {
    stmts: Vec<Stmt>,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program {{\n")?;
        for stmt in self.stmts.iter() {
            write!(f, "{}\n", stmt)?;
            write!(f, "---------------------------------------------------\n")?;
        }
        write!(f, "}}")?;
        return Ok(());
    }
}

#[derive(Debug)]
pub struct IntLiteral {
    pub file: Rc<str>,
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
    pub file: Rc<str>,
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
    LExp(LExp),
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
    pub fn iter(&self) -> Iter<'_, Stmt> {
        self.stmts.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Stmt> {
        self.stmts.iter_mut()
    }
}

#[derive(Debug)]
pub enum Stmt {
    Declare(Identifier),
    Initialize(Identifier, RExp),
    Assign(LExp, RExp),
    RExp(RExp),
    Scope(Vec<Stmt>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declare(ident) => write!(f, "Declare({})", ident),
            Self::Assign(lexp, rexp) => write!(f, "Assign({}, {})", lexp, rexp),
            Self::Initialize(ident, rexp) => write!(f, "Initialize({}, {})", ident, rexp),
            Self::RExp(rexp) => write!(f, "RExp({})", rexp),
            Self::Scope(scope) => {
                writeln!(f, "{{")?;
                for stmt in scope {
                    writeln!(f, "{}", stmt)?;
                }
                writeln!(f, "}}")?;
                return Ok(());
            }
            _ => panic!("[Display for Stmt] unimplemented: {:?}", self),
        }
    }
}

#[derive(Debug)]
pub enum RExp {
    Term(Term),
    Add(Box<RExp>, Box<RExp>),
    Sub(Box<RExp>, Box<RExp>),
    Mul(Box<RExp>, Box<RExp>),
    Div(Box<RExp>, Box<RExp>),
    Equal(Box<RExp>, Box<RExp>),
    NotEqual(Box<RExp>, Box<RExp>),
    Less(Box<RExp>, Box<RExp>),
    LessEqual(Box<RExp>, Box<RExp>),
    Greater(Box<RExp>, Box<RExp>),
    GreaterEqual(Box<RExp>, Box<RExp>),
}

impl RExp {
    fn combine(operator: &TT, lhs: RExp, rhs: RExp) -> Self {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);
        match operator {
            TT::Plus => RExp::Add(lhs, rhs),
            TT::Minus => RExp::Sub(lhs, rhs),
            TT::Asterisk => RExp::Mul(lhs, rhs),
            TT::ForwardSlash => RExp::Div(lhs, rhs),
            TT::Equal => RExp::Equal(lhs, rhs),
            TT::NotEqual => RExp::NotEqual(lhs, rhs),
            TT::Less => RExp::Less(lhs, rhs),
            TT::LessEqual => RExp::LessEqual(lhs, rhs),
            TT::Greater => RExp::Greater(lhs, rhs),
            TT::GreaterEqual => RExp::GreaterEqual(lhs, rhs),
            _ => panic!(
                "[Parser] [RExp.from_bin_exp] Invalid operator: {:?}",
                operator
            ),
        }
    }
}

impl Display for RExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RExp::Add(lhs, rhs) => write!(f, "({} + {})", lhs, rhs),
            RExp::Mul(lhs, rhs) => write!(f, "({} * {})", lhs, rhs),
            RExp::Sub(lhs, rhs) => write!(f, "({} - {})", lhs, rhs),
            RExp::Div(lhs, rhs) => write!(f, "({} / {})", lhs, rhs),
            RExp::Equal(lhs, rhs) => write!(f, "({} == {})", lhs, rhs),
            RExp::NotEqual(lhs, rhs) => write!(f, "({} != {})", lhs, rhs),
            RExp::Less(lhs, rhs) => write!(f, "({} < {})", lhs, rhs),
            RExp::LessEqual(lhs, rhs) => write!(f, "({} <= {})", lhs, rhs),
            RExp::Greater(lhs, rhs) => write!(f, "({} > {})", lhs, rhs),
            RExp::GreaterEqual(lhs, rhs) => write!(f, "({} >= {})", lhs, rhs),
            RExp::Term(term) => write!(f, "{}", term),
            _ => panic!("[RExp.Display] not implemented for: {:?}", self),
        }
    }
}

impl TryInto<Term> for RExp {
    type Error = ();
    fn try_into(self) -> Result<Term, Self::Error> {
        match self {
            Self::Term(term) => Ok(term),
            _ => return Err(()),
        }
    }
}

impl From<LExp> for RExp {
    fn from(value: LExp) -> Self {
        return RExp::Term(Term::LExp(value));
    }
}

impl From<Term> for RExp {
    fn from(value: Term) -> Self {
        return RExp::Term(value);
    }
}

#[derive(Debug)]
pub enum LExp {
    Ident(Identifier),
}
impl TryFrom<RExp> for LExp {
    type Error = ();
    fn try_from(value: RExp) -> Result<Self, Self::Error> {
        return match value {
            RExp::Term(Term::LExp(lexp)) => Ok(lexp),
            _ => Err(()),
        };
    }
}

impl Display for LExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => write!(f, "{}", ident),
        }
    }
}

fn is_op(tokentype: &TT) -> bool {
    match tokentype {
        TT::Minus
        | TT::Plus
        | TT::Asterisk
        | TT::ForwardSlash
        | TT::Equal
        | TT::NotEqual
        | TT::Less
        | TT::LessEqual
        | TT::Greater
        | TT::GreaterEqual => true,
        _ => false,
    }
}

enum OpAssoc {
    Left,
    Right,
}

fn op_prec_and_assoc(tokentype: &TT) -> (usize, OpAssoc) {
    match tokentype {
        TT::Equal | TT::NotEqual | TT::Less | TT::LessEqual | TT::Greater | TT::GreaterEqual => {
            (1, OpAssoc::Right)
        }
        TT::Minus | TT::Plus => (2, OpAssoc::Left),
        TT::Asterisk | TT::ForwardSlash => (3, OpAssoc::Left),
        _ => panic!("{:?} is not an operator.", tokentype),
    }
}

macro_rules! parse_terminal {
    ($lexer:expr, $pattern:pat) => {{
        let token = $lexer.peek();
        match token.tokentype {
            $pattern => {
                $lexer.consume()?;
                Ok(token)
            }
            _ => Err(token),
        }
    }};
}

pub struct Parser {
    lexer: Lexer,
    pub program: Program,
}

impl Parser {
    pub fn from_file(path: Rc<str>) -> Self {
        return Self {
            lexer: Lexer::from_file(path),
            program: Program { stmts: Vec::new() },
        };
    }

    fn parse_aux(&mut self, scope: &mut Vec<Stmt>) -> Result<(), CompileError> {
        loop {
            let token = self.lexer.peek();
            println!("[Parser] Parsing new statement.");
            println!("[Parser] Peek token: {:?}", token);
            if !self.program.stmts.is_empty() {
                println!("[Parser] Statement: {}", self.program.stmts.last().unwrap())
            }
            match token.tokentype {
                TT::StartOfFile | TT::NewLine => {
                    self.lexer.consume()?;
                    continue;
                }

                TT::EndOfFile => {
                    break;
                }
                TT::SCurly => {
                    self.scope(scope)?;
                }
                TT::ECurly => {
                    return Ok(());
                }

                TT::Let => {
                    self.lexer.consume()?;
                    self.decl_or_init(scope)?;
                }
                TT::Ident(_) | TT::IntLiteral(_) => {
                    self.assign_stmt_or_rexp(scope)?;
                }
                _ => {
                    return Err(CompileError::unexpected(
                        token,
                        "Invalid token for starting a statement",
                    ));
                }
            }
        }
        return Ok(());
    }

    pub fn parse(&mut self) -> Result<(), CompileError> {
        let mut stmts = Vec::<Stmt>::new();
        self.parse_aux(&mut stmts)?;
        if !self.lexer.is_eof() {
            let token = self.lexer.peek();
            return Err(CompileError::unexpected(
                token,
                "Could not parser source completely.",
            ));
        }
        std::mem::swap(&mut self.program.stmts, &mut stmts);
        return Ok(());
    }

    fn scope(&mut self, parent_scope: &mut Vec<Stmt>) -> Result<(), CompileError> {
        let res = parse_terminal!(self.lexer, TT::SCurly);
        if res.is_err() {
            return Err(CompileError::unexpected(
                res.unwrap_err(),
                "[Parser.scope] err 1",
            ));
        }
        let mut scope = Vec::<Stmt>::new();
        self.parse_aux(&mut scope)?;
        let res = parse_terminal!(self.lexer, TT::ECurly);
        if res.is_err() {
            return Err(CompileError::unexpected(
                res.unwrap_err(),
                "[Parser.scope] err 2",
            ));
        }
        parent_scope.push(Stmt::Scope(scope));
        return Ok(());
    }

    fn assign(&mut self, errmsg: impl Into<String>) -> Result<(), CompileError> {
        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::Assign) {
            return Err(CompileError::unexpected(token, errmsg));
        }
        self.lexer.consume()?;
        return Ok(());
    }

    fn newline_or_eof(&mut self, errmsg: impl Into<String>) -> Result<(), CompileError> {
        let token = self.lexer.peek();
        if !matches!(token.tokentype, TT::NewLine | TT::EndOfFile) {
            return Err(CompileError::unexpected(token, errmsg));
        }
        self.lexer.consume()?;
        return Ok(());
    }

    fn assign_stmt_or_rexp(&mut self, scope: &mut Vec<Stmt>) -> Result<(), CompileError> {
        let exp = self.rexp("");
        if exp.is_ok() && self.newline_or_eof("").is_ok() {
            scope.push(Stmt::RExp(exp.unwrap()));
            return Ok(());
        }

        let lexp = LExp::try_from(exp.unwrap());
        if lexp.is_err() {
            return Err(CompileError::unexpected(
                self.lexer.peek(),
                "Expected an lexpression.",
            ));
        }
        let lexp = lexp.unwrap();

        let res = self.assign("");
        match res {
            Ok(_) => {}
            Err(_) => {
                self.newline_or_eof("[assign_stmt_or_rexp] Expected a newline.")?;
                scope.push(Stmt::RExp(lexp.into()));
                return Ok(());
            }
        }

        println!("[Parser] next token: {:?}", self.lexer.peek());
        let rexp = self.rexp("What do I assign it to?")?;
        scope.push(Stmt::Assign(lexp, rexp));
        return Ok(());
    }

    fn rexp_min_prec(
        &mut self,
        min_prec: usize,
        errmsg: impl Into<String>,
    ) -> Result<RExp, CompileError> {
        let mut rexp = self.term(errmsg)?.into();
        loop {
            let op = self.lexer.peek();
            if !is_op(&op.tokentype) {
                break;
            }
            let (prec, assoc) = op_prec_and_assoc(&op.tokentype);
            if prec < min_prec {
                break;
            }
            self.lexer.consume()?;
            let next_min_prec = match assoc {
                OpAssoc::Left => prec + 1,
                OpAssoc::Right => prec,
            };
            let rhs = self.rexp_min_prec(
                next_min_prec,
                format!("Expected rhs for binary operator: {:?}", op),
            )?;
            rexp = RExp::combine(&op.tokentype, rexp, rhs)
        }
        return Ok(rexp);
    }

    fn rexp(&mut self, errmsg: impl Into<String>) -> Result<RExp, CompileError> {
        return self.rexp_min_prec(0, errmsg);
    }

    fn term(&mut self, errmsg: impl Into<String>) -> Result<Term, CompileError> {
        let token = self.lexer.peek();
        let term = match token.tokentype {
            TT::Ident(_) => Term::LExp(LExp::Ident(token.into())),
            TT::IntLiteral(_) => Term::IntLit(token.into()),
            _ => return Err(CompileError::unexpected(token, errmsg)),
        };
        self.lexer.consume()?;
        println!("[Parser] term: {}", term);
        return Ok(term);
    }

    fn ident(&mut self) -> Result<Identifier, CompileError> {
        let token = self.lexer.peek();
        let TT::Ident(_) = token.tokentype else {

            return Err(CompileError::unexpected(token, "Expected an identifier."));
        };
        self.lexer.consume()?;
        return Ok(token.into());
    }

    fn decl_or_init(&mut self, scope: &mut Vec<Stmt>) -> Result<(), CompileError> {
        let ident = self.ident()?;
        if self.newline_or_eof("").is_ok() {
            scope.push(Stmt::Declare(ident));
            return Ok(());
        }

        self.assign("Expected an `=` sign.")?;

        let rexp = self.rexp("What do I initialize it to?")?;

        self.newline_or_eof("[decl_or_init] Expected a newline.")?;

        scope.push(Stmt::Initialize(ident, rexp));

        return Ok(());
    }
}
