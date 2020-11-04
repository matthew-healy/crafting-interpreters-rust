use std::{
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

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
        let mut environment = Environment::new();
        environment.define("clock", Value::new_native_fn(&|| {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time since epoch should never be negative")
                .as_millis();
            Value::from(time as f64)
        }));
        Interpreter { environment, writer }
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

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<()> {
        self.environment.push_child_env();
        for statement in statements {
            // Reset the environment before returning an error.
            if let Err(error) = self.execute(statement) {
                self.environment.pop_child_env();
                return Err(error)
            }
        }
        self.environment.pop_child_env();
        Ok(())
    }
    
    fn evaluate(&mut self, e: &Expr) -> Result<Value> {
        e.accept(self)
    }
}

impl <W: Write> stmt::Visitor<Result<()>> for Interpreter<W> {
    fn visit_block_stmt(&mut self, b: &stmt::Block) -> Result<()> {
        self.execute_block(&b.statements)
    }

    fn visit_expression_stmt(&mut self, e: &stmt::Expression) -> Result<()> {
        self.evaluate(&e.expression)?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, i: &stmt::If) -> Result<()> {
        if self.evaluate(&i.condition)?.is_truthy() {
            self.execute(&i.then_branch)?;
        } else if let Some(else_branch) = &i.else_branch {
            self.execute(&else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, p: &stmt::Print) -> Result<()> {
        let value = self.evaluate(&p.expression)?;
        writeln!(self.writer, "{}", value)?;
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

    fn visit_while_stmt(&mut self, w: &stmt::While) -> Result<()> {
        while self.evaluate(&w.condition)?.is_truthy() {
            self.execute(&w.body)?;
        }
        Ok(())
    }
}

impl <W: Write> expr::Visitor<Result<Value>> for Interpreter<W> {
    fn visit_assign_expr(&mut self, a: &expr::Assign) -> Result<Value> {
        let value = self.evaluate(&a.value)?;
        self.environment.assign(&a.name, value.clone())?;
        Ok(value)
    }

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

    fn visit_call_expr(&mut self, e: &expr::Call) -> Result<Value> {
        let callee = self.evaluate(&e.callee)?;

        let args: Vec<Value> = e.arguments.iter()
            .map(|a| self.evaluate(a))
            .collect::<Result<_>>()?;

        callee.callable()
            .ok_or(Error::runtime(e.paren.clone(), "Can only call functions and classes."))
            .and_then(|c| {
                if args.len() == c.arity() {
                    Ok(c)
                } else {
                    Err(Error::runtime(
                        e.paren.clone(),
                        format!("Expected {} arguments but got {}", c.arity(), args.len())
                    ))
                }
            })
            .map(|c| c.call(self, args))
    }

    fn visit_grouping_expr(&mut self, e: &expr::Grouping) -> Result<Value> {
        self.evaluate(&e.expression)
    }

    fn visit_literal_expr(&mut self, e: &expr::Literal) -> Result<Value> {
        Ok(e.value.clone())
    }

    fn visit_logical_expr(&mut self, e: &expr::Logical) -> Result<Value> {
        let left = self.evaluate(&e.left)?;

        use TokenKind::*;
        Ok(match (&e.op.kind, left.is_truthy()) {
            (Or, true) | (And, false) => left,
            (Or, false) | (And, true) => self.evaluate(&e.right)?,
            _ => unreachable!("Logical expression must be either And or Or.")
        })
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