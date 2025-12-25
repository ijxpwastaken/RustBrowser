//! Text node implementation

use crate::NodeId;

/// A text node containing character data
#[derive(Debug, Clone)]
pub struct Text {
    /// Unique identifier
    pub id: NodeId,
    /// Text content
    pub content: String,
}

impl Text {
    /// Create a new text node
    pub fn new(content: &str) -> Self {
        Text {
            id: NodeId::new(),
            content: content.to_string(),
        }
    }

    /// Get the length of the text content
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if the text is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Check if the text is only whitespace
    pub fn is_whitespace_only(&self) -> bool {
        self.content.chars().all(|c| c.is_whitespace())
    }

    /// Append more text
    pub fn append(&mut self, text: &str) {
        self.content.push_str(text);
    }

    /// Split the text at the given index
    pub fn split_at(&mut self, index: usize) -> Text {
        let second_half = self.content.split_off(index);
        Text::new(&second_half)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_creation() {
        let text = Text::new("Hello, World!");
        assert_eq!(text.content, "Hello, World!");
        assert!(!text.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let text = Text::new("   \t\n  ");
        assert!(text.is_whitespace_only());

        let text2 = Text::new("  hello  ");
        assert!(!text2.is_whitespace_only());
    }

    #[test]
    fn test_split() {
        let mut text = Text::new("Hello, World!");
        let second = text.split_at(7);
        assert_eq!(text.content, "Hello, ");
        assert_eq!(second.content, "World!");
    }
}
