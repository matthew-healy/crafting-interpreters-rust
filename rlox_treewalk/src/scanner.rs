use crate::{
    error::{Error, Result},
    token::{HashableNumber, Token, TokenKind},
};
use peekmore::{PeekMore, PeekMoreIterator};
use phf::phf_map;
use std::str::Chars;

static KEYWORDS: phf::Map<&'static str, TokenKind> = phf_map! {
    "and" => TokenKind::And,
    "class" => TokenKind::Class,
    "else" => TokenKind::Else,
    "false" => TokenKind::False,
    "for" => TokenKind::For,
    "fun" => TokenKind::Fun,
    "if" => TokenKind::If,
    "nil" => TokenKind::Nil,
    "or" => TokenKind::Or,
    "print" => TokenKind::Print,
    "return" => TokenKind::Return,
    "super" => TokenKind::Super,
    "this" => TokenKind::This,
    "true" => TokenKind::True,
    "var" => TokenKind::Var,
    "while" => TokenKind::While,
};

enum ScannerResult {
    Next(Result<TokenKind>),
    Skip,
    NoMoreTokens,
}

pub struct Scanner<'a> {
    src: PeekMoreIterator<Chars<'a>>,
    lexeme_buffer: String,
    line: usize,
}

impl <'a> Iterator for Scanner<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Result<Token>> {
        let next = self.next_token_kind();

        let lexeme = self.lexeme_buffer.clone();
        self.lexeme_buffer.clear();

        match next {
            ScannerResult::Next(result) => {
                Some(result.map(|kind|
                    Token {
                        kind,
                        lexeme,
                        line: self.line,
                    }
                ))
            },
            ScannerResult::Skip => self.next(),
            ScannerResult::NoMoreTokens => None,
        }
    }
}

impl <'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.chars().peekmore(),
            lexeme_buffer: String::new(),
            line: 1,
        }
    }

    pub fn scan_tokens(self) -> Vec<Result<Token>> {
        let line = self.line;
        let mut tokens = self.collect::<Vec<Result<Token>>>();
        tokens.push(Ok(Token {
            kind: TokenKind::EndOfFile,
            lexeme: "".to_string(),
            line: line,
        }));
        tokens
    }

    fn next_token_kind(&mut self) -> ScannerResult {
        match self.src.next() {
            None => ScannerResult::NoMoreTokens,
            Some(next_char) => {
                self.lexeme_buffer.push(next_char);
                use TokenKind::*;
                match next_char {
                    '(' => ScannerResult::Next(Ok(LeftParen)),
                    ')' => ScannerResult::Next(Ok(RightParen)),
                    '{' => ScannerResult::Next(Ok(LeftBrace)),
                    '}' => ScannerResult::Next(Ok(RightBrace)),
                    ',' => ScannerResult::Next(Ok(Comma)),
                    '.' => ScannerResult::Next(Ok(Dot)),
                    '-' => ScannerResult::Next(Ok(Minus)),
                    '+' => ScannerResult::Next(Ok(Plus)),
                    ';' => ScannerResult::Next(Ok(Semicolon)),
                    '*' => ScannerResult::Next(Ok(Star)),
                    '!' => ScannerResult::Next(Ok(if self.does_next_match('=') { BangEqual } else { Bang })),
                    '=' => ScannerResult::Next(Ok(if self.does_next_match('=') { EqualEqual } else { Equal })),
                    '<' => ScannerResult::Next(Ok(if self.does_next_match('=') { LessEqual } else { Less })),
                    '>' => ScannerResult::Next(Ok(if self.does_next_match('=') { GreaterEqual } else { Greater })),
                    '/' => {
                        if self.does_next_match('/') { // is this a comment?
                            self.advance_until_match('\n');
                            ScannerResult::Skip
                        } else {
                            ScannerResult::Next(Ok(Slash))
                        }
                    },
                    ' ' | '\r' | '\t' => ScannerResult::Skip,
                    '\n' => {
                        self.line += 1;
                        ScannerResult::Skip
                    },
                    '"' => ScannerResult::Next(self.extract_string()),
                    c if c.is_digit(10) => ScannerResult::Next(self.extract_number()),
                    c if can_start_identifier(&c) => ScannerResult::Next(self.extract_identifier()),
                    c => ScannerResult::Next(Err(Error::lexical(self.line, format!("Unexpected character '{}'", c)))),
                }
            }
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

    fn extract_string(&mut self) -> Result<TokenKind> {
        let mut newline_count = 0;
        self.advance_until_match_for_each('"', |c| if c == '\n' { newline_count += 1 });
        self.line += newline_count;
        match self.src.next() {
            None => Err(Error::lexical(self.line, "Unterminated string literal.")),
            Some(q) => { // q here must be " due to iterate_until
                self.lexeme_buffer.push(q);
                Ok(TokenKind::String(self.lexeme_buffer.trim_matches('"').to_string()))
            },
        }
    }

    fn extract_number(&mut self) -> Result<TokenKind> {
        self.advance_until(|n| !n.is_digit(10));

        if let Some(&'.') = self.src.peek() {
            if let Some(maybe_digit) = self.src.peek_next() {
                if maybe_digit.is_digit(10) {
                    self.lexeme_buffer.push(self.src.next().unwrap());
                    self.advance_until(|n| !n.is_digit(10));
                }
            }
        }

        match self.lexeme_buffer.parse() {
            Err(_) => Err(Error::lexical(
                self.line,
                format!("Could not convert {} into a number", self.lexeme_buffer.clone())
            )),
            Ok(number) => Ok(TokenKind::Number(HashableNumber(number))),
        }
    }

    fn extract_identifier(&mut self) -> Result<TokenKind> {
        self.advance_until(|n| !is_part_of_valid_identifier(n));

        let text = self.lexeme_buffer.as_str();
        match KEYWORDS.get(text) {
            Some(token) => Ok(token.clone()),
            None => Ok(TokenKind::Identifier)
        }
    }

    fn advance_until_match(&mut self, c: char) {
        self.advance_until(|n| n == &c)
    }

    fn advance_until(&mut self, should_stop: impl Fn(&char) -> bool) {
        self.advance_until_for_each(should_stop, |_| {})
    }

    fn advance_until_match_for_each(
        &mut self,
        c: char,
        f: impl FnMut(char) -> ()
    ) {
        self.advance_until_for_each(|n| n == &c, f);
    }

    fn advance_until_for_each(
        &mut self,
        should_stop: impl Fn(&char) -> bool,
        mut f: impl FnMut(char) -> ()
    ) {
        let is_done = |nxt: Option<&char>| nxt.is_none() || should_stop(nxt.unwrap());
        while !is_done(self.src.peek()) {
            let next = self.src.next().unwrap();
            self.lexeme_buffer.push(next);
            f(next);
        }
    }
}

fn can_start_identifier(c: &char) -> bool {
    c.is_ascii_alphabetic() || c == &'_'
}

fn is_part_of_valid_identifier(c: &char) -> bool {
    can_start_identifier(c) || c.is_digit(10)
}