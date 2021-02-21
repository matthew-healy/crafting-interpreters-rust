use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Debug, Display},
    rc::Rc
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
    Class(ClassPointer),
    Function(Function),
    Instance(InstancePointer),
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
    pub(crate) fn new_class<S: Into<String>>(name: S, superclass: Option<ClassPointer>, fields: HashMap<String, Value>) -> Self {
        Value::Class(ClassPointer::new(name.into(), superclass, fields))
    }

    pub(crate) fn new_native_fn(body: &'static dyn Fn() -> Value) -> Self {
        Value::NativeFn(NativeFn { body })
    }

    pub(crate) fn new_function(
        declaration: stmt::Function,
        closure: Rc<RefCell<Environment>>,
        is_init: bool
    ) -> Self {
        Value::Function(Function::new(declaration, closure, is_init))
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
            Class(c) => write!(f, "{}", c),
            Function(fnc) => write!(f, "{}", fnc),
            Instance(i) => write!(f, "{}", i),
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
    pub(crate) is_init: bool,
}

impl Function {
    pub(crate) fn new(declaration: stmt::Function, closure: Rc<RefCell<Environment>>, is_init: bool) -> Self {
        Self { declaration, closure, is_init }
    }

    pub(crate) fn binding(&self, i: InstancePointer) -> Function {
        let mut env = Environment::from(&self.closure);
        env.define("this", Value::Instance(i));
        Function {
            declaration: self.declaration.clone(),
            closure: Rc::new(RefCell::new(env)),
            is_init: self.is_init
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
    superclass: Option<ClassPointer>,
    fields: HashMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ClassPointer(Rc<RefCell<Class>>);

impl ClassPointer {
    fn new(name: String, superclass: Option<ClassPointer>, fields: HashMap<String, Value>) -> Self {
        let class = Class { name, superclass, fields };
        Self(Rc::new(RefCell::new(class)))
    }

    pub(crate) fn get_field(&self, name: &str) -> Option<Value> {
        let class = self.0.borrow();
        class.fields.get(name).map(|m| m.clone())
    }

    pub(crate) fn instantiate(&self) -> InstancePointer {
        InstancePointer::new(Instance {
            class: self.clone(),
            fields: HashMap::new()
        })
    }
}

impl Display for ClassPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.0.borrow().name)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Instance {
    class: ClassPointer,
    fields: HashMap<String, Value>,
}

impl Instance {
    fn get_field(&self, name: &str) -> Option<Value> {
        self.fields.get(name).map(|f| f.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InstancePointer(Rc<RefCell<Instance>>);

impl InstancePointer {
    pub(crate) fn new(instance: Instance) -> Self {
        Self(Rc::new(RefCell::new(instance)))
    }

    pub(crate) fn get(&self, name: &Token) -> Result<Value> {
        let instance = self.0.borrow();
        instance.get_field(&name.lexeme)
            .or_else(|| {
                let field = instance.class.get_field(&name.lexeme);
                if let Some(Value::Function(method)) = field {
                    Some(Value::Function(method.binding(self.clone())))
                } else { field }
            })
            .ok_or_else(||
                Error::runtime(
                    name.clone(),
                    format!("Undefined property {}.", &name.lexeme)
                )
            )
    }

    pub(crate) fn set(&self, name: &Token, value: &Value) {
        let mut instance = self.0.borrow_mut();
        instance.fields.insert(name.lexeme.clone(), value.clone());
    }
}

impl Display for InstancePointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.0.borrow().class)
    }
}