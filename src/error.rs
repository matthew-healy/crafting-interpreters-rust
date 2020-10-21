use std::result;
use std::fmt::{self, Display};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    Lexical { line: usize, message: String },
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind
}

impl Error {
    pub fn lexical<S: Into<String>>(line: usize, message: S) -> Error {
        let kind = ErrorKind::Lexical { line, message: message.into() };
        Error { kind }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        match self.kind() {
            Lexical { line, message } => write!(f, "[line {}] Error: {}", line, message),
        }
    }
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> std::io::Error {
        use std::io::ErrorKind::*;
        std::io::Error::new(Other, e)
    }
}