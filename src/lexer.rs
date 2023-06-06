use std::{fs::read_to_string};

#[derive(Debug)]
pub enum Token {
    Ident(String),
    KeyWord(KeyWordType),
    OpenParen,
    CloseParen,
    OpenCurlyBrace,
    CloseCurlyBrace,
    Comma,
    Invalid
}


#[derive(Debug)]
pub enum KeyWordType {
    Fun,
    Return
}

impl KeyWordType {
    fn from_str(str: &str) -> Option<Self> {
        use KeyWordType::*;

        match str {
            "fun" => Some(Fun),
            "return" => Some(Return),
            _ => None
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

    fn peek_ch(& self) -> Option<char> {
        let len = self.contents.len();
        if !(0..len).contains(&self.next_index) {
            return None;
        }
        return Some(self.contents[self.next_index]);
    }

    fn next_ch(&mut self) -> Option<char> {
        let ch = self.peek_ch()?;
        self.next_index += 1;
        return Some(ch);
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_ch() {
            if !ch.is_ascii_whitespace() {
                return
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
            ch if ch.is_alphabetic() || ch == '_' => {
                let mut ident = String::from(ch);
                while let Some(ch) = self.peek_ch() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(self.next_ch().unwrap());
                    } else {
                        break;
                    }
                }

                if let Some(keyword_type) = KeyWordType::from_str(ident.as_str()) {
                    return Some(KeyWord(keyword_type));
                }
                
                Ident(ident)
            }
            _ => Token::Invalid
        };
        return Some(token);
    }
}