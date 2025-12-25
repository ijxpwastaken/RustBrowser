//! Document (root node) implementation

use std::sync::{Arc, RwLock};
use crate::{Element, Node, NodeId, NodeRef};

/// The document root - container for the entire DOM tree
#[derive(Debug)]
pub struct Document {
    /// Unique identifier
    pub id: NodeId,
    /// Document type (DOCTYPE)
    pub doctype: Option<DocumentType>,
    /// The root element (usually <html>)
    pub document_element: Option<NodeRef>,
    /// Document title
    pub title: String,
    /// Base URL
    pub base_url: Option<String>,
}

/// DOCTYPE information
#[derive(Debug, Clone)]
pub struct DocumentType {
    pub name: String,
    pub public_id: String,
    pub system_id: String,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Document {
            id: NodeId::new(),
            doctype: None,
            document_element: None,
            title: String::new(),
            base_url: None,
        }
    }

    /// Set the DOCTYPE
    pub fn set_doctype(&mut self, name: &str, public_id: &str, system_id: &str) {
        self.doctype = Some(DocumentType {
            name: name.to_string(),
            public_id: public_id.to_string(),
            system_id: system_id.to_string(),
        });
    }

    /// Set the document element (root <html>)
    pub fn set_document_element(&mut self, element: Element) {
        self.document_element = Some(Arc::new(RwLock::new(Node::Element(element))));
    }

    /// Get the document element
    pub fn document_element(&self) -> Option<&NodeRef> {
        self.document_element.as_ref()
    }

    /// Get the <head> element
    pub fn head(&self) -> Option<NodeRef> {
        self.find_element("head")
    }

    /// Get the <body> element
    pub fn body(&self) -> Option<NodeRef> {
        self.find_element("body")
    }

    /// Find an element by tag name (first match)
    pub fn find_element(&self, tag_name: &str) -> Option<NodeRef> {
        let root = self.document_element.as_ref()?;
        Self::find_element_recursive(root, tag_name)
    }

    fn find_element_recursive(node_ref: &NodeRef, tag_name: &str) -> Option<NodeRef> {
        let node = node_ref.read().ok()?;
        
        if let Some(elem) = node.as_element() {
            if elem.tag_name.eq_ignore_ascii_case(tag_name) {
                return Some(Arc::clone(node_ref));
            }
            
            for child in &elem.children {
                if let Some(found) = Self::find_element_recursive(child, tag_name) {
                    return Some(found);
                }
            }
        }
        
        None
    }

    /// Get an element by ID
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeRef> {
        let root = self.document_element.as_ref()?;
        Self::find_by_id_recursive(root, id)
    }

    fn find_by_id_recursive(node_ref: &NodeRef, id: &str) -> Option<NodeRef> {
        let node = node_ref.read().ok()?;
        
        if let Some(elem) = node.as_element() {
            if elem.id_attr().map(|i| i.as_str()) == Some(id) {
                return Some(Arc::clone(node_ref));
            }
            
            for child in &elem.children {
                if let Some(found) = Self::find_by_id_recursive(child, id) {
                    return Some(found);
                }
            }
        }
        
        None
    }

    /// Print the DOM tree (for debugging)
    pub fn print_tree(&self) {
        println!("Document:");
        if let Some(doctype) = &self.doctype {
            println!("  DOCTYPE: {}", doctype.name);
        }
        if let Some(root) = &self.document_element {
            Self::print_node(root, 1);
        }
    }

    fn print_node(node_ref: &NodeRef, depth: usize) {
        let indent = "  ".repeat(depth);
        
        if let Ok(node) = node_ref.read() {
            match &*node {
                Node::Element(elem) => {
                    let attrs: Vec<String> = elem.attributes
                        .iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    
                    if attrs.is_empty() {
                        println!("{}<{}>", indent, elem.tag_name);
                    } else {
                        println!("{}<{} {}>", indent, elem.tag_name, attrs.join(" "));
                    }
                    
                    for child in &elem.children {
                        Self::print_node(child, depth + 1);
                    }
                    
                    if !elem.is_void_element() {
                        println!("{}</{}>", indent, elem.tag_name);
                    }
                }
                Node::Text(text) => {
                    let content = text.content.trim();
                    if !content.is_empty() {
                        println!("{}\"{}\"", indent, content);
                    }
                }
                Node::Comment(comment) => {
                    println!("{}<!-- {} -->", indent, comment);
                }
                _ => {}
            }
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new();
        assert!(doc.document_element.is_none());
    }

    #[test]
    fn test_doctype() {
        let mut doc = Document::new();
        doc.set_doctype("html", "", "");
        assert!(doc.doctype.is_some());
        assert_eq!(doc.doctype.as_ref().unwrap().name, "html");
    }
}
