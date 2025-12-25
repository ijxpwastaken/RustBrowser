//! HTML5 Parser
//! 
//! This crate provides an HTML5 tokenizer and tree-builder for parsing
//! HTML documents into a DOM tree.

pub mod tokenizer;
pub mod parser;
pub mod error;

pub use tokenizer::{Tokenizer, Token};
pub use parser::HtmlParser;
pub use error::ParseError;

use dom::Document;

/// Parse an HTML string into a Document
pub fn parse(html: &str) -> Result<Document, ParseError> {
    let mut parser = HtmlParser::new(html);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let html = "<html><body><p>Hello</p></body></html>";
        let doc = parse(html).unwrap();
        assert!(doc.body().is_some());
    }

    #[test]
    fn test_parse_with_doctype() {
        let html = "<!DOCTYPE html><html><head><title>Test</title></head><body></body></html>";
        let doc = parse(html).unwrap();
        assert!(doc.doctype.is_some());
    }
}
