use std::io::Write;

use crate::{
    environment::Environment,
    error::{Error, Result},
    expr::{self, Expr},
    stmt::{self, Stmt},
    token::{TokenKind, Token},
    value::Value,
};


pub struct Interpreter<W> {
    environment: Environment,
    writer: W,
}

impl <W: Write> Interpreter<W> {
    pub fn new(writer: W) -> Self {
        Interpreter { environment: Environment::new(), writer }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<()> {
        for s in statements.iter() {
            self.execute(s)?;
        }
        Ok(())
    }

    fn execute(&mut self, s: &Stmt) -> Result<()> {
        s.accept(self)?;
        Ok(())
    }
    
    fn evaluate(&mut self, e: &Expr) -> Result<Value> {
        e.accept(self)
    }
}

impl <W: Write> stmt::Visitor<Result<()>> for Interpreter<W> {
    fn visit_print_stmt(&mut self, p: &stmt::Print) -> Result<()> {
        let value = self.evaluate(&p.expression)?;
        writeln!(self.writer, "{}", value)?;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, e: &stmt::Expression) -> Result<()> {
        self.evaluate(&e.expression)?;
        Ok(())
    }

    fn visit_var_stmt(&mut self, v: &stmt::Var) -> Result<()> {
        let value = if let Some(initializer) = &v.initializer {
            self.evaluate(initializer)?
        } else {
            Value::Nil
        };

        let var_name = v.name.lexeme.clone();

        self.environment.define(var_name, value);
        Ok(())
    }
}

impl <W: Write> expr::Visitor<Result<Value>> for Interpreter<W> {
    fn visit_binary_expr(&mut self, e: &expr::Binary) -> Result<Value> {
        let left = self.evaluate(e.left.as_ref())?;
        let right = self.evaluate(e.right.as_ref())?;
        let kind = e.op.kind.clone();

        use Value::{Number, String, Bool};
        match kind {
            TokenKind::Minus => compute_if_numbers(&e.op, left, right, |l, r| l - r),
            TokenKind::Plus => {
                if let Number(left) = left {
                    if let Number(right) = right {
                        return Ok(Number(left + right))
                    }
                } 
                if let String(mut left) = left {
                    if let String(right) = right {
                        left.push_str(right.as_str());
                        return Ok(String(left))
                    } 
                } 
                Err(Error::runtime(e.op.clone(), "Operands must be two numbers or two strings."))
            }
            TokenKind::Slash => compute_if_numbers(&e.op, left, right, |l, r| l / r),
            TokenKind::Star => compute_if_numbers(&e.op, left, right, |l, r| l * r),
            TokenKind::Greater => compute_if_numbers(&e.op, left, right, |l, r| l > r),
            TokenKind::GreaterEqual => compute_if_numbers(&e.op, left, right, |l, r| l >= r),
            TokenKind::Less => compute_if_numbers(&e.op, left, right, |l, r| l < r),
            TokenKind::LessEqual => compute_if_numbers(&e.op, left, right, |l, r| l <= r),
            TokenKind::EqualEqual => Ok(Bool(left.is_equal(&right))),
            TokenKind::BangEqual => Ok(Bool(!left.is_equal(&right))),
            _ => unreachable!(),
        }
    }

    fn visit_grouping_expr(&mut self, e: &expr::Grouping) -> Result<Value> {
        self.evaluate(&e.expression)
    }

    fn visit_literal_expr(&mut self, e: &expr::Literal) -> Result<Value> {
        Ok(e.value.clone())
    }

    fn visit_unary_expr(&mut self, e: &expr::Unary) -> Result<Value> {
        let right = self.evaluate(e.right.as_ref())?;
        let kind = e.op.kind.clone();

        use Value::*;
        match (kind, right) {
            (TokenKind::Minus, Number(right)) => Ok(Number(-right)),
            (TokenKind::Minus, _) => Err(Error::runtime(e.op.clone(), "Operand must be a number.")),
            (TokenKind::Bang, right) => Ok(Bool(!right.is_truthy())),
            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(&mut self, e: &expr::Variable) -> Result<Value> {
        self.environment.get(&e.name)
    }
}

fn compute_if_numbers<T: Into<Value>>(
    op: &Token, 
    left: Value,
    right: Value,
    f: impl Fn(f64, f64) -> T
) -> Result<Value> {
    use Value::Number;
    if let Number(left) = left {
        if let Number(right) = right { 
            return Ok(f(left, right).into())
        }
    }
    Err(Error::runtime(op.clone(), "Operands must be numbers."))
}

impl Value {
    fn is_equal(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(s), Bool(o)) => s == o,
            (Number(s), Number(o)) => {
                // Lox follows Java's Double convention in that NaN == NaN
                // is true whereas f64 follows IEEE 754.
                if s.is_nan() && o.is_nan() {
                    true
                } else {
                    s == o
                }
            },
            (String(s), String(o)) => s == o,
            _ => false,
        }
    }

    fn is_truthy(&self) -> bool {
        use Value::*;
        match self {
            Bool(false) | Nil => false,
            _ => true,
        }
    }
}