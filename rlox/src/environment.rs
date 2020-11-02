use std::collections::HashMap;
use crate::{
    error::{Error, Result},
    token::Token,
    value::Value,
};

pub(crate) struct Environment {
    values: HashMap<String, Value>
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub(crate) fn get(&self, name: &Token) -> Result<Value> {
        self.values.get(&name.lexeme)
            .map(|v| v.clone())
            .ok_or_else(|| {
                Error::runtime(
                    name.clone(), 
                    format!("Undefined variable '{}'.", name.lexeme)
                )
            })
    }

    pub(crate) fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}