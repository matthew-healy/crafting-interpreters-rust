use std::{
    collections::{HashMap},
    rc::Rc,
    cell::RefCell
};
use crate::{
    error::{Error, Result},
    token::Token,
    value::Value,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self { enclosing: None, values: HashMap::new() }
    }

    pub(crate) fn from(e: &Rc<RefCell<Environment>>) -> Self {
        Self { enclosing: Some(Rc::clone(e)), values: HashMap::new() }
    }

    pub(crate) fn get(&self, name: &Token) -> Result<Value> {
        self.values.get(&name.lexeme)
            .map(|v| Ok(v.clone()))
            .unwrap_or_else(|| {
                self.enclosing.as_ref()
                    .map(|e| e.borrow().get(name))
                    .unwrap_or_else(|| Err(undefined_var_error(&name)))
            })
    }

    pub(crate) fn get_at(&mut self, distance: usize, name: &Token) -> Result<Value> {
        self.with_ancestor_at(distance, |e| {
            e.values.get(&name.lexeme).cloned()
        }).ok_or_else(|| undefined_var_error(&name))
    }

    pub(crate) fn assign_at(&mut self, distance: usize, name: &Token, value: &Value) -> Result<()> {
        self.with_ancestor_at(distance, |e| {
            e.values.get_mut(&name.lexeme).map(|v| *v = value.clone())
        }).ok_or_else(|| undefined_var_error(&name))
    }

    fn with_ancestor_at<T, F: Fn(&mut Environment) -> T>(&mut self, distance: usize, f: F) -> T {
        match distance {
            0 => f(self),
            dist if self.enclosing.is_some() => {
                self.enclosing.as_ref()
                    .unwrap()
                    .borrow_mut()
                    .with_ancestor_at(dist - 1, f)
            },
            _ => unreachable!("Impossible scope. This is a static analysis bug.")
        }
    }

    pub(crate) fn assign(&mut self, name: &Token, value: &Value) -> Result<()> {
        self.values.get_mut(&name.lexeme)
            .map(|v| Ok(*v = value.clone()))
            .unwrap_or_else(|| {
                self.enclosing.as_ref()
                    .map(|e| e.borrow_mut().assign(name, value))
                    .unwrap_or_else(|| Err(undefined_var_error(&name)))
            })
    }

    pub(crate) fn define<S: Into<String>>(&mut self, name: S, value: Value) {
        self.values.insert(name.into(), value);
    }
}

fn undefined_var_error(name: &Token) -> Error {
    Error::runtime(
        name.clone(),
        format!("Undefined variable: {}", name.lexeme)
    )
}