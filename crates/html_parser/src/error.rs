//! HTML Parser errors

use thiserror::Error;

/// Errors that can occur during HTML parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Unexpected character: '{0}'")]
    UnexpectedChar(char),

    #[error("Invalid tag name: '{0}'")]
    InvalidTagName(String),

    #[error("Mismatched closing tag: expected '{expected}', found '{found}'")]
    MismatchedTag { expected: String, found: String },

    #[error("Invalid attribute: {0}")]
    InvalidAttribute(String),

    #[error("Unterminated string")]
    UnterminatedString,

    #[error("Invalid DOCTYPE")]
    InvalidDoctype,

    #[error("Parse error: {0}")]
    Generic(String),
}
