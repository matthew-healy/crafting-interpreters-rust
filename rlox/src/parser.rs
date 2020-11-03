use std::iter::Peekable;

use crate::{
    error::{Error, Result},
    expr::*,
    stmt::{self, Stmt},
    token::*,
    value::Value,
};

const EQUALITY_TOKENS: &'static [&'static TokenKind] = &[
    &TokenKind::BangEqual, 
    &TokenKind::EqualEqual,
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

    pub fn parse(&mut self) -> Vec<Result<Stmt>> {
        let mut statements = Vec::new();
        while let Some(statement) = self.declaration() {
            statements.push(statement);
        }
        statements
    }

    fn declaration(&mut self) -> Option<Result<Stmt>> {
        if self.tokens.peek() == None { return None }

        let result = if self.match_single(&TokenKind::Var).is_some() {
            self.var_declaration()
        } else {
            self.statement()
        };

        if result.is_err() {
            self.synchronise();
        }

        Some(result)
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(&TokenKind::Identifier, "Expected variable name.")?;

        let initializer = if self.match_single(&TokenKind::Equal).is_some() {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenKind::Semicolon, "Expected ';' after variable declaration.")?;
        Ok(Stmt::Var(stmt::Var { name, initializer }))
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_single(&TokenKind::For).is_some() {
            self.for_statement()
        } else if self.match_single(&TokenKind::If).is_some() {
            self.if_statement()
        } else if self.match_single(&TokenKind::Print).is_some() {
            self.print_statement()
        } else if self.match_single(&TokenKind::While).is_some() {
            self.while_statement()
        } else if self.match_single(&TokenKind::LeftBrace).is_some() {
            Ok(Stmt::Block(stmt::Block { statements: self.block()? }))
        } else if self.tokens.peek().is_some() {
            self.expression_statement()
        } else {
            Err(Error::unexpected())
        }
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'for'.")?;

        let initializer = if self.match_single(&TokenKind::Semicolon).is_some() {
            None
        } else if self.match_single(&TokenKind::Var).is_some() {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check_next(&TokenKind::Semicolon) {
            self.expression()?
        } else { Expr::Literal(Literal { value: Value::from(true) }) };

        self.consume(&TokenKind::Semicolon, "Expected ';' after loop condition.")?;

        let increment = if !self.check_next(&TokenKind::RightParen) {
            Some(Stmt::Expression(stmt::Expression { expression: self.expression()? }))
        } else { None };

        self.consume(&TokenKind::RightParen, "Expected ')' after for clauses.")?;

        let body = self.statement()?;
        let body = Box::new(match increment {
            Some(i) => Stmt::Block(stmt::Block { statements: vec![body, i]}),
            None => body,
        });
        let while_loop = Stmt::While(stmt::While { condition, body });
        let while_loop = match initializer {
            Some(i) => Stmt::Block(stmt::Block { statements: vec![i, while_loop] }),
            None => while_loop,
        };

        Ok(while_loop)
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RightParen, "Expected ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_single(&TokenKind::Else).is_some() {
            Some(Box::new(self.statement()?))
        } else { None };

        Ok(Stmt::If(stmt::If { condition, then_branch, else_branch }))
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expression = self.expression()?;
        self.consume(&TokenKind::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Print(stmt::Print { expression }))
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RightParen, "Expected ')' after condition.")?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While(stmt::While { condition, body }))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expression = self.expression()?;
        self.consume(&TokenKind::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression(stmt::Expression { expression }))
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while self.tokens.peek().map(|t| &t.kind) != Some(&TokenKind::RightBrace) {
            match self.declaration() {
                Some(d) => statements.push(d?),
                None => break
            }
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after block.")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;
        if let Some(equals) = self.match_single(&TokenKind::Equal) {
            if let Expr::Variable(lhs) = expr {
                let value = self.assignment()?;
                Ok(Expr::Assign(Assign { name: lhs.name, value: Box::new(value) }))
            } else {
                // N.b. in jlox this error doesn't throw - it just returns
                // the expr we already parsed on the lhs. This is inconvenient
                // with rlox's current error-handling. I'm also not sure the
                // overall difference in behaviour is worth the refactor this
                // would require.
                Err(Error::syntactic(equals, "Invalid assignment target."))
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr> {
        let mut e = self.and()?;

        while let Some(op) = self.match_single(&TokenKind::Or) {
            let right = Box::new(self.and()?);
            e = Expr::Logical(Logical { left: Box::new(e), op, right });
        }

        Ok(e)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut e = self.equality()?;

        while let Some(op) = self.match_single(&TokenKind::And) {
            let right = Box::new(self.equality()?);
            e = Expr::Logical(Logical { left: Box::new(e), op, right });
        }

        Ok(e)
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
            TokenKind::True => Ok(Expr::Literal(Literal { value: true.into() })),
            TokenKind::False => Ok(Expr::Literal(Literal { value: false.into() })),
            TokenKind::Nil => Ok(Expr::Literal(Literal { value: Value::Nil })),
            TokenKind::Number(n) => Ok(Expr::Literal(Literal { value: n.into() })),
            TokenKind::String(s) => Ok(Expr::Literal(Literal { value: Value::String(s) })),
            TokenKind::Identifier => Ok(Expr::Variable(Variable { name: nxt })),
            TokenKind::LeftParen => {
                let expression = Box::new(self.expression()?);
                 self.consume(&TokenKind::RightParen, "Expected ')' after expression.")?;
                 Ok(Expr::Grouping(Grouping { expression }))
            },
            _ => Err(Error::syntactic(nxt, ""))
        }
    }

    fn check_next(&mut self, kind: &TokenKind) -> bool {
        self.tokens.peek()
            .map(|t| &t.kind == kind)
            .unwrap_or(false)
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
        if self.check_next(kind) {
            self.tokens.next() 
        } else { 
            None 
        }
    }

    fn match_any(&mut self, kinds: &[&TokenKind]) -> Option<Token> {
        kinds.iter().find_map(|k| self.match_single(k) )
    }

    fn synchronise(& mut self) {
        loop {
            let current = self.tokens.next();

            if let Some(token) = current {
                use TokenKind::*;
                if token.kind == Semicolon { break }

                if let Some(next) = self.tokens.peek() {
                    match next.kind {
                        Class | Fun | Var
                        | For | If | While
                        | Print | Return => break,
                        _ => continue,
                    }
                }
            } else { break }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn assert_tokens_parse_to_expr(tokens: Vec<Token>, expr: Expr) -> io::Result<()> {
        let mut parser = Parser::new(tokens.into_iter());
        let parsed = parser.expression()?;
        assert_eq!(expr, parsed);
        Ok(())
    }

    #[test]
    fn string_literal_token() -> io::Result<()> {
        assert_tokens_parse_to_expr(
            vec![
                Token { kind: TokenKind::String("abc".into()), lexeme: "".into(), line: 1 }, 
            ], 
            Expr::Literal(Literal { value: Value::String("abc".into()) })
        )
    }

    #[test]
    fn number_literal_token() -> io::Result<()> {
        assert_tokens_parse_to_expr(
            vec![
                Token { kind: TokenKind::Number(5.1), lexeme: "".into(), line: 1 }, 
            ], 
            Expr::Literal(Literal { value: Value::Number(5.1) })
        )
    }

    #[test]
    fn nil_literal_token() -> io::Result<()> {
        assert_tokens_parse_to_expr(
            vec![
                Token { kind: TokenKind::Nil, lexeme: "".into(), line: 1 }, 
            ], 
            Expr::Literal(Literal { value: Value::Nil })
        )
    }

    #[test]
    fn bool_literal_tokens() -> io::Result<()> {
        for (kind, expected) in [(TokenKind::True, Value::Bool(true)), (TokenKind::False, Value::Bool(false))].iter() {
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
            Expr::Literal(Literal { value: Value::Bool(b) })
        }
    }
}