use std::iter::Peekable;

use crate::{
    error::{Error, Result},
    expr::*,
    token::*,
};

const EQUALITY_TOKENS: &'static [&'static TokenKind] = &[
    &TokenKind::BangEqual, 
    &TokenKind::Equal,
];

const COMPARISON_TOKENS: &'static [&'static TokenKind] = &[
    &TokenKind::Greater, 
    &TokenKind::GreaterEqual, 
    &TokenKind::Less, 
    &TokenKind::LessEqual,
];

static TERM_TOKENS: &'static [&'static TokenKind] = &[
    &TokenKind::Minus,
    &TokenKind::Plus,
];

static FACTOR_TOKENS: &'static [&'static TokenKind] = &[
    &TokenKind::Star, 
    &TokenKind::Slash,
];

static UNARY_TOKENS: &'static [&'static TokenKind] = &[
    &TokenKind::Bang,
    &TokenKind:: Minus,
];

pub struct Parser<T> {
    tokens: T,
}

impl <T: Iterator<Item = Token>> Parser<Peekable<T>> {
    pub fn new(tokens: T) -> Self {
        let tokens = tokens.peekable();
        Parser { tokens }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        self.match_binary_precedence_with_tokens(
            Self::comparison, 
            EQUALITY_TOKENS
        )
    }

    fn comparison(&mut self) -> Result<Expr> {
        self.match_binary_precedence_with_tokens(
            Self::term, 
            COMPARISON_TOKENS
        )
    }

    fn term(&mut self) -> Result<Expr> {
        self.match_binary_precedence_with_tokens(
            Self::factor, 
            TERM_TOKENS
        )
    }
    
    fn factor(&mut self) -> Result<Expr> {
        self.match_binary_precedence_with_tokens(
            Self::unary, 
            FACTOR_TOKENS
        )
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Some(token) = self.match_any(UNARY_TOKENS) {
            let right = Box::new(self.unary()?);
            Ok(Expr::Unary(Unary { op: token, right }))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        let (nxt, kind) = {
            let next = self.tokens.next().ok_or(Error::unexpected())?;
            let kind = next.kind.clone();
            (next, kind)
        };

        match kind {
            TokenKind::True => Ok(Expr::Literal(Literal { value: LoxLiteral::Bool(true) })),
            TokenKind::False => Ok(Expr::Literal(Literal { value: LoxLiteral::Bool(false) })),
            TokenKind::Nil => Ok(Expr::Literal(Literal { value: LoxLiteral::Nil })),
            TokenKind::Number(n) => Ok(Expr::Literal(Literal { value: LoxLiteral::Number(n) })),
            TokenKind::String(s) => Ok(Expr::Literal(Literal { value: LoxLiteral::String(s) })),
            TokenKind::LeftParen => {
                let expression = Box::new(self.expression()?);
                 self.consume(&TokenKind::RightParen, "Expected ')' after expression.")?;
                 Ok(Expr::Grouping(Grouping { expression }))
            },
            _ => Err(Error::syntactic(nxt, ""))
        }
    }

    fn consume(&mut self, kind: &TokenKind, error_msg: &str) -> Result<Token> {
        if let Some(token) = self.match_single(kind) {
            Ok(token)
        } else {
            Err(match self.tokens.next() {
                Some(t) => Error::syntactic(t, error_msg),
                None => Error::unexpected(),
            })
        }
    }

    fn match_binary_precedence_with_tokens(
        &mut self, 
        parse: impl Fn(&mut Self) -> Result<Expr>, 
        kinds: &[&TokenKind]
    ) -> Result<Expr> {
        let mut e = parse(self)?;

        while let Some(token) = self.match_any(kinds) {
            let right = Box::new(parse(self)?);
            e = Expr::Binary(Binary { left: Box::new(e), op: token, right })
        }

        Ok(e)
    }

    fn match_single(&mut self, kind: &TokenKind) -> Option<Token> {
        let nxt = self.tokens.peek()?;
        if kind == &nxt.kind { 
            self.tokens.next() 
        } else { 
            None 
        }
    }

    fn match_any(&mut self, kinds: &[&TokenKind]) -> Option<Token> {
        kinds.iter().find_map(|k| self.match_single(k) )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn assert_tokens_parse_to_expr(tokens: Vec<Token>, expr: Expr) -> io::Result<()> {
        let mut parser = Parser::new(tokens.into_iter());
        assert_eq!(expr, parser.parse()?);
        Ok(())
    }

    #[test]
    fn string_literal_token() -> io::Result<()> {
        assert_tokens_parse_to_expr(
            vec![
                Token { kind: TokenKind::String("abc".into()), lexeme: "".into(), line: 1 }, 
            ], 
            Expr::Literal(Literal { value: LoxLiteral::String("abc".into()) })
        )
    }

    #[test]
    fn number_literal_token() -> io::Result<()> {
        assert_tokens_parse_to_expr(
            vec![
                Token { kind: TokenKind::Number(5.1), lexeme: "".into(), line: 1 }, 
            ], 
            Expr::Literal(Literal { value: LoxLiteral::Number(5.1) })
        )
    }

    #[test]
    fn nil_literal_token() -> io::Result<()> {
        assert_tokens_parse_to_expr(
            vec![
                Token { kind: TokenKind::Nil, lexeme: "".into(), line: 1 }, 
            ], 
            Expr::Literal(Literal { value: LoxLiteral::Nil })
        )
    }

    #[test]
    fn bool_literal_tokens() -> io::Result<()> {
        for (kind, expected) in [(TokenKind::True, LoxLiteral::Bool(true)), (TokenKind::False, LoxLiteral::Bool(false))].iter() {
            assert_tokens_parse_to_expr(
                vec![
                    Token { kind: kind.clone(), lexeme: "".into(), line: 1 }, 
                ], 
                Expr::Literal(Literal { value: expected.clone() })
            )?;
        }
        Ok(())
    }

    #[test]
    fn unary_op_tokens() -> io::Result<()> {
        let not = Token::make(TokenKind::Bang); 
        assert_tokens_parse_to_expr(
            vec![
                not.clone(),
                Token::make(TokenKind::True),
            ],
            Expr::Unary(Unary { op: not, right: Box::new(Expr::make(true)) })
        )
    }

    impl Token {
        fn make(kind: TokenKind) -> Token {
            Token { kind, lexeme: "".into(), line: 0 }
        }
    }

    impl Expr {
        fn make(b: bool) -> Expr {
            Expr::Literal(Literal { value: LoxLiteral::Bool(b) })
        }
    }
}