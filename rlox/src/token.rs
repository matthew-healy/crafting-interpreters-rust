use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) lexeme: String,
    pub(crate) line: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TokenKind {
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Identifier, String(String), Number(HashableNumber),

    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EndOfFile,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HashableNumber(pub(crate) f64);

// This is almost certainly a bad thing to do,
// but it serves our purposes for now.
impl Hash for HashableNumber {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Eq for HashableNumber {}