use crate::{
    interpreter::Interpreter,
    value::{NativeFn, Value},
};
use std::{io::Write};

pub(crate) trait Callable<W: Write> {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter<W>, args: Vec<Value>) -> Value;
}

impl Value {
    pub(crate) fn callable<W: Write>(&self) -> Option<&dyn Callable<W>> {
        match self {
            Value::NativeFn(ref n) => Some(n),
            _ => None,
        }
    }
}

impl <W: Write> Callable<W> for NativeFn<&'static dyn Fn() -> Value> {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter<W>, _args: Vec<Value>) -> Value {
        (self.body)()
    }
}