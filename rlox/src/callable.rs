use crate::{
    environment::Environment,
    interpreter::{self, Interpreter},
    value::{Function, NativeFn, Value},
};
use std::io::Write;

pub(crate) trait Callable<W: Write> {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter<W>, args: Vec<Value>) -> interpreter::Result<Value>;
}

impl Value {
    pub(crate) fn callable<W: Write>(&self) -> Option<&dyn Callable<W>> {
        match self {
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

impl <W: Write> Callable<W> for Function {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter<W>, args: Vec<Value>) -> interpreter::Result<Value> {
        let mut environment = Environment::from(&self.environment);

        let params_with_args = self.declaration.params.iter().zip(args);

        for (param, arg) in params_with_args {
            environment.define(&param.lexeme, arg)
        }

        match interpreter.execute_block(&self.declaration.body, environment) {
            Ok(()) => Ok(Value::Nil),
            Err(interpreter::Thrown::Return(v)) => Ok(v),
            Err(e) => Err(e),
        }
    }
}