//! CSS Parser
//!
//! This crate provides CSS tokenization and parsing with full selector support,
//! specificity calculation, and comprehensive value handling.

pub mod tokenizer;
pub mod parser;

pub use tokenizer::{CssToken, CssTokenizer};
pub use parser::{
    Stylesheet, Rule, Selector, SelectorPart, Specificity,
    Declaration, Value, Unit, Color, Combinator, AttributeOperator,
    parse_css, CssParser,
};
