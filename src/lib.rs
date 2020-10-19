mod error;

pub use crate::error::{Error, Result};

pub struct Scanner {

}

impl Scanner {
    pub fn new(_source: String) -> Self {
        Self {}
    }

    pub fn scan_tokens(&self) -> Result<Vec<Token>> {
        Ok(Vec::new())
    }
}

#[derive(Debug)]
pub enum Token {

}