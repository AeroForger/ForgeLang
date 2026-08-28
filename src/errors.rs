use std::fmt;

#[derive(Debug)]
pub struct ForgeError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl ForgeError {
    pub fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self { message: message.into(), line, col }
    }
    pub fn parse(message: impl Into<String>) -> Self {
        Self { message: message.into(), line: 0, col: 0 }
    }
    pub fn codegen(message: impl Into<String>) -> Self {
        Self { message: message.into(), line: 0, col: 0 }
    }
}

impl fmt::Display for ForgeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.line > 0 {
            write!(f, "error at {}:{}: {}", self.line, self.col, self.message)
        } else {
            write!(f, "error: {}", self.message)
        }
    }
}

impl std::error::Error for ForgeError {}

impl From<String> for ForgeError {
    fn from(message: String) -> Self {
        ForgeError { message, line: 0, col: 0 }
    }
}

pub type ForgeResult<T> = Result<T, ForgeError>;