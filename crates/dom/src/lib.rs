//! DOM (Document Object Model) implementation
//! 
//! This crate provides the core data structures for representing HTML documents
//! as a tree of nodes.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub mod node;
pub mod element;
pub mod document;
pub mod text;

pub use node::{Node, NodeId, NodeType, WeakNode};
pub use element::Element;
pub use document::Document;
pub use text::Text;

/// Attributes map type alias
pub type Attributes = HashMap<String, String>;

/// A thread-safe reference to a DOM node
pub type NodeRef = Arc<RwLock<Node>>;

/// Create a new element node
pub fn create_element(tag_name: &str) -> Node {
    Node::Element(Element::new(tag_name))
}

/// Create a new text node
pub fn create_text(content: &str) -> Node {
    Node::Text(Text::new(content))
}

/// Create a new document
pub fn create_document() -> Document {
    Document::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_element() {
        let node = create_element("div");
        assert!(matches!(node, Node::Element(_)));
    }

    #[test]
    fn test_create_text() {
        let node = create_text("Hello, World!");
        assert!(matches!(node, Node::Text(_)));
    }
}
