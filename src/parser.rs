use crate::{
    lexer::{Location, Token, TokenType as TT},
    CompileError, HandleNotFound,
};

use super::lexer::Lexer;
use std::{fmt::Display, rc::Rc};

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
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
    pub file: Option<Rc<str>>,
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
    pub file: Option<Rc<str>>,
    pub start: Location,
    pub end: Location,
    pub lexeme: String,
}

impl Into<LExp> for Identifier {
    fn into(self) -> LExp {
        LExp::Ident(self)
    }
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
    Neg(Box<Term>),
    Bracketed(Box<RExp>),
}

impl TryFrom<Token> for Term {
    type Error = Token;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.tokentype {
            TT::Ident(_) => Ok(Term::LExp(LExp::Ident(Identifier::from(value)))),
            TT::IntLiteral(_) => Ok(Term::IntLit(IntLiteral::from(value))),
            _ => Err(value),
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LExp(LExp::Ident(ident)) => write!(f, "{}", ident.lexeme),
            Self::IntLit(intlit) => write!(f, "{}", intlit.lexeme),
            Self::Neg(term) => write!(f, "-{}", term),
            Self::Bracketed(rexp) => write!(f, "({})", rexp),
            _ => panic!("[Display for Term] not implemented: {:?}", self),
        }
    }
}

type Block = Vec<Stmt>;

#[derive(Debug)]
pub enum Stmt {
    Declare(Identifier),
    Initialize(Identifier, RExp),
    Assign(LExp, RExp),
    RExp(RExp),
    Block(Block),
    If(RExp, Block, Option<Box<Stmt>>),
    Exit(RExp),
}

impl Stmt {
    pub fn is_if(&self) -> bool {
        match self {
            Self::If(_, _, _) => true,
            _ => false,
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declare(ident) => write!(f, "Declare({})", ident),
            Self::Assign(lexp, rexp) => write!(f, "Assign({}, {})", lexp, rexp),
            Self::Initialize(ident, rexp) => write!(f, "Initialize({}, {})", ident, rexp),
            Self::RExp(rexp) => write!(f, "RExp({})", rexp),
            Self::Block(block) => {
                writeln!(f, "{{")?;
                for stmt in block {
                    writeln!(f, "{}", stmt)?;
                }
                writeln!(f, "}}")?;
                return Ok(());
            }
            Self::If(rexp, if_block, else_block) => {
                writeln!(f, "if {} {{", rexp)?;
                for stmt in if_block {
                    writeln!(f, "{}", stmt)?;
                }
                write!(f, "}}")?;
                let else_stmt = match else_block {
                    None => return writeln!(f),
                    Some(else_box) => {
                        write!(f, " else ")?;
                        else_box.as_ref()
                    }
                };
                match else_stmt {
                    Stmt::Block(else_stmts) => {
                        writeln!(f, "{{")?;
                        for stmt in else_stmts {
                            writeln!(f, "{}", stmt)?;
                        }

                        write!(f, "}}")?;
                    }
                    stmt if stmt.is_if() => write!(f, "{}", stmt)?,
                    else_stmt => {
                        panic!(
                            "[Display for Stmt] else_block in if contains: {:?}",
                            else_stmt
                        )
                    }
                }

                return Ok(());
            }

            Self::Exit(rexp) => write!(f, "Exit({})", rexp),
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
    type Error = RExp;
    fn try_from(value: RExp) -> Result<Self, Self::Error> {
        return match value {
            RExp::Term(Term::LExp(lexp)) => Ok(lexp),
            _ => Err(value),
        };
    }
}

impl TryFrom<Token> for LExp {
    type Error = Token;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.tokentype {
            TT::Ident(_) => Ok(LExp::Ident(value.into())),
            _ => Err(value),
        }
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
    rexp_nesting_level: u32,
    pub program: Program,
}

impl Parser {
    pub fn new(source: String) -> Self {
        return Self {
            lexer: Lexer::new(source),
            program: Program { stmts: Vec::new() },
            rexp_nesting_level: 0,
        };
    }
    pub fn from_file(path: Rc<str>) -> Self {
        return Self {
            lexer: Lexer::from_file(path),
            program: Program { stmts: Vec::new() },
            rexp_nesting_level: 0,
        };
    }

    pub fn parse_program(&mut self) -> Result<(), CompileError> {
        loop {
            self.skip_newlines()?;
            match self.stmt() {
                Ok(stmt) => self.program.stmts.push(stmt),
                Err(CompileError::NotFound) => {
                    println!("Notfound");
                    break;
                }
                Err(err) => return Err(err),
            }

            match parse_terminal!(self.lexer, TT::NewLine) {
                Ok(_) => continue,
                _ => (),
            }
            match parse_terminal!(self.lexer, TT::EndOfFile) {
                Ok(_) => break,
                Err(token) => return Err(CompileError::ExpectedNewline(token.start)),
            }
        }
        return Ok(());
    }

    fn stmt(&mut self) -> Result<Stmt, CompileError> {
        let token = self.lexer.peek();

        let stmt = match token.tokentype {
            TT::Let => self.decl_or_init(),
            TT::Ident(_) | TT::IntLiteral(_) | TT::SBrace | TT::Minus => self.assign_stmt_or_rexp(),
            TT::Exit => self.exit(),
            TT::SCurly => self.block(),
            TT::If => self.if_(),
            _ => Err(CompileError::NotFound),
        };
        match stmt {
            Ok(ref stmt) => println!("[Parser.stmt] Ok({})", stmt),
            Err(ref err) => println!("[Parser.stmt] Err({:?})", err),
        }
        stmt
    }

    fn skip_newlines(&mut self) -> Result<bool, CompileError> {
        let mut newlines_skipped = false;
        while parse_terminal!(self.lexer, TT::NewLine | TT::StartOfFile).is_ok() {
            newlines_skipped = true;
        }
        return Ok(newlines_skipped);
    }

    fn if_(&mut self) -> Result<Stmt, CompileError> {
        match parse_terminal!(self.lexer, TT::If) {
            Err(_) => return Err(CompileError::NotFound),
            _ => (),
        }
        let rexp = self
            .rexp()
            .handle_not_found(CompileError::ExpectedExpression(self.lexer.peek().start))?;

        let if_block = match self
            .block()
            .handle_not_found(CompileError::ExpectedBlock(self.lexer.peek().start))?
        {
            Stmt::Block(block) => block,
            stmt => panic!("[Parser.if_] Parser.block returned: {}", stmt),
        };

        match parse_terminal!(self.lexer, TT::Else) {
            Err(_) => {
                return Ok(Stmt::If(rexp, if_block, None));
            }

            _ => (),
        }

        match self.if_() {
            Ok(else_if_block) => {
                return Ok(Stmt::If(rexp, if_block, Some(Box::new(else_if_block))))
            }
            Err(CompileError::NotFound) => (),
            Err(err) => return Err(err),
        }

        match self.block() {
            Ok(else_block) => return Ok(Stmt::If(rexp, if_block, Some(Box::new(else_block)))),
            Err(CompileError::NotFound) => {
                return Err(CompileError::ExpectedBlock(self.lexer.peek().start))
            }
            Err(err) => return Err(err),
        }
    }

    fn block(&mut self) -> Result<Stmt, CompileError> {
        match parse_terminal!(self.lexer, TT::SCurly) {
            Ok(_) => (),
            Err(_) => return Err(CompileError::NotFound),
        }
        let mut stmts = Vec::<Stmt>::new();

        loop {
            while parse_terminal!(self.lexer, TT::NewLine).is_ok() {}
            match self.stmt() {
                Ok(stmt) => stmts.push(stmt),
                Err(CompileError::NotFound) => break,
                err => return err,
            }
            match parse_terminal!(self.lexer, TT::NewLine) {
                Err(_) => break,
                _ => (),
            }
        }

        match parse_terminal!(self.lexer, TT::ECurly) {
            Ok(_) => (),
            Err(token) => return Err(CompileError::ExpectedECurly(token.start)),
        }
        return Ok(Stmt::Block(stmts));
    }

    fn exit(&mut self) -> Result<Stmt, CompileError> {
        let exit_kw_loc = match parse_terminal!(self.lexer, TT::Exit) {
            Ok(token) => token.end,
            Err(_) => return Err(CompileError::NotFound),
        };
        let rexp = self
            .rexp()
            .handle_not_found(CompileError::ExpectedExpression(exit_kw_loc))?;
        return Ok(Stmt::Exit(rexp));
    }

    fn assign_stmt_or_rexp(&mut self) -> Result<Stmt, CompileError> {
        let exp = self
            .rexp()
            .handle_not_found(CompileError::ExpectedExpression(self.lexer.peek().start))?;
        let assign_loc = match parse_terminal!(self.lexer, TT::Assign) {
            Err(_) => return Ok(Stmt::RExp(exp)),
            Ok(token) => token.end,
        };

        let lexp = match LExp::try_from(exp) {
            Err(rexp) => return Err(CompileError::RExpOnLHS(rexp)),
            Ok(lexp) => lexp,
        };
        let rexp = self
            .rexp()
            .handle_not_found(CompileError::ExpectedExpression(assign_loc))?;
        return Ok(Stmt::Assign(lexp, rexp));
    }

    fn rexp_min_prec(&mut self, min_prec: usize) -> Result<RExp, CompileError> {
        let mut rexp = self.term()?.into();
        loop {
            let op = self.lexer.peek();
            if !is_op(&op.tokentype) {
                break;
            }
            let op_location = op.end;
            let (prec, assoc) = op_prec_and_assoc(&op.tokentype);
            if prec < min_prec {
                break;
            }
            self.lexer.consume()?;
            let next_min_prec = match assoc {
                OpAssoc::Left => prec + 1,
                OpAssoc::Right => prec,
            };
            let rhs = self
                .rexp_min_prec(next_min_prec)
                .handle_not_found(CompileError::ExpectedExpression(op_location))?;
            rexp = RExp::combine(&op.tokentype, rexp, rhs)
        }
        return Ok(rexp);
    }

    fn rexp(&mut self) -> Result<RExp, CompileError> {
        return self.rexp_min_prec(0);
    }

    fn term(&mut self) -> Result<Term, CompileError> {
        match parse_terminal!(self.lexer, TT::Ident(_) | TT::IntLiteral(_)) {
            Ok(token) => return Ok(token.try_into().unwrap()),
            _ => (),
        }
        match parse_terminal!(self.lexer, TT::Minus) {
            Ok(_) => return Ok(Term::Neg(Box::new(self.term()?))),
            _ => (),
        }
        let token = self.lexer.peek();
        match token.tokentype {
            TT::SBrace => {
                self.rexp_nesting_level += 1;
                self.lexer.emit_newline = false;
                self.lexer.consume()?;
            }
            _ => return Err(CompileError::NotFound),
        }
        let rexp = self
            .rexp()
            .handle_not_found(CompileError::ExpectedExpression(self.lexer.peek().start))?;
        let token = self.lexer.peek();
        match token.tokentype {
            TT::EBrace => {
                self.rexp_nesting_level -= 1;
                if self.rexp_nesting_level <= 0 {
                    self.lexer.emit_newline = true;
                }
                self.lexer.consume()?;
            }
            _ => return Err(CompileError::UnexpectedToken(token)),
        }
        return Ok(Term::Bracketed(Box::new(rexp)));
    }

    fn decl_or_init(&mut self) -> Result<Stmt, CompileError> {
        match parse_terminal!(self.lexer, TT::Let) {
            Err(token) => panic!("[Parser.decl_or_init] Expected `let` but got: {:?}", token),
            Ok(_) => (),
        }
        let ident = match parse_terminal!(self.lexer, TT::Ident(_)) {
            Ok(token) => Identifier::from(token),
            Err(token) => return Err(CompileError::ExpectedIdent(token.start)),
        };

        match parse_terminal!(self.lexer, TT::Assign) {
            Err(_) => return Ok(Stmt::Declare(ident)),
            Ok(_) => (),
        }

        let rexp = self
            .rexp()
            .handle_not_found(CompileError::ExpectedExpression(self.lexer.peek().start))?;
        return Ok(Stmt::Initialize(ident.into(), rexp));
    }
}
