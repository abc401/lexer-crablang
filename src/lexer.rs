use std::{
    fmt::{Debug, Display},
    fs::read_to_string,
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
}
use TokenType as TT;
impl Default for TokenType {
    fn default() -> Self {
        return TT::StartOfFile;
    }
}

pub enum LexingError {
    IllegalToken(Token),
}
use LexingError as LErr;

#[derive(Debug)]
pub struct Lexer {
    source: Vec<char>,
    peek_ch: Option<char>,
    peek_token: Token,
    cursor: usize,
    loc: Location,
}

impl Lexer {
    pub fn from_file(path: &str) -> Self {
        let source = read_to_string(path).expect("Provided input file does not exist!");
        let mut ret = Self {
            source: source.chars().collect(),
            peek_ch: None,
            peek_token: Token {
                file: Some(path.into()),
                ..Default::default()
            },
            cursor: 0,
            loc: Location { row: 1, col: 1 },
        };
        if ret.source.len() > 0 {
            ret.peek_ch = Some(ret.source[0]);
        }
        return ret;
    }

    fn is_eof(&mut self) -> bool {
        return self.cursor >= self.source.len();
    }

    pub fn peek(&self) -> Token {
        return self.peek_token.clone();
    }

    fn set_peek_token(&mut self, tokentype: TokenType) {
        self.peek_token.tokentype = tokentype;
        self.peek_token.end = self.loc;
    }

    pub fn consume(&mut self) -> Result<(), LexingError> {
        self.skip_whitespace();
        self.peek_token.start = self.loc.clone();
        let Some(ch) = self.peek_ch else {
            self.set_peek_token(TT::EndOfFile);
            return Ok(());
        };
        match ch {
            '\n' => {
                self.consume_ch();
                self.set_peek_token(TT::NewLine);
            }
            '=' => {
                self.consume_ch();
                self.set_peek_token(TT::Assign);
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.ident_or_keyword(),
            ch if ch.is_ascii_digit() => self.int_literal()?,
            ch => {
                self.set_peek_token(TT::Illegal(String::from(ch)));
                return Err(LErr::IllegalToken(self.peek_token.clone()));
            }
        };
        return Ok(());
    }

    fn consume_ch(&mut self) {
        if self.is_eof() {
            return;
        }

        if let Some('\n') = self.peek_ch {
            self.loc.row += 1;
            self.loc.col = 1;
        } else {
            self.loc.col += 1;
        }

        self.cursor += 1;
        if self.cursor < self.source.len() {
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

    fn int_literal(&mut self) -> Result<(), LexingError> {
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
            self.set_peek_token(TT::Illegal(lexeme));
            return Err(LErr::IllegalToken(self.peek_token.clone()));
        }

        self.set_peek_token(TT::IntLiteral(lexeme));
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
                self.set_peek_token(TT::Let);
            }
            _ => {
                self.set_peek_token(TT::Ident(lexeme));
            }
        };
        // println!("{:?}", self.peek_token);
    }
}
