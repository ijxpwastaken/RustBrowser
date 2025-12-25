//! CSS Tokenizer
//!
//! Tokenizes CSS input into a stream of tokens.

use std::iter::Peekable;
use std::str::Chars;

/// CSS Token types
#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    /// Identifier (property names, tag names, etc.)
    Ident(String),
    /// Hash (id selectors like #main)
    Hash(String),
    /// String literal
    String(String),
    /// Number
    Number(f64),
    /// Percentage (50%)
    Percentage(f64),
    /// Dimension (10px, 2em, etc.)
    Dimension { value: f64, unit: String },
    /// Function (rgb(, url(, etc.)
    Function(String),
    /// @-keyword (@media, @keyframes)
    AtKeyword(String),
    /// URL
    Url(String),
    /// Colon (:)
    Colon,
    /// Semicolon (;)
    Semicolon,
    /// Comma (,)
    Comma,
    /// Left brace ({)
    LeftBrace,
    /// Right brace (})
    RightBrace,
    /// Left paren (()
    LeftParen,
    /// Right paren ())
    RightParen,
    /// Left bracket ([)
    LeftBracket,
    /// Right bracket (])
    RightBracket,
    /// Dot (.)
    Dot,
    /// Hash symbol (#) - for colors
    HashSymbol,
    /// Greater than (>)
    Greater,
    /// Plus (+)
    Plus,
    /// Tilde (~)
    Tilde,
    /// Star (*)
    Star,
    /// Equals (=)
    Equals,
    /// Pipe (|)
    Pipe,
    /// Caret (^)
    Caret,
    /// Dollar ($)
    Dollar,
    /// Exclamation (!)
    Exclamation,
    /// Whitespace
    Whitespace,
    /// Comment
    Comment(String),
    /// End of file
    Eof,
    /// Delim (any other single character)
    Delim(char),
}

/// CSS Tokenizer
pub struct CssTokenizer<'a> {
    input: Peekable<Chars<'a>>,
    position: usize,
}

impl<'a> CssTokenizer<'a> {
    /// Create a new tokenizer
    pub fn new(input: &'a str) -> Self {
        CssTokenizer {
            input: input.chars().peekable(),
            position: 0,
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Vec<CssToken> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            if token == CssToken::Eof {
                tokens.push(token);
                break;
            }
            // Skip whitespace and comments for simpler parsing
            if !matches!(token, CssToken::Whitespace | CssToken::Comment(_)) {
                tokens.push(token);
            }
        }

        tokens
    }

    /// Get the next token
    pub fn next_token(&mut self) -> CssToken {
        self.skip_whitespace_and_comments()
    }

    fn skip_whitespace_and_comments(&mut self) -> CssToken {
        loop {
            match self.peek() {
                None => return CssToken::Eof,
                Some(c) if c.is_whitespace() => {
                    self.consume_whitespace();
                    // Continue to skip
                }
                Some('/') => {
                    if self.peek_next() == Some('*') {
                        self.consume_comment();
                        // Continue to skip
                    } else {
                        return self.consume_token();
                    }
                }
                _ => return self.consume_token(),
            }
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn consume_comment(&mut self) {
        // Skip /*
        self.advance();
        self.advance();

        loop {
            match self.advance() {
                None => break,
                Some('*') if self.peek() == Some('/') => {
                    self.advance();
                    break;
                }
                _ => {}
            }
        }
    }

    fn consume_token(&mut self) -> CssToken {
        match self.peek() {
            None => CssToken::Eof,
            Some(c) => match c {
                ':' => { self.advance(); CssToken::Colon }
                ';' => { self.advance(); CssToken::Semicolon }
                ',' => { self.advance(); CssToken::Comma }
                '{' => { self.advance(); CssToken::LeftBrace }
                '}' => { self.advance(); CssToken::RightBrace }
                '(' => { self.advance(); CssToken::LeftParen }
                ')' => { self.advance(); CssToken::RightParen }
                '[' => { self.advance(); CssToken::LeftBracket }
                ']' => { self.advance(); CssToken::RightBracket }
                '>' => { self.advance(); CssToken::Greater }
                '+' => { self.advance(); CssToken::Plus }
                '~' => { self.advance(); CssToken::Tilde }
                '*' => { self.advance(); CssToken::Star }
                '=' => { self.advance(); CssToken::Equals }
                '|' => { self.advance(); CssToken::Pipe }
                '^' => { self.advance(); CssToken::Caret }
                '$' => { self.advance(); CssToken::Dollar }
                '!' => { self.advance(); CssToken::Exclamation }
                '.' => { self.advance(); CssToken::Dot }
                '#' => {
                    self.advance();
                    // Check if it's an ID selector or hash
                    if let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' || c == '-' {
                            let name = self.consume_ident_like();
                            return CssToken::Hash(name);
                        }
                    }
                    CssToken::HashSymbol
                }
                '@' => {
                    self.advance();
                    let name = self.consume_ident_like();
                    CssToken::AtKeyword(name)
                }
                '"' | '\'' => self.consume_string(),
                c if c.is_ascii_digit() || c == '-' || c == '+' => self.consume_number_or_ident(),
                c if c.is_alphabetic() || c == '_' || c == '-' => self.consume_ident_or_function(),
                c => {
                    self.advance();
                    CssToken::Delim(c)
                }
            }
        }
    }

    fn consume_string(&mut self) -> CssToken {
        let quote = self.advance().unwrap();
        let mut s = String::new();

        loop {
            match self.advance() {
                None => break,
                Some(c) if c == quote => break,
                Some('\\') => {
                    if let Some(escaped) = self.advance() {
                        s.push(escaped);
                    }
                }
                Some(c) => s.push(c),
            }
        }

        CssToken::String(s)
    }

    fn consume_number_or_ident(&mut self) -> CssToken {
        let first = self.peek().unwrap();

        // If it starts with - and next is not a digit, it's an ident
        if first == '-' && !matches!(self.peek_next(), Some(c) if c.is_ascii_digit()) {
            return self.consume_ident_or_function();
        }

        // Consume the number
        let mut num_str = String::new();

        // Optional sign
        if self.peek() == Some('-') || self.peek() == Some('+') {
            num_str.push(self.advance().unwrap());
        }

        // Integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        // Decimal part
        if self.peek() == Some('.') {
            num_str.push(self.advance().unwrap());
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(self.advance().unwrap());
                } else {
                    break;
                }
            }
        }

        let value: f64 = num_str.parse().unwrap_or(0.0);

        // Check for unit or percentage
        if self.peek() == Some('%') {
            self.advance();
            return CssToken::Percentage(value);
        }

        if let Some(c) = self.peek() {
            if c.is_alphabetic() || c == '_' {
                let unit = self.consume_ident_like();
                return CssToken::Dimension { value, unit };
            }
        }

        CssToken::Number(value)
    }

    fn consume_ident_or_function(&mut self) -> CssToken {
        let name = self.consume_ident_like();

        if self.peek() == Some('(') {
            self.advance();

            // Special handling for url()
            if name.to_lowercase() == "url" {
                return self.consume_url();
            }

            return CssToken::Function(name);
        }

        CssToken::Ident(name)
    }

    fn consume_ident_like(&mut self) -> String {
        let mut name = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                name.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        name
    }

    fn consume_url(&mut self) -> CssToken {
        // Skip whitespace
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }

        // Check for quoted string
        if matches!(self.peek(), Some('"') | Some('\'')) {
            if let CssToken::String(s) = self.consume_string() {
                // Skip whitespace and closing paren
                while let Some(c) = self.peek() {
                    if c.is_whitespace() {
                        self.advance();
                    } else {
                        break;
                    }
                }
                if self.peek() == Some(')') {
                    self.advance();
                }
                return CssToken::Url(s);
            }
        }

        // Unquoted URL
        let mut url = String::new();
        while let Some(c) = self.peek() {
            if c == ')' || c.is_whitespace() {
                break;
            }
            url.push(self.advance().unwrap());
        }

        // Skip to closing paren
        while let Some(c) = self.peek() {
            if c == ')' {
                self.advance();
                break;
            }
            self.advance();
        }

        CssToken::Url(url)
    }

    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn peek_next(&self) -> Option<char> {
        let mut clone = self.input.clone();
        clone.next();
        clone.next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if c.is_some() {
            self.position += 1;
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rule() {
        let mut tokenizer = CssTokenizer::new("div { color: red; }");
        let tokens = tokenizer.tokenize();

        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "div"));
        assert!(matches!(tokens[1], CssToken::LeftBrace));
    }

    #[test]
    fn test_selectors() {
        let mut tokenizer = CssTokenizer::new(".class #id div.foo");
        let tokens = tokenizer.tokenize();

        assert!(matches!(tokens[0], CssToken::Dot));
        assert!(matches!(&tokens[1], CssToken::Ident(s) if s == "class"));
        assert!(matches!(&tokens[2], CssToken::Hash(s) if s == "id"));
    }

    #[test]
    fn test_dimensions() {
        let mut tokenizer = CssTokenizer::new("10px 50% 2em 1.5rem");
        let tokens = tokenizer.tokenize();

        assert!(matches!(&tokens[0], CssToken::Dimension { value: 10.0, unit } if unit == "px"));
        assert!(matches!(tokens[1], CssToken::Percentage(50.0)));
    }
}
