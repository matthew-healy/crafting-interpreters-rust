use std::{fmt::{self, Debug, Display}};

use crate::stmt;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Value {
    Bool(bool),
    Function(Function),
    NativeFn(NativeFn<&'static dyn Fn() -> Value>),
    Nil,
    Number(f64),
    String(String),
}

impl Value {
    pub(crate) fn new_native_fn(body: &'static dyn Fn() -> Value) -> Self {
        Value::NativeFn(NativeFn { body })
    }

    pub(crate) fn new_function(declaration: stmt::Function) -> Self {
        Value::Function(Function { declaration })
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            Bool(b) => write!(f, "{}", b),
            Function(fnc) => write!(f, "{}", fnc),
            NativeFn(_) => write!(f, "<native fn>"),
            Nil => write!(f, "nil"),
            Number(n) => write!(f, "{}", n),
            String(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone)]
pub(crate) struct NativeFn<F> {
    pub(crate) body: F,
}

impl <F> Debug for NativeFn<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl <F> PartialEq for NativeFn<F> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Function {
    pub(crate) declaration: stmt::Function,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}