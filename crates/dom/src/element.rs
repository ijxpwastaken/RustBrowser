//! Element node implementation

use std::sync::{Arc, RwLock};
use crate::{Attributes, Node, NodeId, NodeRef};

/// An HTML element node
#[derive(Debug, Clone)]
pub struct Element {
    /// Unique identifier for this element
    pub id: NodeId,
    /// Tag name (e.g., "div", "p", "span")
    pub tag_name: String,
    /// Element attributes
    pub attributes: Attributes,
    /// Child nodes
    pub children: Vec<NodeRef>,
    /// Namespace (for XML/SVG support)
    pub namespace: Option<String>,
}

impl Element {
    /// Create a new element with the given tag name
    pub fn new(tag_name: &str) -> Self {
        Element {
            id: NodeId::new(),
            tag_name: tag_name.to_lowercase(),
            attributes: Attributes::new(),
            children: Vec::new(),
            namespace: None,
        }
    }

    /// Create a new element with attributes
    pub fn with_attributes(tag_name: &str, attributes: Attributes) -> Self {
        Element {
            id: NodeId::new(),
            tag_name: tag_name.to_lowercase(),
            attributes,
            children: Vec::new(),
            namespace: None,
        }
    }

    /// Get an attribute value
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// Set an attribute value
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    /// Remove an attribute
    pub fn remove_attribute(&mut self, name: &str) -> Option<String> {
        self.attributes.remove(name)
    }

    /// Check if the element has an attribute
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Get the element's ID attribute
    pub fn id_attr(&self) -> Option<&String> {
        self.get_attribute("id")
    }

    /// Get the element's class attribute as a list
    pub fn classes(&self) -> Vec<&str> {
        self.get_attribute("class")
            .map(|c| c.split_whitespace().collect())
            .unwrap_or_default()
    }

    /// Check if the element has a specific class
    pub fn has_class(&self, class_name: &str) -> bool {
        self.classes().contains(&class_name)
    }

    /// Append a child node
    pub fn append_child(&mut self, child: Node) {
        self.children.push(Arc::new(RwLock::new(child)));
    }

    /// Append a child node reference
    pub fn append_child_ref(&mut self, child: NodeRef) {
        self.children.push(child);
    }

    /// Remove a child at the given index
    pub fn remove_child(&mut self, index: usize) -> Option<NodeRef> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    /// Get the number of children
    pub fn children_count(&self) -> usize {
        self.children.len()
    }

    /// Check if this is a void element (self-closing, no children allowed)
    pub fn is_void_element(&self) -> bool {
        matches!(
            self.tag_name.as_str(),
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" 
            | "link" | "meta" | "param" | "source" | "track" | "wbr"
        )
    }

    /// Check if this is a block-level element
    pub fn is_block(&self) -> bool {
        matches!(
            self.tag_name.as_str(),
            "address" | "article" | "aside" | "blockquote" | "canvas" | "dd"
            | "div" | "dl" | "dt" | "fieldset" | "figcaption" | "figure"
            | "footer" | "form" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
            | "header" | "hr" | "li" | "main" | "nav" | "noscript" | "ol"
            | "p" | "pre" | "section" | "table" | "tfoot" | "ul" | "video"
        )
    }

    /// Check if this is an inline element
    pub fn is_inline(&self) -> bool {
        !self.is_block()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let elem = Element::new("DIV");
        assert_eq!(elem.tag_name, "div"); // Lowercased
    }

    #[test]
    fn test_attributes() {
        let mut elem = Element::new("div");
        elem.set_attribute("id", "main");
        elem.set_attribute("class", "container fluid");

        assert_eq!(elem.id_attr(), Some(&"main".to_string()));
        assert!(elem.has_class("container"));
        assert!(elem.has_class("fluid"));
        assert!(!elem.has_class("other"));
    }

    #[test]
    fn test_void_elements() {
        assert!(Element::new("br").is_void_element());
        assert!(Element::new("img").is_void_element());
        assert!(!Element::new("div").is_void_element());
    }

    #[test]
    fn test_block_inline() {
        assert!(Element::new("div").is_block());
        assert!(Element::new("p").is_block());
        assert!(Element::new("span").is_inline());
        assert!(Element::new("a").is_inline());
    }
}
