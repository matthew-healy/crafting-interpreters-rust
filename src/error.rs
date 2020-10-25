use std::result;
use std::fmt::{self, Display};

use crate::token::{Token, TokenKind};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    Lexical { line: usize },
    Syntactic { token: Token },
    Unexpected,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn lexical<S: Into<String>>(line: usize, message: S) -> Error {
        let kind = ErrorKind::Lexical { line };
        Error { kind, message: message.into() }
    }

    pub fn syntactic<S: Into<String>>(token: Token, message: S) -> Error {
        let kind = ErrorKind::Syntactic { token };
        Error { kind, message: message.into() }
    }

    pub fn unexpected() -> Error {
        let kind = ErrorKind::Unexpected;
        Error { kind, message: "Unexpected end of input.".into() }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        let (line, loc) = match self.kind() {
            Unexpected => (0, None),
            Lexical { line } => (*line, None),
            Syntactic { token } => {
                let loc = if token.kind == TokenKind::EndOfFile {
                    " at end".into()
                } else {
                    format!(" at {}", token.lexeme)
                };
                (token.line, Some(loc))
            },
        };

        let loc = loc.unwrap_or_else(|| "".to_string() );
        write!(f, "[line {}] Error{}: {}", line, loc, self.message)
    }
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> std::io::Error {
        use std::io::ErrorKind::*;
        std::io::Error::new(Other, e)
    }
}