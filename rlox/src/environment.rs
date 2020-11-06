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
                    .unwrap_or_else(|| Err(Error::runtime(
                        name.clone(),
                        format!("Undefined variable '{}'.", name.lexeme)
                    )))
            })
    }

    pub(crate) fn assign(&mut self, name: &Token, value: &Value) -> Result<()> {
        self.values.get_mut(&name.lexeme)
            .map(|v| Ok(*v = value.clone()))
            .unwrap_or_else(|| {
                self.enclosing.as_ref()
                    .map(|e| e.borrow_mut().assign(name, value))
                    .unwrap_or_else(|| Err(Error::runtime(
                        name.clone(),
                        format!("Undefined variable '{}'", name.lexeme)))
                    )
            })
    }

    pub(crate) fn define<S: Into<String>>(&mut self, name: S, value: Value) {
        self.values.insert(name.into(), value);
    }
}