use crate::{
    error::Error,
    environment::Environment,
    interpreter::{self, Interpreter, Thrown},
    value::{ClassPointer, Function, NativeFn, Value},
};
use std::io::Write;

pub(crate) trait Callable<W: Write> {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter<W>, args: Vec<Value>) -> interpreter::Result<Value>;
}

impl Value {
    pub(crate) fn callable<W: Write>(&self) -> Option<&dyn Callable<W>> {
        match self {
            Value::Class(ref c) => Some(c),
            Value::Function(ref f) => Some(f),
            Value::NativeFn(ref n) => Some(n),
            _ => None,
        }
    }
}

impl <W: Write> Callable<W> for NativeFn<&'static dyn Fn() -> Value> {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter<W>, _args: Vec<Value>) -> interpreter::Result<Value> {
        Ok((self.body)())
    }
}

impl Function {
    fn this_value(&self) -> interpreter::Result<Value> {
        self.closure.borrow()
            .maybe_get_at(0, "this")
            .ok_or(Thrown::Error(Error::unexpected()))
    }
}

impl <W: Write> Callable<W> for Function {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter<W>, args: Vec<Value>) -> interpreter::Result<Value> {
        let mut environment = Environment::from(&self.closure);

        let params_with_args = self.declaration.params.iter().zip(args);

        for (param, arg) in params_with_args {
            environment.define(&param.lexeme, arg)
        }

        match interpreter.execute_block(&self.declaration.body, environment) {
            Ok(_) if self.is_init => self.this_value(),
            Ok(()) => Ok(Value::Nil),
            Err(interpreter::Thrown::Return(_)) if self.is_init => self.this_value(),
            Err(interpreter::Thrown::Return(v)) => Ok(v),
            Err(e) => Err(e),
        }
    }
}

impl <W: Write> Callable<W> for ClassPointer {
    fn arity(&self) -> usize {
        match self.get_field("init") {
            Some(Value::Function(f)) => f.declaration.params.len(),
            _ => 0
        }

    }

    fn call(&self, interpreter: &mut Interpreter<W>, args: Vec<Value>) -> interpreter::Result<Value> {
        let instance = self.instantiate();
        if let Some(Value::Function(init)) = self.get_field("init") {
            init.binding(instance.clone()).call(interpreter, args)?;
        }
        Ok(Value::Instance(instance))
    }
}