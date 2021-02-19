use std::{
    rc::Rc,
    cell::RefCell,
    fmt::{self, Debug, Display},
    collections::HashMap,
};

use crate::{
    environment::Environment,
    error::{Error, Result},
    stmt,
    token::{Token, HashableNumber},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Literal {
    Bool(bool),
    Nil,
    Number(HashableNumber),
    String(String),
}

impl From<bool> for Literal {
    fn from(b: bool) -> Self {
        Literal::Bool(b)
    }
}

impl From<HashableNumber> for Literal {
    fn from(n: HashableNumber) -> Self {
        Literal::Number(n)
    }
}

impl From<String> for Literal {
    fn from(s: String) -> Self {
        Literal::String(s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Bool(bool),
    Class(Class),
    Function(Function),
    Instance(Rc<RefCell<Instance>>),
    NativeFn(NativeFn<&'static dyn Fn() -> Value>),
    Nil,
    Number(f64),
    String(String),
}

impl From<Literal> for Value {
    fn from(l: Literal) -> Self {
        match l {
            Literal::Bool(b) => Self::Bool(b),
            Literal::Nil => Self::Nil,
            Literal::Number(n) => Self::Number(n.0),
            Literal::String(s) => Self::String(s),
        }
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl Value {
    pub(crate) fn new_class<S: Into<String>>(name: S, fields: HashMap<String, Value>) -> Self {
        Value::Class(Class { name: name.into(), fields })
    }

    pub(crate) fn new_instance(class: Class) -> Self {
        Value::Instance(Rc::new(RefCell::new(Instance { class, fields: HashMap::new() })))
    }

    pub(crate) fn new_native_fn(body: &'static dyn Fn() -> Value) -> Self {
        Value::NativeFn(NativeFn { body })
    }

    pub(crate) fn new_function(declaration: stmt::Function, closure: Rc<RefCell<Environment>>) -> Self {
        Value::Function(Function::new(declaration, closure))
    }

    pub(crate) fn is_equal(&self, other: &Value) -> bool {
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

    pub(crate) fn is_truthy(&self) -> bool {
        use Value::*;
        match self {
            Bool(false) | Nil => false,
            _ => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            Bool(b) => write!(f, "{}", b),
            Class(c) => write!(f, "{}", &c.name),
            Function(fnc) => write!(f, "{}", fnc),
            Instance(i) => write!(f, "{} instance", &i.borrow().class.name),
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
    pub(crate) closure: Rc<RefCell<Environment>>,
}

impl Function {
    pub(crate) fn new(declaration: stmt::Function, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            declaration,
            closure,
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Class {
    pub(crate) name: String,
    fields: HashMap<String, Value>,
}

impl Class {
    fn get_field(&self, name: &str) -> Option<Value> {
        self.fields.get(name).map(|m| m.clone())
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Instance {
    class: Class,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub(crate) fn get(&self, name: &Token) -> Result<Value> {
        self.get_field(&name)
            .or_else(|| self.class.get_field(&name.lexeme))
            .ok_or_else(||
                Error::runtime(
                    name.clone(),
                    format!("Undefined property {}.", &name.lexeme)
                )
            )
    }

    fn get_field(&self, name: &Token) -> Option<Value> {
        self.fields.get(&name.lexeme).map(|f| f.clone())
    }

    pub(crate) fn set(&mut self, name: &Token, value: &Value) {
        self.fields.insert(name.lexeme.clone(), value.clone());
    }
}