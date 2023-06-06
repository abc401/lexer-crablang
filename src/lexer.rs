use std::fs::read_to_string;

#[derive(Debug)]
pub enum Token {
    Ident(String),
    Keyword(KeywordType),
    Literal(String),
    OpenParen,
    CloseParen,
    OpenCurlyBrace,
    CloseCurlyBrace,
    Comma,
    Equals,
    Invalid(char),
}

#[derive(Debug)]
pub enum KeywordType {
    Fun,
    Return,
    Let,
}

impl KeywordType {
    fn from_str(str: &str) -> Option<Self> {
        use KeywordType::*;

        match str {
            "fun" => Some(Fun),
            "return" => Some(Return),
            "let" => Some(Let),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Lexer {
    contents: Vec<char>,
    next_index: usize,
}

impl Lexer {
    pub fn new(contents: String) -> Self {
        return Self {
            contents: contents.chars().collect(),
            next_index: 0,
        };
    }

    pub fn from_file(path: &str) -> Self {
        let contents = read_to_string(path).expect("Invalid path!");
        return Self::new(contents);
    }

    fn peek_ch(&self) -> Option<char> {
        let len = self.contents.len();
        if !(0..len).contains(&self.next_index) {
            return None;
        }
        return Some(self.contents[self.next_index]);
    }

    fn next_ch_eq(&self, ch: char) -> bool {
        if let Some(peeked) = self.peek_ch() {
            return peeked == ch;
        }
        return false;
    }

    fn next_ch(&mut self) -> Option<char> {
        let ch = self.peek_ch()?;
        self.next_index += 1;
        return Some(ch);
    }

    fn capture_while<F>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut capture = String::new();
        while let Some(ch) = self.peek_ch() {
            if predicate(ch) {
                capture.push(ch);
                self.next_ch();
            } else {
                break;
            }
        }
        return capture;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_ch() {
            if !ch.is_ascii_whitespace() {
                return;
            }
            self.next_ch();
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        use Token::*;

        self.skip_whitespace();
        let ch = self.next_ch()?;
        let token = match ch {
            '(' => OpenParen,
            ')' => CloseParen,
            '{' => OpenCurlyBrace,
            '}' => CloseCurlyBrace,
            ',' => Comma,
            '=' => Equals,
            ch if ch.is_numeric() => {
                let mut literal =
                    String::from(ch) + self.capture_while(|c| c.is_numeric()).as_str();
                if self.next_ch_eq('.') {
                    self.next_ch();
                    literal += ".";
                    literal += self.capture_while(|c| c.is_numeric()).as_str();
                }
                Literal(literal)
            }
            ch if ch.is_alphabetic() || ch == '_' => {
                let mut ident = String::from(ch);
                while let Some(ch) = self.peek_ch() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(self.next_ch().unwrap());
                    } else {
                        break;
                    }
                }

                if let Some(keyword_type) = KeywordType::from_str(ident.as_str()) {
                    Keyword(keyword_type)
                } else {
                    Ident(ident)
                }
            }
            _ => Token::Invalid(ch),
        };
        return Some(token);
    }
}
