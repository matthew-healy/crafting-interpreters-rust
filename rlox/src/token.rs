#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) lexeme: String,
    pub(crate) line: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Identifier, String(String), Number(f64),

    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EndOfFile,
}