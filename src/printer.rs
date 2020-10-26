use crate::{
    expr::{self, Expr},
    value::Value,
};

pub fn print(e: &Expr) -> String {
    let mut printer = AstPrinter {};
    e.accept(&mut printer)
}

struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let mut s = String::new();
        s.push('(');
        s.push_str(name);
    
        for e in exprs.iter() {
            s.push(' ');
            s.push_str(e.accept(self).as_str());
        }

        s.push(')');
        s
    }
}

impl expr::Visitor<String> for AstPrinter {
    fn visit_binary_expr(&mut self, e: &expr::Binary) -> String {
        self.parenthesize(
            e.op.lexeme.as_str(), 
            &[e.left.as_ref(), e.right.as_ref()]
        )
    }

    fn visit_grouping_expr(&mut self, e: &expr::Grouping) -> String {
        self.parenthesize(
            "group", 
            &[e.expression.as_ref()]
        )
    }

    fn visit_literal_expr(&mut self, e: &expr::Literal) -> String {
        match &e.value {
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
        }
    }

    fn visit_unary_expr(&mut self, e: &expr::Unary) -> String {
        self.parenthesize(
            e.op.lexeme.as_str(), 
            &[e.right.as_ref()]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenKind};

    #[test]
    fn string_literal() {
        let e = Expr::Literal(expr::Literal { value: Value::String("yes".into()) });
        assert_eq!("yes", print(&e));
    }

    #[test]
    fn grouped_number() {
        let e = Expr::Grouping(expr::Grouping {
            expression: Box::new(Expr::Literal(expr::Literal {
                value: Value::Number(531.9)
            }))
        });
        assert_eq!("(group 531.9)", print(&e));
    }

    #[test]
    fn binary_expression_with_unary_and_grouping_sub_exprs() {
        let e = Expr::Binary(expr::Binary {
            left: Box::new(Expr::Unary(expr::Unary {
                op: Token { kind: TokenKind::Minus, lexeme: "-".into(), line: 1 },
                right: Box::new(Expr::Literal(expr::Literal { value: Value::Number(123.0) })),
            })),
            op: Token { kind: TokenKind::Star, lexeme: "*".into(), line: 1},
            right: Box::new(Expr::Grouping(expr::Grouping {
                expression: Box::new(Expr::Literal(expr::Literal { value: Value::Number(45.67) })),
            }))
        });
        assert_eq!("(* (- 123) (group 45.67))", print(&e));
    }
}