mod error;

pub use crate::error::{Error, Result};

use std::{iter::Peekable, str::Chars};

pub struct Scanner<'a> {
    src: Peekable<Chars<'a>>,
    lexeme_buffer: String,
    line: usize,
}

impl <'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.chars().peekable(),
            lexeme_buffer: String::new(),
            line: 1,
        }
    }

    pub fn scan_tokens(self) -> Result<Vec<Token>> {
        self.collect()
    }

    fn token_kind_from_char(&mut self, c: char) -> Option<Result<TokenKind>> {
        use TokenKind::*;
        match c {
            '(' => Some(Ok(LeftParen)),
            ')' => Some(Ok(RightParen)),
            '{' => Some(Ok(LeftBrace)),
            '}' => Some(Ok(RightBrace)),
            ',' => Some(Ok(Comma)),
            '.' => Some(Ok(Dot)),
            '-' => Some(Ok(Minus)),
            '+' => Some(Ok(Plus)),
            ';' => Some(Ok(Semicolon)),
            '*' => Some(Ok(Star)),
            '!' => Some(Ok(if self.does_next_match('=') { BangEqual } else { Bang })),
            '=' => Some(Ok(if self.does_next_match('=') { EqualEqual } else { Equal })),
            '<' => Some(Ok(if self.does_next_match('=') { LessEqual } else { Less })),
            '>' => Some(Ok(if self.does_next_match('=') { GreaterEqual } else { Greater })),
            '/' => {
                if self.does_next_match('/') {
                    self.consume_until('\n');
                    None
                } else {
                    Some(Ok(Slash))
                }
            },
            ' ' | '\r' | '\t' => None,
            '\n' => {
                self.line += 1;
                None
            }
            _ => Some(Err(Error::bad_syntax(self.line, format!("Unexpected character '{}'", c)))),
        }
    }

    fn does_next_match(&mut self, c: char) -> bool {
        match self.src.peek() {
            Some(next) if c == *next => {
                self.lexeme_buffer.push(self.src.next().unwrap());
                true
            }
            _ => false,
        }
    }

    fn consume_until(&mut self, c: char) {
        let is_done = |nxt: Option<&char>| nxt.is_none() || nxt == Some(&c);
        while !is_done(self.src.peek()) {
            self.src.next();
        }
    }
}

impl <'a> Iterator for Scanner<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Result<Token>> {
        let next_char = self.src.next()?;
        self.lexeme_buffer.push(next_char);

        let kind = self.token_kind_from_char(next_char);

        let lexeme = self.lexeme_buffer.clone();
        self.lexeme_buffer.clear();

        kind.map(|kind|
            kind.map(|kind| Token {
                kind,
                lexeme,
                line: self.line,
            })
        ).or_else(|| self.next())
    }
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    lexeme: String,
    line: usize,
}

#[derive(Debug)]
enum TokenKind {
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Identifier, String, Number,

    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EndOfFile,
}