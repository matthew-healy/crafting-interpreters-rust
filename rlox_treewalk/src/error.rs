use std::result;
use std::fmt::{self, Display};

use crate::token::{Token, TokenKind};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    Lexical { line: usize },
    Syntactic { token: Token },
    Static { token: Token },
    Runtime { token: Token },
    Unexpected,
    Io(std::io::Error),
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

    pub fn runtime<S: Into<String>>(token: Token, message: S) -> Error {
        let kind = ErrorKind::Runtime { token };
        Error { kind, message: message.into() }
    }

    pub fn static_analyzer<S: Into<String>>(token: Token, message: S) -> Error {
        let kind = ErrorKind::Static { token };
        Error { kind, message: message.into() }
    }

    pub fn unexpected() -> Error {
        let kind = ErrorKind::Unexpected;
        Error { kind, message: "Unexpected end of input.".into() }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn is_runtime_error(&self) -> bool {
        match self.kind() {
            ErrorKind::Runtime { token: _ } => true,
            _ => false,
        }
    }

    fn loc(&self) -> String {
        use ErrorKind::*;
        match self.kind() {
            Syntactic { token } | Runtime { token } | Static { token } => {
                if token.kind == TokenKind::EndOfFile {
                    " at end".to_string()
                } else {
                    format!(" at {}", token.lexeme)
                }
            },
            _ =>  "".to_string(),
        }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        let line = match self.kind() {
            Unexpected => 0,
            Io(_e) => 0,
            Lexical { line } => *line,
            Syntactic { token } | Runtime { token } | Static { token }  => token.line,
        };
        write!(f, "[line {}] Error{}: {}", line, self.loc(), self.message)
    }
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> std::io::Error {
        use std::io::ErrorKind::*;
        std::io::Error::new(Other, e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error { kind: ErrorKind::Io(e), message: "IO error".into() }
    }
}