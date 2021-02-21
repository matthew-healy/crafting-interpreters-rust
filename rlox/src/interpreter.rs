use std::{
    cell::RefCell,
    collections::HashMap,
    io::Write,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH}
};

use crate::{
    environment::Environment,
    error::{Error, self},
    expr::{self, Expr},
    stmt::{self, Stmt},
    token::{TokenKind, Token},
    value::Value,
};

pub(crate) type Result<T> = std::result::Result<T, Thrown>;

pub(crate) enum Thrown {
    Error(Error),
    Return(Value),
}

impl From<error::Error> for Thrown {
    fn from(e: error::Error) -> Self {
        Self::Error(e)
    }
}

impl From<std::io::Error> for Thrown {
    fn from(e: std::io::Error) -> Self {
        Self::from(error::Error::from(e))
    }
}

pub struct Interpreter<W> {
    globals: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
    environment: Rc<RefCell<Environment>>,
    writer: W,
}

impl <W: Write> Interpreter<W> {
    pub fn new(writer: W) -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        globals.borrow_mut().define("clock", Value::new_native_fn(&|| {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time since epoch should never be negative")
                .as_millis();
            Value::Number(time as f64)
        }));

        let locals = HashMap::new();
        let environment = Rc::new(RefCell::new(Environment::from(&globals)));
        Interpreter {
            globals,
            locals,
            environment,
            writer,
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> error::Result<()> {
        for s in statements.iter() {
            match self.execute(s) {
                Err(Thrown::Return(_v)) => unreachable!("return should never make it this far up the stack."),
                Err(Thrown::Error(e)) => return Err(e),
                _ => continue
            }
        }
        Ok(())
    }

    fn execute(&mut self, s: &Stmt) -> Result<()> {
        s.accept(self)
    }

    pub(crate) fn execute_block(&mut self, statements: &[Stmt], environment: Environment) -> Result<()> {
        let old_env = Rc::clone(&self.environment);
        self.environment = Rc::new(RefCell::new(environment));
        for statement in statements {
            // Reset the environment before returning an error.
            if let Err(error) = self.execute(statement) {
                self.environment = old_env;
                return Err(error)
            }
        }
        self.environment = old_env;
        Ok(())
    }
    
    fn evaluate(&mut self, e: &Expr) -> Result<Value> {
        e.accept(self)
    }

    fn lookup_variable(&mut self, name: &Token, e: &Expr) -> Result<Value> {
        if let Some(distance) = self.locals.get(&e) {
            self.environment.borrow_mut()
                .get_at(*distance, &name)
                .map_err(|e| Thrown::Error(e))
        } else {
            self.globals.borrow().get(&name).map_err(Thrown::from)
        }
    }
}

impl <W> Interpreter<W> {
    pub(crate) fn resolve(&mut self, e: &Expr, depth: usize) {
        self.locals.insert(e.clone(), depth);
    }
}

impl <W: Write> stmt::Visitor<Result<()>> for Interpreter<W> {
    fn visit_block_stmt(&mut self, b: &stmt::Block) -> Result<()> {
        let environment = Environment::from(&self.environment);
        self.execute_block(&b.statements, environment)
    }

    fn visit_class_stmt(&mut self, c: &stmt::Class) -> Result<()> {
        let superclass = &c.superclass.as_ref()
            .map(|s| self.evaluate(s))
            .map(|s| {
                match s {
                    Ok(Value::Class(sup)) => Ok(sup),
                    Ok(_) => Err(Thrown::Error(Error::runtime(c.name.clone(), "Superclass must be a class."))),
                    Err(e) => Err(e),
                }
            }).transpose()?;

        let mut env = self.environment.borrow_mut();
        env.define(&c.name.lexeme, Value::Nil);

        let mut methods = HashMap::new();
        for method in c.methods.iter() {
            let function = Value::new_function(
                method.clone(),
                Rc::clone(&self.environment),
                method.name.lexeme == "init"
            );
            methods.insert(method.name.lexeme.clone(), function);
        }

        let class = Value::new_class(&c.name.lexeme, superclass.clone(), methods);
        env.assign(&c.name, &class)?;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, e: &stmt::Expression) -> Result<()> {
        self.evaluate(&e.expression)?;
        Ok(())
    }

    fn visit_function_stmt(&mut self, f: &stmt::Function) -> Result<()> {
        let function = Value::new_function(f.clone(), Rc::clone(&self.environment), false);
        self.environment.borrow_mut().define(&f.name.lexeme, function);
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

    fn visit_return_stmt(&mut self, r: &stmt::Return) -> Result<()> {
        let to_return = r.value.as_ref()
            .map(|v| self.evaluate(v))
            .unwrap_or(Ok(Value::Nil))?;
        Err(Thrown::Return(to_return))
    }

    fn visit_var_stmt(&mut self, v: &stmt::Var) -> Result<()> {
        let value = if let Some(initializer) = &v.initializer {
            self.evaluate(initializer)?
        } else {
            Value::Nil
        };

        let var_name = v.name.lexeme.clone();

        self.environment.borrow_mut().define(var_name, value);
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

        if let Some(distance) = self.locals.get(&Expr::Assign(a.clone())) {
            self.environment.borrow_mut().assign_at(*distance, &a.name, &value)?;
        } else {
            self.globals.borrow_mut().assign(&a.name, &value)?;
        }

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
                Err(Thrown::Error(Error::runtime(e.op.clone(), "Operands must be two numbers or two strings.")))
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
            .ok_or(Thrown::Error(Error::runtime(e.paren.clone(), "Can only call functions and classes.")))
            .and_then(|c| {
                if args.len() == c.arity() {
                    Ok(c)
                } else {
                    Err(Thrown::Error(Error::runtime(
                        e.paren.clone(),
                        format!("Expected {} arguments but got {}", c.arity(), args.len())
                    )))
                }
            })
            .and_then(|c| c.call(self, args))
    }

    fn visit_get_expr(&mut self, g: &expr::Get) -> Result<Value> {
        match self.evaluate(&g.object)? {
            Value::Instance(i) => i.get(&g.name).map_err(Thrown::Error),
            _ => Err(Thrown::Error(Error::runtime(
                g.name.clone(),
                "Only instances have properties."
            ))),
        }
    }

    fn visit_grouping_expr(&mut self, e: &expr::Grouping) -> Result<Value> {
        self.evaluate(&e.expression)
    }

    fn visit_literal_expr(&mut self, e: &expr::Literal) -> Result<Value> {
        Ok(e.value.clone().into())
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

    fn visit_set_expr(&mut self, e: &expr::Set) -> Result<Value> {
        match self.evaluate(&e.object)? {
            Value::Instance(i) => {
                let value = self.evaluate(&e.value)?;
                i.set(&e.name, &value);
                Ok(value)
            },
            _ => Err(Thrown::Error(Error::runtime(
                e.name.clone(),
                "Only instances have properties."
            )))
        }
    }

    fn visit_this_expr(&mut self, e: &expr::This) -> Result<Value> {
        self.lookup_variable(&e.keyword, &Expr::This(e.clone()))
    }

    fn visit_unary_expr(&mut self, e: &expr::Unary) -> Result<Value> {
        let right = self.evaluate(e.right.as_ref())?;
        let kind = e.op.kind.clone();

        use Value::*;
        match (kind, right) {
            (TokenKind::Minus, Number(right)) => Ok(Number(-right)),
            (TokenKind::Minus, _) => Err(Thrown::Error(Error::runtime(e.op.clone(), "Operand must be a number."))),
            (TokenKind::Bang, right) => Ok(Bool(!right.is_truthy())),
            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(&mut self, e: &expr::Variable) -> Result<Value> {
        self.lookup_variable(&e.name, &Expr::Variable(e.clone()))
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
    Err(Thrown::Error(Error::runtime(op.clone(), "Operands must be numbers.")))
}