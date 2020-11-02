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
        self.values.get(name.lexeme.as_str())
            .map(|v| v.clone())
            .ok_or_else(|| {
                Error::runtime(
                    name.clone(), 
                    format!("Undefined variable '{}'.", name.lexeme)
                )
            })
    }

    pub(crate) fn assign(&mut self, name: &Token, value: Value) -> Result<()> {
        if let Some(prev_value) = self.values.get_mut(name.lexeme.as_str()) {
            Ok(*prev_value = value)
        } else {
            Err(Error::runtime(
                name.clone(),
                format!("Undefined variable '{}'", name.lexeme)
            ))
        }
    }

    pub(crate) fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}