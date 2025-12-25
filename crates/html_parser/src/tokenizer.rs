//! HTML5 Tokenizer
//! 
//! Converts an HTML string into a stream of tokens following the HTML5 specification.

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

use crate::ParseError;

/// Token types produced by the tokenizer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// DOCTYPE declaration
    Doctype {
        name: String,
        public_id: Option<String>,
        system_id: Option<String>,
    },
    /// Start tag: <name attr="value">
    StartTag {
        name: String,
        attributes: HashMap<String, String>,
        self_closing: bool,
    },
    /// End tag: </name>
    EndTag {
        name: String,
    },
    /// Character data
    Text(String),
    /// Comment: <!-- ... -->
    Comment(String),
    /// End of file
    Eof,
}

/// Tokenizer state machine
#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValue,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    Comment,
    CommentEnd,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
}

/// HTML5 Tokenizer
pub struct Tokenizer<'a> {
    input: Peekable<Chars<'a>>,
    state: State,
    current_token: Option<Token>,
    current_tag_name: String,
    current_attr_name: String,
    current_attr_value: String,
    current_attributes: HashMap<String, String>,
    is_self_closing: bool,
    is_end_tag: bool,
    buffer: String,
    tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer for the given HTML string
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            input: input.chars().peekable(),
            state: State::Data,
            current_token: None,
            current_tag_name: String::new(),
            current_attr_name: String::new(),
            current_attr_value: String::new(),
            current_attributes: HashMap::new(),
            is_self_closing: false,
            is_end_tag: false,
            buffer: String::new(),
            tokens: Vec::new(),
        }
    }

    /// Tokenize the entire input and return all tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                self.tokens.push(token);
                break;
            }
            self.tokens.push(token);
        }
        Ok(std::mem::take(&mut self.tokens))
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        loop {
            match self.state {
                State::Data => {
                    match self.input.next() {
                        Some('<') => {
                            // Emit any accumulated text
                            if !self.buffer.is_empty() {
                                let text = std::mem::take(&mut self.buffer);
                                // IMPORTANT: Set state BEFORE returning so next call starts in TagOpen
                                self.state = State::TagOpen;
                                return Ok(Token::Text(text));
                            }
                            self.state = State::TagOpen;
                        }
                        Some(c) => {
                            self.buffer.push(c);
                        }
                        None => {
                            if !self.buffer.is_empty() {
                                let text = std::mem::take(&mut self.buffer);
                                return Ok(Token::Text(text));
                            }
                            return Ok(Token::Eof);
                        }
                    }
                }

                State::TagOpen => {
                    match self.input.peek() {
                        Some('/') => {
                            self.input.next();
                            self.state = State::EndTagOpen;
                        }
                        Some('!') => {
                            self.input.next();
                            self.state = State::MarkupDeclarationOpen;
                        }
                        Some('?') => {
                            // Processing instruction - treat as bogus comment
                            self.input.next();
                            self.state = State::BogusComment;
                        }
                        Some(c) if c.is_ascii_alphabetic() => {
                            self.current_tag_name.clear();
                            self.current_attributes.clear();
                            self.is_self_closing = false;
                            self.is_end_tag = false;
                            self.state = State::TagName;
                        }
                        _ => {
                            // Parse error - emit '<' as text
                            self.buffer.push('<');
                            self.state = State::Data;
                        }
                    }
                }

                State::EndTagOpen => {
                    match self.input.peek() {
                        Some(c) if c.is_ascii_alphabetic() => {
                            self.current_tag_name.clear();
                            self.is_end_tag = true;
                            self.state = State::TagName;
                        }
                        Some('>') => {
                            self.input.next();
                            self.state = State::Data;
                        }
                        _ => {
                            self.state = State::BogusComment;
                        }
                    }
                }

                State::TagName => {
                    match self.input.next() {
                        Some(c) if c.is_whitespace() => {
                            self.state = State::BeforeAttributeName;
                        }
                        Some('/') => {
                            self.state = State::SelfClosingStartTag;
                        }
                        Some('>') => {
                            self.state = State::Data;
                            return Ok(self.emit_tag());
                        }
                        Some(c) => {
                            self.current_tag_name.push(c.to_ascii_lowercase());
                        }
                        None => {
                            return Err(ParseError::UnexpectedEof);
                        }
                    }
                }

                State::BeforeAttributeName => {
                    match self.input.peek() {
                        Some(c) if c.is_whitespace() => {
                            self.input.next();
                        }
                        Some('/') => {
                            self.input.next();
                            self.state = State::SelfClosingStartTag;
                        }
                        Some('>') => {
                            self.input.next();
                            self.state = State::Data;
                            return Ok(self.emit_tag());
                        }
                        Some(_) => {
                            self.current_attr_name.clear();
                            self.current_attr_value.clear();
                            self.state = State::AttributeName;
                        }
                        None => {
                            return Err(ParseError::UnexpectedEof);
                        }
                    }
                }

                State::AttributeName => {
                    match self.input.peek() {
                        Some(c) if c.is_whitespace() => {
                            self.input.next();
                            self.state = State::AfterAttributeName;
                        }
                        Some('/') => {
                            self.save_attribute();
                            self.input.next();
                            self.state = State::SelfClosingStartTag;
                        }
                        Some('=') => {
                            self.input.next();
                            self.state = State::BeforeAttributeValue;
                        }
                        Some('>') => {
                            self.save_attribute();
                            self.input.next();
                            self.state = State::Data;
                            return Ok(self.emit_tag());
                        }
                        Some(&c) => {
                            self.input.next();
                            self.current_attr_name.push(c.to_ascii_lowercase());
                        }
                        None => {
                            return Err(ParseError::UnexpectedEof);
                        }
                    }
                }

                State::AfterAttributeName => {
                    match self.input.peek() {
                        Some(c) if c.is_whitespace() => {
                            self.input.next();
                        }
                        Some('/') => {
                            self.save_attribute();
                            self.input.next();
                            self.state = State::SelfClosingStartTag;
                        }
                        Some('=') => {
                            self.input.next();
                            self.state = State::BeforeAttributeValue;
                        }
                        Some('>') => {
                            self.save_attribute();
                            self.input.next();
                            self.state = State::Data;
                            return Ok(self.emit_tag());
                        }
                        Some(_) => {
                            self.save_attribute();
                            self.current_attr_name.clear();
                            self.current_attr_value.clear();
                            self.state = State::AttributeName;
                        }
                        None => {
                            return Err(ParseError::UnexpectedEof);
                        }
                    }
                }

                State::BeforeAttributeValue => {
                    match self.input.peek() {
                        Some(c) if c.is_whitespace() => {
                            self.input.next();
                        }
                        Some('"') => {
                            self.input.next();
                            self.state = State::AttributeValueDoubleQuoted;
                        }
                        Some('\'') => {
                            self.input.next();
                            self.state = State::AttributeValueSingleQuoted;
                        }
                        Some('>') => {
                            self.save_attribute();
                            self.input.next();
                            self.state = State::Data;
                            return Ok(self.emit_tag());
                        }
                        Some(_) => {
                            self.state = State::AttributeValueUnquoted;
                        }
                        None => {
                            return Err(ParseError::UnexpectedEof);
                        }
                    }
                }

                State::AttributeValueDoubleQuoted => {
                    match self.input.next() {
                        Some('"') => {
                            self.save_attribute();
                            self.state = State::AfterAttributeValue;
                        }
                        Some(c) => {
                            self.current_attr_value.push(c);
                        }
                        None => {
                            return Err(ParseError::UnterminatedString);
                        }
                    }
                }

                State::AttributeValueSingleQuoted => {
                    match self.input.next() {
                        Some('\'') => {
                            self.save_attribute();
                            self.state = State::AfterAttributeValue;
                        }
                        Some(c) => {
                            self.current_attr_value.push(c);
                        }
                        None => {
                            return Err(ParseError::UnterminatedString);
                        }
                    }
                }

                State::AttributeValueUnquoted => {
                    match self.input.peek() {
                        Some(c) if c.is_whitespace() => {
                            self.save_attribute();
                            self.input.next();
                            self.state = State::BeforeAttributeName;
                        }
                        Some('>') => {
                            self.save_attribute();
                            self.input.next();
                            self.state = State::Data;
                            return Ok(self.emit_tag());
                        }
                        Some(&c) => {
                            self.input.next();
                            self.current_attr_value.push(c);
                        }
                        None => {
                            return Err(ParseError::UnexpectedEof);
                        }
                    }
                }

                State::AfterAttributeValue => {
                    match self.input.peek() {
                        Some(c) if c.is_whitespace() => {
                            self.input.next();
                            self.state = State::BeforeAttributeName;
                        }
                        Some('/') => {
                            self.input.next();
                            self.state = State::SelfClosingStartTag;
                        }
                        Some('>') => {
                            self.input.next();
                            self.state = State::Data;
                            return Ok(self.emit_tag());
                        }
                        _ => {
                            self.state = State::BeforeAttributeName;
                        }
                    }
                }

                State::SelfClosingStartTag => {
                    match self.input.peek() {
                        Some('>') => {
                            self.input.next();
                            self.is_self_closing = true;
                            self.state = State::Data;
                            return Ok(self.emit_tag());
                        }
                        _ => {
                            self.state = State::BeforeAttributeName;
                        }
                    }
                }

                State::MarkupDeclarationOpen => {
                    // Check for DOCTYPE or comment
                    let mut peek_chars = String::new();
                    for _ in 0..7 {
                        if let Some(&c) = self.input.peek() {
                            peek_chars.push(c);
                            self.input.next();
                        }
                    }

                    if peek_chars.starts_with("--") {
                        self.buffer.clear();
                        // Skip the "--" we already consumed
                        self.state = State::Comment;
                    } else if peek_chars.to_uppercase().starts_with("DOCTYPE") {
                        self.state = State::BeforeDoctypeName;
                    } else {
                        // Bogus comment
                        self.buffer = peek_chars;
                        self.state = State::BogusComment;
                    }
                }

                State::Comment => {
                    match self.input.next() {
                        Some('-') => {
                            if self.input.peek() == Some(&'-') {
                                self.input.next();
                                self.state = State::CommentEnd;
                            } else {
                                self.buffer.push('-');
                            }
                        }
                        Some(c) => {
                            self.buffer.push(c);
                        }
                        None => {
                            let comment = std::mem::take(&mut self.buffer);
                            return Ok(Token::Comment(comment));
                        }
                    }
                }

                State::CommentEnd => {
                    match self.input.peek() {
                        Some('>') => {
                            self.input.next();
                            let comment = std::mem::take(&mut self.buffer);
                            self.state = State::Data;
                            return Ok(Token::Comment(comment));
                        }
                        _ => {
                            self.buffer.push_str("--");
                            self.state = State::Comment;
                        }
                    }
                }

                State::BogusComment => {
                    loop {
                        match self.input.next() {
                            Some('>') | None => {
                                let comment = std::mem::take(&mut self.buffer);
                                self.state = State::Data;
                                return Ok(Token::Comment(comment));
                            }
                            Some(c) => {
                                self.buffer.push(c);
                            }
                        }
                    }
                }

                State::BeforeDoctypeName => {
                    match self.input.peek() {
                        Some(c) if c.is_whitespace() => {
                            self.input.next();
                        }
                        Some(_) => {
                            self.buffer.clear();
                            self.state = State::DoctypeName;
                        }
                        None => {
                            return Err(ParseError::InvalidDoctype);
                        }
                    }
                }

                State::DoctypeName => {
                    match self.input.next() {
                        Some(c) if c.is_whitespace() => {
                            self.state = State::AfterDoctypeName;
                        }
                        Some('>') => {
                            let name = std::mem::take(&mut self.buffer);
                            self.state = State::Data;
                            return Ok(Token::Doctype {
                                name: name.to_lowercase(),
                                public_id: None,
                                system_id: None,
                            });
                        }
                        Some(c) => {
                            self.buffer.push(c);
                        }
                        None => {
                            return Err(ParseError::InvalidDoctype);
                        }
                    }
                }

                State::AfterDoctypeName => {
                    // Simplified: just consume until '>'
                    loop {
                        match self.input.next() {
                            Some('>') => {
                                let name = std::mem::take(&mut self.buffer);
                                self.state = State::Data;
                                return Ok(Token::Doctype {
                                    name: name.to_lowercase(),
                                    public_id: None,
                                    system_id: None,
                                });
                            }
                            Some(_) => continue,
                            None => {
                                return Err(ParseError::InvalidDoctype);
                            }
                        }
                    }
                }

                _ => {
                    self.state = State::Data;
                }
            }
        }
    }

    fn save_attribute(&mut self) {
        if !self.current_attr_name.is_empty() {
            self.current_attributes.insert(
                std::mem::take(&mut self.current_attr_name),
                std::mem::take(&mut self.current_attr_value),
            );
        }
    }

    fn emit_tag(&mut self) -> Token {
        let name = std::mem::take(&mut self.current_tag_name);
        let attributes = std::mem::take(&mut self.current_attributes);

        if self.is_end_tag {
            Token::EndTag { name }
        } else {
            Token::StartTag {
                name,
                attributes,
                self_closing: self.is_self_closing,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let mut tokenizer = Tokenizer::new("<div></div>");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 3); // StartTag, EndTag, Eof
        assert!(matches!(&tokens[0], Token::StartTag { name, .. } if name == "div"));
        assert!(matches!(&tokens[1], Token::EndTag { name } if name == "div"));
    }

    #[test]
    fn test_attributes() {
        let mut tokenizer = Tokenizer::new(r#"<div class="container" id="main"></div>"#);
        let tokens = tokenizer.tokenize().unwrap();
        
        if let Token::StartTag { attributes, .. } = &tokens[0] {
            assert_eq!(attributes.get("class"), Some(&"container".to_string()));
            assert_eq!(attributes.get("id"), Some(&"main".to_string()));
        } else {
            panic!("Expected StartTag");
        }
    }

    #[test]
    fn test_text() {
        let mut tokenizer = Tokenizer::new("<p>Hello, World!</p>");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 4); // StartTag, Text, EndTag, Eof
        assert!(matches!(&tokens[1], Token::Text(text) if text == "Hello, World!"));
    }

    #[test]
    fn test_doctype() {
        let mut tokenizer = Tokenizer::new("<!DOCTYPE html>");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert!(matches!(&tokens[0], Token::Doctype { name, .. } if name == "html"));
    }

    #[test]
    fn test_comment() {
        let mut tokenizer = Tokenizer::new("<!-- This is a comment -->");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert!(matches!(&tokens[0], Token::Comment(text) if text.contains("comment")));
    }

    #[test]
    fn test_self_closing() {
        let mut tokenizer = Tokenizer::new("<br/><img src=\"test.png\"/>");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert!(matches!(&tokens[0], Token::StartTag { self_closing: true, .. }));
        assert!(matches!(&tokens[1], Token::StartTag { self_closing: true, .. }));
    }
}
