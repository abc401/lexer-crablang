use std::{
    fmt::{Debug, Display},
    fs::read_to_string,
    rc::Rc,
    vec,
};

const DEBUG_TOKENS: bool = false;

#[derive(Debug, Clone)]
pub struct Token {
    pub file: Option<Rc<str>>,
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
    EndOfFile,

    Ident(String),
    IntLiteral(String),
    Illegal(String),

    Let,
    Exit,
    If,
    Else,

    NewLine,

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

    SCurly,
    ECurly,

    SBrace,
    EBrace,
}
use TokenType as TT;

const TOKENTYPE_MAPPINGS: &[(&str, TT)] = &[
    ("==", TT::Equal),
    ("!=", TT::NotEqual),
    ("<=", TT::LessEqual),
    (">=", TT::GreaterEqual),
    ("+", TT::Plus),
    ("-", TT::Minus),
    ("*", TT::Asterisk),
    ("/", TT::ForwardSlash),
    ("=", TT::Assign),
    ("<", TT::Less),
    (">", TT::Greater),
    ("{", TT::SCurly),
    ("}", TT::ECurly),
    ("(", TT::SBrace),
    (")", TT::EBrace),
    ("\n", TT::NewLine),
];

use crate::CompileError;

impl Default for TokenType {
    fn default() -> Self {
        return TT::StartOfFile;
    }
}

#[derive(Debug)]
pub struct Lexer {
    source: Vec<char>,
    tokens: Vec<Token>,

    next_token: Token,
    token_cursor: usize,

    peek_ch: Option<char>,
    ch_cursor: usize,

    pub loc: Location,
    pub emit_newline: bool,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let first_token = Token {
            file: None,
            start: Location::default(),
            end: Location::default(),
            tokentype: TT::StartOfFile,
        };
        let mut ret = Self {
            source: source.chars().collect(),
            peek_ch: None,
            tokens: vec![first_token],
            next_token: Token {
                file: None,
                start: Location::default(),
                end: Location::default(),
                tokentype: TT::StartOfFile,
            },
            ch_cursor: 0,
            token_cursor: 0,
            loc: Location::default(),
            emit_newline: true,
        };
        if ret.source.len() > 0 {
            ret.peek_ch = Some(ret.source[0]);
        }
        return ret;
    }
    pub fn from_file(path: Rc<str>) -> Self {
        let source = read_to_string(path.as_ref()).expect("Provided input file does not exist!");
        let first_token = Token {
            file: Some(path.clone()),
            start: Location::default(),
            end: Location::default(),
            tokentype: TT::StartOfFile,
        };
        let mut ret = Self {
            source: source.chars().collect(),
            peek_ch: None,
            tokens: vec![first_token],
            next_token: Token {
                file: Some(path.clone()),
                start: Location::default(),
                end: Location::default(),
                tokentype: TT::StartOfFile,
            },
            ch_cursor: 0,
            token_cursor: 0,
            loc: Location::default(),
            emit_newline: true,
        };
        if ret.source.len() > 0 {
            ret.peek_ch = Some(ret.source[0]);
        }
        return ret;
    }

    pub fn is_eof(&mut self) -> bool {
        return self.ch_cursor >= self.source.len();
    }

    pub fn peek(&self) -> Token {
        return self.tokens[self.token_cursor].clone();
    }

    fn prepare_next_token(&mut self) {
        self.next_token.start = self.loc;
    }

    fn set_next_token(&mut self, tokentype: TokenType) {
        if DEBUG_TOKENS {
            println!("[Lexer] emit_newline: {}", self.emit_newline);
            println!("[Lexer] lexed: {:?}", tokentype);
        }

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

    fn try_consume_str(&mut self, string: &str) -> bool {
        let mut cursor = self.ch_cursor;
        for c in string.chars() {
            if cursor >= self.source.len() {
                return false;
            }
            if c != self.source[cursor] {
                return false;
            }
            cursor += 1;
        }
        for _ in string.chars() {
            self.consume_ch();
        }
        return true;
    }

    pub fn consume(&mut self) -> Result<(), CompileError> {
        // println!("[Lexer] peek: {:?}", self.peek().tokentype);
        if self.token_cursor < self.tokens.len() - 1 {
            self.token_cursor += 1;
            return Ok(());
        }
        self.skip_whitespace();
        self.prepare_next_token();
        let Some(ch) = self.peek_ch else {
            self.set_next_token(TT::EndOfFile);
            return Ok(());
        };

        for (string, tokentype) in TOKENTYPE_MAPPINGS.iter() {
            if self.try_consume_str(string) {
                self.set_next_token(tokentype.clone());
                return Ok(());
            }
        }

        match ch {
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.ident_or_keyword(),
            ch if ch.is_ascii_digit() => self.int_literal()?,
            ch => {
                self.set_next_token(TT::Illegal(String::from(ch)));
                self.consume_ch();
                return Err(CompileError::IllegalToken(self.peek()));
            }
        };
        return Ok(());
    }

    fn consume_ch(&mut self) {
        if self.is_eof() {
            return;
        }

        self.ch_cursor += 1;

        if let Some('\n') = self.peek_ch {
            self.loc.row += 1;
            self.loc.col = 1;
        } else if !self.is_eof() && self.source[self.ch_cursor] != '\n' {
            self.loc.col += 1;
        }

        if !self.is_eof() {
            self.peek_ch = Some(self.source[self.ch_cursor as usize]);
        } else {
            self.peek_ch = None;
        }
    }

    fn skip_whitespace(&mut self) {
        while self.peek_ch.map_or(false, |ch| {
            (ch != '\n' && ch.is_whitespace()) || (ch == '\n' && !self.emit_newline)
        }) {
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
            "else" => self.set_next_token(TT::Else),
            "exit" => self.set_next_token(TT::Exit),
            "let" => self.set_next_token(TT::Let),
            "if" => self.set_next_token(TT::If),
            _ => self.set_next_token(TT::Ident(lexeme)),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn illegal_tokens() {
        let source = String::from(
            r#"
        12dsa2&@$
        "#,
        );
        use TokenType::*;
        let expected = [
            StartOfFile,
            NewLine,
            Illegal(String::from("12dsa2")),
            Illegal(String::from("&")),
            Illegal(String::from("@")),
            Illegal(String::from("$")),
            NewLine,
            EndOfFile,
        ];
        let mut lexer = Lexer::new(source);
        let mut i = 0;
        while !lexer.is_eof() {
            let tokentype = &expected[i];
            println!("{:?}, {:?}", lexer.peek().tokentype, tokentype);
            assert_eq!(lexer.peek().tokentype, *tokentype);
            i += 1;
            let _ = lexer.consume();
        }
    }

    #[test]
    fn legal_tokens() {
        let source = String::from(
            r#"
a 
b  a1352 _ _ab

325252 1234

let exit if else 

= + - * /
== != < <= > >=

{ } {}
( ) ()

        "#,
        );

        use TokenType::*;
        let expected_result = [
            StartOfFile,
            NewLine,
            Ident(String::from("a")),
            NewLine,
            Ident(String::from("b")),
            Ident(String::from("a1352")),
            Ident(String::from("_")),
            Ident(String::from("_ab")),
            NewLine,
            NewLine,
            IntLiteral(String::from("325252")),
            IntLiteral(String::from("1234")),
            NewLine,
            NewLine,
            Let,
            Exit,
            If,
            Else,
            NewLine,
            NewLine,
            Assign,
            Plus,
            Minus,
            Asterisk,
            ForwardSlash,
            NewLine,
            Equal,
            NotEqual,
            Less,
            LessEqual,
            Greater,
            GreaterEqual,
            NewLine,
            NewLine,
            SCurly,
            ECurly,
            SCurly,
            ECurly,
            NewLine,
            SBrace,
            EBrace,
            SBrace,
            EBrace,
            NewLine,
            NewLine,
            EndOfFile,
        ];
        let mut lexer = Lexer::new(source);
        let mut i = 0;
        while !lexer.is_eof() {
            let tokentype = &expected_result[i];
            println!("{:?}, {:?}", tokentype, lexer.peek().tokentype);
            assert_eq!(lexer.peek().tokentype, *tokentype);
            lexer.consume().unwrap();
            i += 1;
        }
    }
}
