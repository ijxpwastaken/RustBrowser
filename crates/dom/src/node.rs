//! Node types and base node structure

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak, RwLock};

use crate::{Element, Text};

/// Global node ID counter for unique identification
static NODE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for each node in the DOM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    /// Generate a new unique node ID
    pub fn new() -> Self {
        NodeId(NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Get the raw ID value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of nodes in the DOM tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Element,
    Text,
    Comment,
    Document,
    DocumentType,
}

/// A weak reference to a node (used for parent references to avoid cycles)
pub type WeakNode = Weak<RwLock<Node>>;

/// A DOM Node - the fundamental building block of the document tree
#[derive(Debug, Clone)]
pub enum Node {
    /// An element node (e.g., <div>, <p>)
    Element(Element),
    /// A text node containing character data
    Text(Text),
    /// A comment node
    Comment(String),
    /// The document root
    Document,
    /// DOCTYPE declaration
    DocumentType {
        name: String,
        public_id: String,
        system_id: String,
    },
}

impl Node {
    /// Get the type of this node
    pub fn node_type(&self) -> NodeType {
        match self {
            Node::Element(_) => NodeType::Element,
            Node::Text(_) => NodeType::Text,
            Node::Comment(_) => NodeType::Comment,
            Node::Document => NodeType::Document,
            Node::DocumentType { .. } => NodeType::DocumentType,
        }
    }

    /// Get this node as an element, if it is one
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            Node::Element(elem) => Some(elem),
            _ => None,
        }
    }

    /// Get this node as a mutable element, if it is one
    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        match self {
            Node::Element(elem) => Some(elem),
            _ => None,
        }
    }

    /// Get this node as a text node, if it is one
    pub fn as_text(&self) -> Option<&Text> {
        match self {
            Node::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Get the text content of this node and its descendants
    pub fn text_content(&self) -> String {
        match self {
            Node::Text(text) => text.content.clone(),
            Node::Comment(text) => text.clone(),
            Node::Element(elem) => {
                elem.children
                    .iter()
                    .map(|child| child.read().unwrap().text_content())
                    .collect()
            }
            _ => String::new(),
        }
    }

    /// Check if this is an element with the given tag name
    pub fn is_element(&self, tag_name: &str) -> bool {
        match self {
            Node::Element(elem) => elem.tag_name.eq_ignore_ascii_case(tag_name),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_unique() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_node_type() {
        let elem = Node::Element(Element::new("div"));
        assert_eq!(elem.node_type(), NodeType::Element);

        let text = Node::Text(Text::new("hello"));
        assert_eq!(text.node_type(), NodeType::Text);
    }
}
