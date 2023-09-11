use std::{
    fmt::{Debug, Display},
    fs::read_to_string,
};

#[derive(Debug, Clone, Default)]
pub struct Token {
    pub loc: Location,
    pub tokentype: TokenType,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub file: Option<String>,
    pub row: usize,
    pub col: usize,
}

impl Default for Location {
    fn default() -> Self {
        return Self {
            file: None,
            row: 1,
            col: 1,
        };
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.file.is_none() {
            write!(f, "({}:{})", self.row, self.col)
        } else {
            write!(
                f,
                "{}:{}:{}",
                self.file.as_ref().unwrap(),
                self.row,
                self.col
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    StartOfFile,
    Ident(String),
    IntLiteral(String),
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
    IllegalToken(Location, String),
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
            peek_token: Default::default(),
            cursor: 0,
            loc: Location {
                file: Some(path.into()),
                row: 1,
                col: 1,
            },
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

    pub fn consume(&mut self) -> Result<(), LexingError> {
        self.skip_whitespace();
        self.peek_token.loc = self.loc.clone();
        let Some(ch) = self.peek_ch else {
            self.peek_token.tokentype = TT::EndOfFile;
            return Ok(());
        };
        match ch {
            '=' => {
                self.consume_ch();
                self.peek_token.tokentype = TT::Assign;
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.ident_or_keyword(),
            ch if ch.is_ascii_digit() => self.int_literal()?,
            ch => {
                return Err(LErr::IllegalToken(
                    self.peek_token.loc.clone(),
                    String::from(ch),
                ))
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
        while self.peek_ch.map_or(false, |ch| ch.is_whitespace()) {
            self.consume_ch();
        }
    }

    // fn make_token(&self, tokentype: TokenType) -> Token {
    //     let token = Token {
    //         loc: self.loc.clone(),
    //         tokentype,
    //     };
    //     println!("[Lexer] {:?}", token);
    //     return token;
    // }

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
            return Err(LErr::IllegalToken(self.peek_token.loc.clone(), lexeme));
        }

        self.peek_token.tokentype = TT::IntLiteral(lexeme);
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
                self.peek_token.tokentype = TT::Let;
            }
            _ => {
                self.peek_token.tokentype = TT::Ident(lexeme);
            }
        };
        println!("{:?}", self.peek_token);
    }
}
