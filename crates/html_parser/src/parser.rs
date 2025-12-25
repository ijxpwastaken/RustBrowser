//! HTML Tree Builder (Parser)
//! 
//! Constructs a DOM tree from the token stream produced by the tokenizer.

use std::sync::{Arc, RwLock};

use dom::{Document, Element, Node, NodeRef, Text};
use crate::{ParseError, Token, Tokenizer};

/// HTML Parser - builds DOM tree from tokens
pub struct HtmlParser<'a> {
    tokenizer: Tokenizer<'a>,
    document: Document,
    open_elements: Vec<NodeRef>,
    head_pointer: Option<NodeRef>,
    body_pointer: Option<NodeRef>,
}

impl<'a> HtmlParser<'a> {
    /// Create a new parser for the given HTML string
    pub fn new(input: &'a str) -> Self {
        HtmlParser {
            tokenizer: Tokenizer::new(input),
            document: Document::new(),
            open_elements: Vec::new(),
            head_pointer: None,
            body_pointer: None,
        }
    }

    /// Parse the HTML and return the document
    pub fn parse(&mut self) -> Result<Document, ParseError> {
        let tokens = self.tokenizer.tokenize()?;
        
        println!("[Parser] Got {} tokens", tokens.len());
        for (i, t) in tokens.iter().enumerate() {
            println!("[Parser] Token {}: {:?}", i, t);
        }
        
        for token in tokens {
            self.process_token(token)?;
        }
        
        println!("[Parser] After processing, document_element is: {:?}", self.document.document_element.is_some());
        
        Ok(std::mem::take(&mut self.document))
    }

    fn process_token(&mut self, token: Token) -> Result<(), ParseError> {
        match token {
            Token::Doctype { name, public_id, system_id } => {
                self.document.set_doctype(
                    &name,
                    &public_id.unwrap_or_default(),
                    &system_id.unwrap_or_default(),
                );
            }

            Token::StartTag { name, attributes, self_closing } => {
                self.handle_start_tag(&name, attributes, self_closing)?;
            }

            Token::EndTag { name } => {
                self.handle_end_tag(&name)?;
            }

            Token::Text(text) => {
                self.handle_text(&text)?;
            }

            Token::Comment(text) => {
                self.handle_comment(&text)?;
            }

            Token::Eof => {
                // End of parsing
            }
        }
        
        Ok(())
    }

    fn handle_start_tag(
        &mut self,
        name: &str,
        attributes: std::collections::HashMap<String, String>,
        self_closing: bool,
    ) -> Result<(), ParseError> {
        let element = Element::with_attributes(name, attributes);
        let node = Node::Element(element);
        let node_ref = Arc::new(RwLock::new(node));

        // Handle special elements
        match name {
            "html" => {
                self.document.document_element = Some(Arc::clone(&node_ref));
                self.open_elements.push(node_ref);
            }
            "head" => {
                self.head_pointer = Some(Arc::clone(&node_ref));
                self.insert_element(node_ref)?;
            }
            "body" => {
                self.body_pointer = Some(Arc::clone(&node_ref));
                self.insert_element(node_ref)?;
            }
            _ => {
                self.insert_element(node_ref)?;
            }
        }

        // Handle void elements and self-closing tags
        if self_closing || is_void_element(name) {
            self.open_elements.pop();
        }

        Ok(())
    }

    fn handle_end_tag(&mut self, name: &str) -> Result<(), ParseError> {
        // Find and pop the matching element from the stack
        let mut found_index = None;
        
        for (i, elem_ref) in self.open_elements.iter().enumerate().rev() {
            if let Ok(node) = elem_ref.read() {
                if let Some(elem) = node.as_element() {
                    if elem.tag_name.eq_ignore_ascii_case(name) {
                        found_index = Some(i);
                        break;
                    }
                }
            }
        }

        if let Some(index) = found_index {
            // Close all elements up to and including the matched one
            self.open_elements.truncate(index);
        }
        // If not found, just ignore the end tag (error recovery)

        Ok(())
    }

    fn handle_text(&mut self, text: &str) -> Result<(), ParseError> {
        // Skip whitespace-only text nodes between elements (optional - for nicer trees)
        // For now, keep all text
        if text.is_empty() {
            return Ok(());
        }

        let text_node = Node::Text(Text::new(text));
        let node_ref = Arc::new(RwLock::new(text_node));

        self.insert_node(node_ref)?;

        Ok(())
    }

    fn handle_comment(&mut self, text: &str) -> Result<(), ParseError> {
        let comment_node = Node::Comment(text.to_string());
        let node_ref = Arc::new(RwLock::new(comment_node));

        self.insert_node(node_ref)?;

        Ok(())
    }

    fn insert_element(&mut self, node_ref: NodeRef) -> Result<(), ParseError> {
        self.insert_node(Arc::clone(&node_ref))?;
        self.open_elements.push(node_ref);
        Ok(())
    }

    fn insert_node(&mut self, node_ref: NodeRef) -> Result<(), ParseError> {
        // Find the current parent (last open element)
        if let Some(parent_ref) = self.open_elements.last() {
            if let Ok(mut parent) = parent_ref.write() {
                if let Some(parent_elem) = parent.as_element_mut() {
                    parent_elem.append_child_ref(node_ref);
                }
            }
        } else if self.document.document_element.is_none() {
            // No open elements and no document element - this is the root
            // Check if it's an element before moving
            let is_element = node_ref.read()
                .map(|n| n.as_element().is_some())
                .unwrap_or(false);
            
            if is_element {
                self.document.document_element = Some(node_ref);
            }
        }
        
        Ok(())
    }
}

/// Check if a tag name is a void element
fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
            | "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let html = "<html><head></head><body><p>Hello</p></body></html>";
        let mut parser = HtmlParser::new(html);
        let doc = parser.parse().unwrap();
        
        assert!(doc.document_element.is_some());
        assert!(doc.body().is_some());
    }

    #[test]
    fn test_parse_nested() {
        let html = "<html><body><div><p><span>Text</span></p></div></body></html>";
        let mut parser = HtmlParser::new(html);
        let doc = parser.parse().unwrap();
        
        doc.print_tree();
    }

    #[test]
    fn test_parse_void_elements() {
        let html = "<html><body><p>Line 1<br>Line 2</p><img src=\"test.png\"></body></html>";
        let mut parser = HtmlParser::new(html);
        let doc = parser.parse().unwrap();
        
        doc.print_tree();
    }

    #[test]
    fn test_parse_with_attributes() {
        let html = r#"<html><body><div id="main" class="container"><a href="https://example.com">Link</a></div></body></html>"#;
        let mut parser = HtmlParser::new(html);
        let doc = parser.parse().unwrap();
        
        let main = doc.get_element_by_id("main");
        assert!(main.is_some());
    }
}
