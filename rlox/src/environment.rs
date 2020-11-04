use std::collections::{HashMap, VecDeque};
use crate::{
    error::{Error, Result},
    token::Token,
    value::Value,
};

pub(crate) struct Environment {
    stack: VecDeque<ScopedEnv>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        let mut stack = VecDeque::new();
        stack.push_front(ScopedEnv::new());
        Self { stack }
    }

    pub(crate) fn push_child_env(&mut self) {
        self.stack.push_front(ScopedEnv::new())
    }

    pub(crate) fn pop_child_env(&mut self) {
        if self.stack.len() == 1 { return }
        self.stack.pop_front();
    }

    pub(crate) fn get(&self, name: &Token) -> Result<Value> {
        self.stack.iter()
            .find_map(|e| e.get(name))
            .ok_or_else(|| {
                Error::runtime(
                    name.clone(), 
                    format!("Undefined variable '{}'.", name.lexeme)
                )
            })
    }

    pub(crate) fn assign(&mut self, name: &Token, value: Value) -> Result<()> {
        self.stack.iter_mut()
            .find_map(|e| e.get_mut(name))
            .map(|v| *v = value)
            .ok_or_else(|| {
                Error::runtime(
                    name.clone(),
                    format!("Undefined variable '{}'", name.lexeme)
                )
            })
    }

    pub(crate) fn define<S: Into<String>>(&mut self, name: S, value: Value) {
        self.stack[0].define(name.into(), value)
    }
}

struct ScopedEnv {
    values: HashMap<String, Value>,
}

impl ScopedEnv {
    fn new() -> Self {
        Self { values: HashMap::new() }
    }

    fn get(&self, name: &Token) -> Option<Value> {
        self.values
            .get(name.lexeme.as_str())
            .map(|v| v.clone())
    }

    fn get_mut(&mut self, name: &Token) -> Option<&mut Value> {
        self.values.get_mut(name.lexeme.as_str())
    }

    fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}