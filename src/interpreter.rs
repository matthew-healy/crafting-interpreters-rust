use std::io::Write;

use crate::{
    error::{Error, Result},
    expr::{self, Expr},
    token::{TokenKind, Token},
};

type LoxValue = expr::LoxLiteral;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter{}
    }

    pub fn interpret<W: Write>(&mut self, w: &mut W, e: &Expr) -> Result<()> {
        self.evaluate(e)
            .and_then(|r| 
                writeln!(w, "{}", r).map_err(|e| e.into())
            )
    }
    
    fn evaluate(&mut self, e: &Expr) -> Result<LoxValue> {
        e.accept(self)
    }
}

impl expr::Visitor<Result<LoxValue>> for Interpreter {
    fn visit_binary_expr(&mut self, e: &expr::Binary) -> Result<LoxValue> {
        let left = self.evaluate(e.left.as_ref())?;
        let right = self.evaluate(e.right.as_ref())?;
        let kind = e.op.kind.clone();

        use expr::LoxLiteral::{Number, String, Bool};
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

    fn visit_grouping_expr(&mut self, e: &expr::Grouping) -> Result<LoxValue> {
        self.evaluate(&e.expression)
    }

    fn visit_literal_expr(&mut self, e: &expr::Literal) -> Result<LoxValue> {
        Ok(e.value.clone())
    }

    fn visit_unary_expr(&mut self, e: &expr::Unary) -> Result<LoxValue> {
        let right = self.evaluate(e.right.as_ref())?;
        let kind = e.op.kind.clone();

        use expr::LoxLiteral::*;
        match (kind, right) {
            (TokenKind::Minus, Number(right)) => Ok(Number(-right)),
            (TokenKind::Minus, _) => Err(Error::runtime(e.op.clone(), "Operand must be a number.")),
            (TokenKind::Bang, right) => Ok(Bool(!right.is_truthy())),
            _ => unreachable!(),
        }
    }
}

fn compute_if_numbers<T: Into<LoxValue>>(
    op: &Token, 
    left: LoxValue, 
    right: LoxValue, 
    f: impl Fn(f64, f64) -> T
) -> Result<LoxValue> {
    use expr::LoxLiteral::Number;
    if let Number(left) = left {
        if let Number(right) = right { 
            return Ok(f(left, right).into())
        }
    }
    Err(Error::runtime(op.clone(), "Operands must be numbers."))
}

impl LoxValue {
    fn is_equal(&self, other: &expr::LoxLiteral) -> bool {
        use expr::LoxLiteral::*;
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
        use expr::LoxLiteral::*;
        match self {
            Bool(false) | Nil => false,
            _ => true,
        }
    }
}