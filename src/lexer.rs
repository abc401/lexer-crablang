use std::{
    fmt::{Debug, Display},
    fs::read_to_string,
    vec,
};

#[derive(Debug, Clone, Default)]
pub struct Token {
    pub file: Option<String>,
    pub start: Location,
    pub end: Location,
    pub tokentype: TokenType,
}

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Default for Location {
    fn default() -> Self {
        return Self { row: 1, col: 1 };
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    StartOfFile,
    NewLine,
    Ident(String),
    IntLiteral(String),
    Illegal(String),
    Let,
    EndOfFile,
    Assign,
    Plus,
    Minus,
    Asterisk,
    ForwardSlash,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
use TokenType as TT;

use crate::CompileError;

impl Default for TokenType {
    fn default() -> Self {
        return TT::StartOfFile;
    }
}

#[derive(Debug)]
pub struct Lexer {
    source: Vec<char>,
    peek_ch: Option<char>,
    tokens: Vec<Token>,
    next_token: Token,

    token_cursor: usize,
    cursor: usize,
    file: String,
    loc: Location,
}

impl Lexer {
    pub fn from_file(path: &str) -> Self {
        let source = read_to_string(path).expect("Provided input file does not exist!");
        let first_token = Token {
            file: Some(path.into()),
            start: Location::default(),
            end: Location::default(),
            tokentype: TT::StartOfFile,
        };
        let mut ret = Self {
            source: source.chars().collect(),
            peek_ch: None,
            tokens: vec![first_token],
            next_token: Token::default(),
            file: path.into(),
            cursor: 0,
            token_cursor: 0,
            loc: Location::default(),
        };
        ret.next_token.file = Some(path.into());
        if ret.source.len() > 0 {
            ret.peek_ch = Some(ret.source[0]);
        }
        return ret;
    }

    fn is_eof(&mut self) -> bool {
        return self.cursor >= self.source.len();
    }

    pub fn peek(&self) -> Token {
        return self.tokens[self.token_cursor].clone();
    }

    fn make_next_token(&mut self) {
        self.next_token.start = self.loc;
    }

    fn set_next_token(&mut self, tokentype: TokenType) {
        self.next_token.tokentype = tokentype;
        self.next_token.end = self.loc;
        self.tokens.push(self.next_token.clone());
        self.token_cursor += 1;
    }

    pub fn rewind(&mut self) {
        if self.token_cursor <= 0 {
            return;
        }
        self.token_cursor -= 1;
    }

    fn single_char_token(&mut self, tokentype: TokenType) {
        self.consume_ch();
        self.set_next_token(tokentype);
    }

    pub fn consume(&mut self) -> Result<(), CompileError> {
        // println!("[Lexer] peek: {:?}", self.peek().tokentype);
        if self.token_cursor < self.tokens.len() - 1 {
            self.token_cursor += 1;
            return Ok(());
        }
        self.skip_whitespace();
        self.make_next_token();
        let Some(ch) = self.peek_ch else {
            self.set_next_token(TT::EndOfFile);
            return Ok(());
        };
        match ch {
            '\n' => self.single_char_token(TT::NewLine),
            '+' => self.single_char_token(TT::Plus),
            '-' => self.single_char_token(TT::Minus),
            '*' => self.single_char_token(TT::Asterisk),
            '/' => self.single_char_token(TT::ForwardSlash),
            '!' => {
                self.consume_ch();
                let Some(ch) = self.peek_ch else {
                    self.set_next_token(TT::Illegal('!'.into()));
                    return Ok(());
                };
                match ch {
                    '=' => self.single_char_token(TT::NotEqual),
                    _ => self.set_next_token(TT::Illegal('!'.into())),
                }
            }
            '>' => {
                self.consume_ch();
                let Some(ch) = self.peek_ch else {
                    self.set_next_token(TT::Greater);
                    return Ok(());
                };
                match ch {
                    '=' => self.single_char_token(TT::GreaterEqual),
                    _ => self.set_next_token(TT::Greater),
                }
            }
            '<' => {
                self.consume_ch();
                let Some(ch) = self.peek_ch else {
                    self.set_next_token(TT::Less);
                    return Ok(());
                };
                match ch {
                    '=' => self.single_char_token(TT::LessEqual),
                    _ => self.set_next_token(TT::Less),
                }
            }
            '=' => {
                self.consume_ch();
                let Some(ch) = self.peek_ch else {
                    self.set_next_token(TT::Assign);
                    return Ok(());
                };
                match ch {
                    '=' => self.single_char_token(TT::Equal),
                    _ => self.set_next_token(TT::Assign),
                }
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.ident_or_keyword(),
            ch if ch.is_ascii_digit() => self.int_literal()?,
            ch => {
                self.set_next_token(TT::Illegal(String::from(ch)));
                return Err(CompileError::IllegalToken(self.peek()));
            }
        };
        return Ok(());
    }

    fn consume_ch(&mut self) {
        if self.is_eof() {
            return;
        }

        self.cursor += 1;

        if let Some('\n') = self.peek_ch {
            self.loc.row += 1;
            self.loc.col = 1;
        } else if !self.is_eof() && self.source[self.cursor] != '\n' {
            self.loc.col += 1;
        }

        if !self.is_eof() {
            self.peek_ch = Some(self.source[self.cursor as usize]);
        } else {
            self.peek_ch = None;
        }
    }

    fn skip_whitespace(&mut self) {
        while self
            .peek_ch
            .map_or(false, |ch| ch.is_whitespace() && ch != '\n')
        {
            self.consume_ch();
        }
    }

    fn int_literal(&mut self) -> Result<(), CompileError> {
        // TODO: Handle 64 bit int literals
        let Some(ch) = self.peek_ch else {
            panic!("[Lexer.int_literal] Called eventhough no characters are left!");
        };
        assert!(
            ch.is_ascii_digit(),
            "[Lexer.ident_or_keyword] Falsely called!"
        );

        let mut lexeme = String::new();
        while self.peek_ch.map_or(false, |ch| ch.is_ascii_digit()) {
            lexeme.push(self.peek_ch.unwrap());
            self.consume_ch();
        }
        let mut illegal_lexeme = String::new();
        while self.peek_ch.map_or(false, |ch| ch.is_ascii_alphanumeric()) {
            illegal_lexeme.push(self.peek_ch.unwrap());
            self.consume_ch();
        }

        if illegal_lexeme.len() > 0 {
            lexeme.extend(illegal_lexeme.chars());
            self.set_next_token(TT::Illegal(lexeme));
            return Err(CompileError::IllegalToken(self.peek()));
        }

        self.set_next_token(TT::IntLiteral(lexeme));
        // println!("[Lexer] [IntLiteral] {:?}", self.peek());
        return Ok(());
    }

    fn ident_or_keyword(&mut self) {
        let Some(ch) = self.peek_ch else {
            panic!("[Lexer.ident_or_keyword] Called eventhough no characters are left!");
        };
        assert!(
            ch.is_ascii_alphabetic() || ch == '_',
            "[Lexer.ident_or_keyword] Falsely called!"
        );

        let mut lexeme = String::new();
        while self
            .peek_ch
            .map_or(false, |ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            lexeme.push(self.peek_ch.unwrap());
            self.consume_ch();
        }

        match lexeme.as_str() {
            "let" => {
                self.set_next_token(TT::Let);
            }
            _ => {
                self.set_next_token(TT::Ident(lexeme));
            }
        };
    }
}
