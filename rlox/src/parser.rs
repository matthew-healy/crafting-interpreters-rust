use std::iter::Peekable;

use crate::{
    error::{Error, Result},
    expr::{Expr},
    stmt::{self, Stmt},
    token::*,
    value,
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

const TERM_TOKENS: &'static [&'static TokenKind] = &[
    &TokenKind::Minus,
    &TokenKind::Plus,
];

const FACTOR_TOKENS: &'static [&'static TokenKind] = &[
    &TokenKind::Star, 
    &TokenKind::Slash,
];

const UNARY_TOKENS: &'static [&'static TokenKind] = &[
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

        let result = if self.match_single(&TokenKind::Class).is_some() {
            self.class_declaration()
        } else if self.match_single(&TokenKind::Fun).is_some() {
            self.function("function").map(|f| Stmt::Function(f))
        } else if self.match_single(&TokenKind::Var).is_some() {
            self.var_declaration()
        } else {
            self.statement()
        };

        if result.is_err() {
            self.synchronise();
        }

        Some(result)
    }

    fn class_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(&TokenKind::Identifier, "Expected class name.")?;
        self.consume(&TokenKind::LeftBrace, "Expected '{' before class body.")?;

        let mut methods = Vec::new();
        while self.tokens.peek().map(|t| &t.kind) != Some(&TokenKind::RightBrace) {
            methods.push(self.function("method")?);
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after class body.")?;

        Ok(Stmt::new_class(name, methods))
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(&TokenKind::Identifier, "Expected variable name.")?;

        let initializer = if self.match_single(&TokenKind::Equal).is_some() {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenKind::Semicolon, "Expected ';' after variable declaration.")?;
        Ok(Stmt::new_var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_single(&TokenKind::For).is_some() {
            self.for_statement()
        } else if self.match_single(&TokenKind::If).is_some() {
            self.if_statement()
        } else if self.match_single(&TokenKind::Print).is_some() {
            self.print_statement()
        } else if let Some(token) = self.match_single(&TokenKind::Return) {
            self.return_statement(token)
        } else if self.match_single(&TokenKind::While).is_some() {
            self.while_statement()
        } else if self.match_single(&TokenKind::LeftBrace).is_some() {
            Ok(Stmt::new_block(self.block()?))
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
        } else { Expr::new_literal(value::Literal::from(true)) };

        self.consume(&TokenKind::Semicolon, "Expected ';' after loop condition.")?;

        let increment = if !self.check_next(&TokenKind::RightParen) {
            Some(Stmt::new_expression(self.expression()?))
        } else { None };

        self.consume(&TokenKind::RightParen, "Expected ')' after for clauses.")?;

        let body = self.statement()?;
        let body = Box::new(match increment {
            Some(i) => Stmt::new_block(vec![body, i]),
            None => body,
        });
        let while_loop = Stmt::new_while(condition, body);
        let while_loop = match initializer {
            Some(i) => Stmt::new_block(vec![i, while_loop]),
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

        Ok(Stmt::new_if(condition, then_branch, else_branch))
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expression = self.expression()?;
        self.consume(&TokenKind::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::new_print(expression))
    }

    fn return_statement(&mut self, token: Token) -> Result<Stmt> {
        let return_value = if !self.check_next(&TokenKind::Semicolon) {
            self.expression()?
        } else { Expr::new_literal(value::Literal::Nil) };
        self.consume(&TokenKind::Semicolon, "Expected ';' after return value.")?;
        Ok(Stmt::new_return(token, return_value))
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RightParen, "Expected ')' after condition.")?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::new_while(condition, body))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expression = self.expression()?;
        self.consume(&TokenKind::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::new_expression(expression))
    }

    fn function(&mut self, kind: &str) -> Result<stmt::Function> {
        let name = self.consume(
            &TokenKind::Identifier,
            format!("Expected {} name", kind).as_str()
        )?;
        self.consume(
            &TokenKind::LeftParen,
            format!("Expected '(' after {} name.", kind).as_str()
        )?;

        let mut params = Vec::new();
        if !self.check_next(&TokenKind::RightParen) {
            loop {
                params.push(self.consume(&TokenKind::Identifier, "Expected a parameter name.")?);
                if self.match_single(&TokenKind::Comma).is_none() { break }
            }
        }

        if params.len() > 255 {
            // Another error bubbled up instead of just reported.
            return Err(Error::syntactic(name, "Cannot have more than 255 parameters."))
        }

        self.consume(&TokenKind::RightParen, "Expected ')' after parameters.")?;
        self.consume(
            &TokenKind::LeftBrace,
            format!("Expect '{{' before {} body.", kind).as_str()
        )?;

        Ok(stmt::Function { name, params, body: self.block()? })
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
                Ok(Expr::new_assign(lhs.name, Box::new(value)))
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
            e = Expr::new_logical(Box::new(e), op, right);
        }

        Ok(e)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut e = self.equality()?;

        while let Some(op) = self.match_single(&TokenKind::And) {
            let right = Box::new(self.equality()?);
            e = Expr::new_logical(Box::new(e), op, right);
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
            Ok(Expr::new_unary(token, right))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut e = self.primary()?;

        loop {
            if self.match_single(&TokenKind::LeftParen).is_some() {
                e = self.finish_call(e)?;
            } else if self.match_single(&TokenKind::Dot).is_some() {
                let name = self.consume(&TokenKind::Identifier, "Expected property name after '.'.")?;
                e = Expr::new_get(Box::new(e), name);
            } else {
                break
            }
        }

        Ok(e)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut args = Vec::new();

        if !self.check_next(&TokenKind::RightParen) {
            args.push(self.expression()?);
            while self.match_single(&TokenKind::Comma).is_some() {
                args.push(self.expression()?);
            }
        }
        let paren = self.consume(
            &TokenKind::RightParen,
            "Expected ')' after arguments."
        )?;

        if args.len() > 255 {
            // Another situation where jlox merely reports the error & rlox bubbles it up.
            Err(Error::syntactic(paren, "Function cannot have more than 255 arguments."))
        } else {
            Ok(Expr::new_call(Box::new(callee), paren, args))
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        let (nxt, kind) = {
            let next = self.tokens.next().ok_or(Error::unexpected())?;
            let kind = next.kind.clone();
            (next, kind)
        };

        match kind {
            TokenKind::True => Ok(Expr::new_literal(true.into())),
            TokenKind::False => Ok(Expr::new_literal(false.into())),
            TokenKind::Nil => Ok(Expr::new_literal(value::Literal::Nil)),
            TokenKind::Number(n) => Ok(Expr::new_literal(n.into())),
            TokenKind::String(s) => Ok(Expr::new_literal(s.into())),
            TokenKind::Identifier => Ok(Expr::new_variable(nxt)),
            TokenKind::LeftParen => {
                let expression = Box::new(self.expression()?);
                 self.consume(&TokenKind::RightParen, "Expected ')' after expression.")?;
                 Ok(Expr::new_grouping(expression))
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
            e = Expr::new_binary(Box::new(e), token, right)
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
            Expr::new_literal(value::Literal::String("abc".into()))
        )
    }

    #[test]
    fn number_literal_token() -> io::Result<()> {
        assert_tokens_parse_to_expr(
            vec![
                Token { kind: TokenKind::Number(HashableNumber(5.1)), lexeme: "".into(), line: 1 },
            ], 
            Expr::new_literal(value::Literal::Number(HashableNumber(5.1)))
        )
    }

    #[test]
    fn nil_literal_token() -> io::Result<()> {
        assert_tokens_parse_to_expr(
            vec![
                Token { kind: TokenKind::Nil, lexeme: "".into(), line: 1 }, 
            ], 
            Expr::new_literal(value::Literal::Nil)
        )
    }

    #[test]
    fn bool_literal_tokens() -> io::Result<()> {
        for (kind, expected) in [(TokenKind::True, value::Literal::Bool(true)), (TokenKind::False, value::Literal::Bool(false))].iter() {
            assert_tokens_parse_to_expr(
                vec![
                    Token { kind: kind.clone(), lexeme: "".into(), line: 1 }, 
                ], 
                Expr::new_literal(expected.clone())
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
            Expr::new_unary(not, Box::new(Expr::make(true)))
        )
    }

    impl Token {
        fn make(kind: TokenKind) -> Token {
            Token { kind, lexeme: "".into(), line: 0 }
        }
    }

    impl Expr {
        fn make(b: bool) -> Expr {
            Expr::new_literal(value::Literal::Bool(b))
        }
    }
}