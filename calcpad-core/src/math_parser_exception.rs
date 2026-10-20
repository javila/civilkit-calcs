//! Port of MathParserException.cs.
use std::fmt;

/// Error raised by the math engine. `message` is the user visible text.
#[derive(Clone, Debug, PartialEq)]
pub struct MathParserException {
    pub message: String,
    /// Set for failures that in C# are not MathParserException (for example
    /// IndexOutOfRangeException). ExpressionParser reports those as "Unexpected error".
    pub unexpected: bool,
}

impl MathParserException {
    pub fn new(message: impl Into<String>) -> Self {
        MathParserException {
            message: message.into(),
            unexpected: false,
        }
    }

    /// A non MathParserException failure (.NET runtime exception) with its message.
    pub fn unexpected(message: impl Into<String>) -> Self {
        MathParserException {
            message: message.into(),
            unexpected: true,
        }
    }
}

impl fmt::Display for MathParserException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MathParserException {}

pub type CResult<T> = Result<T, MathParserException>;
