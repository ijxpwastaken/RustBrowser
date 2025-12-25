//! JavaScript Tokenizer
//! 
//! Converts JavaScript source code into tokens.

use crate::JsError;

/// Token types in JavaScript
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Identifier(String),
    
    // Template literals
    TemplateHead(String),       // `text${
    TemplateMiddle(String),     // }text${
    TemplateTail(String),       // }text`
    NoSubstitutionTemplate(String), // `text` with no ${}
    
    // Regular expressions
    RegExp { pattern: String, flags: String },
    
    // Keywords
    Var,
    Let,
    Const,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    New,
    This,
    True,
    False,
    
    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    StarStar,       // ** (exponentiation)
    Slash,          // /
    Percent,        // %
    PlusPlus,       // ++
    MinusMinus,     // --
    Assign,         // =
    PlusAssign,     // +=
    MinusAssign,    // -=
    StarAssign,     // *=
    StarStarAssign, // **=
    SlashAssign,    // /=
    PercentAssign,  // %=
    Equal,          // ==
    StrictEqual,    // ===
    NotEqual,       // !=
    StrictNotEqual, // !==
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    And,            // &&
    Or,             // ||
    Not,            // !
    BitAnd,         // &
    BitOr,          // |
    BitXor,         // ^
    BitNot,         // ~
    LeftShift,      // <<
    RightShift,     // >>
    UnsignedRightShift, // >>>
    NullishCoalesce,// ??
    OptionalChain,  // ?.
    Spread,         // ...
    AndAssign,      // &&=
    OrAssign,       // ||=
    NullishAssign,  // ??=
    BitAndAssign,   // &=
    BitOrAssign,    // |=
    BitXorAssign,   // ^=
    LeftShiftAssign,    // <<=
    RightShiftAssign,   // >>=
    
    // Punctuation
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Comma,          // ,
    Dot,            // .
    Semicolon,      // ;
    Colon,          // :
    Question,       // ?
    Arrow,          // =>
    
    // More keywords
    Class,
    Extends,
    Super,
    Static,
    Get,
    Set,
    Async,
    Await,
    Yield,
    In,
    Instanceof,
    Typeof,
    Delete,
    Void,
    Try,
    Catch,
    Finally,
    Throw,
    Switch,
    Case,
    Default,
    Do,
    Export,
    Import,
    From,
    As,
    Of,
    
    // Special
    Eof,
}

/// A token with position information
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Token { token_type, line, column }
    }
}

/// JavaScript Tokenizer
pub struct Tokenizer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    column: usize,
    current_pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source,
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
            current_pos: 0,
        }
    }

    /// Tokenize the entire source code
    pub fn tokenize(&mut self) -> Result<Vec<Token>, JsError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = token.token_type == TokenType::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, JsError> {
        self.skip_whitespace_and_comments();
        
        let line = self.line;
        let column = self.column;
        
        let c = match self.peek() {
            Some(c) => c,
            None => return Ok(Token::new(TokenType::Eof, line, column)),
        };
        
        // Numbers
        if c.is_ascii_digit() {
            return self.read_number();
        }
        
        // Strings
        if c == '"' || c == '\'' || c == '`' {
            return self.read_string();
        }
        
        // Identifiers and keywords
        if c.is_ascii_alphabetic() || c == '_' || c == '$' {
            return self.read_identifier();
        }
        
        // Operators and punctuation
        self.read_operator()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                Some('/') => {
                    // Check for comments
                    let next = self.peek_next();
                    if next == Some('/') {
                        // Single line comment
                        self.advance();
                        self.advance();
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if next == Some('*') {
                        // Multi-line comment
                        self.advance();
                        self.advance();
                        loop {
                            match self.peek() {
                                None => break,
                                Some('*') if self.peek_next() == Some('/') => {
                                    self.advance();
                                    self.advance();
                                    break;
                                }
                                Some('\n') => {
                                    self.advance();
                                    self.line += 1;
                                    self.column = 1;
                                }
                                _ => {
                                    self.advance();
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, JsError> {
        let line = self.line;
        let column = self.column;
        let mut num_str = String::new();
        
        // Integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Decimal part
        if self.peek() == Some('.') {
            num_str.push('.');
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        // Exponent part
        if let Some('e') | Some('E') = self.peek() {
            num_str.push('e');
            self.advance();
            if let Some('+') | Some('-') = self.peek() {
                num_str.push(self.peek().unwrap());
                self.advance();
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        let value = num_str.parse::<f64>()
            .map_err(|_| JsError::SyntaxError(format!("Invalid number: {}", num_str)))?;
        
        Ok(Token::new(TokenType::Number(value), line, column))
    }

    fn read_string(&mut self) -> Result<Token, JsError> {
        let line = self.line;
        let column = self.column;
        let quote = self.advance().unwrap();
        
        // Handle template literals
        if quote == '`' {
            return self.read_template_literal(line, column);
        }
        
        let mut value = String::new();
        
        loop {
            match self.peek() {
                None => return Err(JsError::SyntaxError("Unterminated string".to_string())),
                Some(c) if c == quote => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    value.push_str(&self.read_escape_sequence()?);
                }
                Some('\n') if quote != '`' => {
                    return Err(JsError::SyntaxError("Unterminated string".to_string()));
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
            }
        }
        
        Ok(Token::new(TokenType::String(value), line, column))
    }
    
    fn read_escape_sequence(&mut self) -> Result<String, JsError> {
        match self.advance() {
            Some('n') => Ok("\n".to_string()),
            Some('t') => Ok("\t".to_string()),
            Some('r') => Ok("\r".to_string()),
            Some('\\') => Ok("\\".to_string()),
            Some('"') => Ok("\"".to_string()),
            Some('\'') => Ok("'".to_string()),
            Some('`') => Ok("`".to_string()),
            Some('$') => Ok("$".to_string()),
            Some('0') => Ok("\0".to_string()),
            Some('b') => Ok("\x08".to_string()),
            Some('f') => Ok("\x0C".to_string()),
            Some('v') => Ok("\x0B".to_string()),
            Some('x') => {
                // Hex escape \xHH
                let mut hex = String::new();
                for _ in 0..2 {
                    match self.advance() {
                        Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                        _ => return Err(JsError::SyntaxError("Invalid hex escape".to_string())),
                    }
                }
                let code = u32::from_str_radix(&hex, 16)
                    .map_err(|_| JsError::SyntaxError("Invalid hex escape".to_string()))?;
                Ok(char::from_u32(code).unwrap_or('\u{FFFD}').to_string())
            }
            Some('u') => {
                // Unicode escape \uHHHH or \u{H...}
                if self.peek() == Some('{') {
                    self.advance();
                    let mut hex = String::new();
                    while let Some(c) = self.peek() {
                        if c == '}' {
                            self.advance();
                            break;
                        }
                        if c.is_ascii_hexdigit() {
                            hex.push(c);
                            self.advance();
                        } else {
                            return Err(JsError::SyntaxError("Invalid unicode escape".to_string()));
                        }
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| JsError::SyntaxError("Invalid unicode escape".to_string()))?;
                    Ok(char::from_u32(code).unwrap_or('\u{FFFD}').to_string())
                } else {
                    let mut hex = String::new();
                    for _ in 0..4 {
                        match self.advance() {
                            Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                            _ => return Err(JsError::SyntaxError("Invalid unicode escape".to_string())),
                        }
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| JsError::SyntaxError("Invalid unicode escape".to_string()))?;
                    Ok(char::from_u32(code).unwrap_or('\u{FFFD}').to_string())
                }
            }
            Some('\n') => {
                // Line continuation
                self.line += 1;
                self.column = 1;
                Ok(String::new())
            }
            Some(c) => Ok(c.to_string()),
            None => Err(JsError::SyntaxError("Unterminated string".to_string())),
        }
    }
    
    fn read_template_literal(&mut self, line: usize, column: usize) -> Result<Token, JsError> {
        let mut value = String::new();
        
        loop {
            match self.peek() {
                None => return Err(JsError::SyntaxError("Unterminated template literal".to_string())),
                Some('`') => {
                    self.advance();
                    // End of template, no substitutions
                    return Ok(Token::new(TokenType::NoSubstitutionTemplate(value), line, column));
                }
                Some('$') if self.peek_next() == Some('{') => {
                    self.advance(); // consume $
                    self.advance(); // consume {
                    // This is the head of a template with substitutions
                    return Ok(Token::new(TokenType::TemplateHead(value), line, column));
                }
                Some('\\') => {
                    self.advance();
                    value.push_str(&self.read_escape_sequence()?);
                }
                Some('\n') => {
                    value.push('\n');
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
            }
        }
    }
    
    /// Continue reading a template literal after a ${...} expression
    /// This should be called after parsing the expression inside ${}
    pub fn read_template_continuation(&mut self) -> Result<Token, JsError> {
        let line = self.line;
        let column = self.column;
        
        // We should be right after the } of ${...}
        let mut value = String::new();
        
        loop {
            match self.peek() {
                None => return Err(JsError::SyntaxError("Unterminated template literal".to_string())),
                Some('`') => {
                    self.advance();
                    // End of template
                    return Ok(Token::new(TokenType::TemplateTail(value), line, column));
                }
                Some('$') if self.peek_next() == Some('{') => {
                    self.advance(); // consume $
                    self.advance(); // consume {
                    // Another substitution
                    return Ok(Token::new(TokenType::TemplateMiddle(value), line, column));
                }
                Some('\\') => {
                    self.advance();
                    value.push_str(&self.read_escape_sequence()?);
                }
                Some('\n') => {
                    value.push('\n');
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
            }
        }
    }

    fn read_identifier(&mut self) -> Result<Token, JsError> {
        let line = self.line;
        let column = self.column;
        let mut name = String::new();
        
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check for keywords
        let token_type = match name.as_str() {
            "var" => TokenType::Var,
            "let" => TokenType::Let,
            "const" => TokenType::Const,
            "function" => TokenType::Function,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "new" => TokenType::New,
            "this" => TokenType::This,
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            "null" => TokenType::Null,
            "undefined" => TokenType::Undefined,
            "class" => TokenType::Class,
            "extends" => TokenType::Extends,
            "super" => TokenType::Super,
            "static" => TokenType::Static,
            "get" => TokenType::Get,
            "set" => TokenType::Set,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "yield" => TokenType::Yield,
            "in" => TokenType::In,
            "instanceof" => TokenType::Instanceof,
            "typeof" => TokenType::Typeof,
            "delete" => TokenType::Delete,
            "void" => TokenType::Void,
            "try" => TokenType::Try,
            "catch" => TokenType::Catch,
            "finally" => TokenType::Finally,
            "throw" => TokenType::Throw,
            "switch" => TokenType::Switch,
            "case" => TokenType::Case,
            "default" => TokenType::Default,
            "do" => TokenType::Do,
            "export" => TokenType::Export,
            "import" => TokenType::Import,
            "from" => TokenType::From,
            "as" => TokenType::As,
            "of" => TokenType::Of,
            _ => TokenType::Identifier(name),
        };
        
        Ok(Token::new(token_type, line, column))
    }

    fn read_operator(&mut self) -> Result<Token, JsError> {
        let line = self.line;
        let column = self.column;
        let c = self.advance().unwrap();
        
        let token_type = match c {
            '+' => {
                if self.peek() == Some('+') {
                    self.advance();
                    TokenType::PlusPlus
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenType::PlusAssign
                } else {
                    TokenType::Plus
                }
            }
            '-' => {
                if self.peek() == Some('-') {
                    self.advance();
                    TokenType::MinusMinus
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenType::MinusAssign
                } else {
                    TokenType::Minus
                }
            }
            '*' => {
                if self.peek() == Some('*') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenType::StarStarAssign
                    } else {
                        TokenType::StarStar
                    }
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenType::StarAssign
                } else {
                    TokenType::Star
                }
            }
            '/' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::SlashAssign
                } else {
                    TokenType::Slash
                }
            }
            '%' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::PercentAssign
                } else {
                    TokenType::Percent
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenType::StrictEqual
                    } else {
                        TokenType::Equal
                    }
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Assign
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenType::StrictNotEqual
                    } else {
                        TokenType::NotEqual
                    }
                } else {
                    TokenType::Not
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::LessEqual
                } else if self.peek() == Some('<') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenType::LeftShiftAssign
                    } else {
                        TokenType::LeftShift
                    }
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::GreaterEqual
                } else if self.peek() == Some('>') {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        TokenType::UnsignedRightShift
                    } else if self.peek() == Some('=') {
                        self.advance();
                        TokenType::RightShiftAssign
                    } else {
                        TokenType::RightShift
                    }
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenType::AndAssign
                    } else {
                        TokenType::And
                    }
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenType::BitAndAssign
                } else {
                    TokenType::BitAnd
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenType::OrAssign
                    } else {
                        TokenType::Or
                    }
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenType::BitOrAssign
                } else {
                    TokenType::BitOr
                }
            }
            '^' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::BitXorAssign
                } else {
                    TokenType::BitXor
                }
            }
            '~' => TokenType::BitNot,
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            ',' => TokenType::Comma,
            '.' => {
                if self.peek() == Some('.') && self.peek_next() == Some('.') {
                    self.advance();
                    self.advance();
                    TokenType::Spread
                } else {
                    TokenType::Dot
                }
            }
            ';' => TokenType::Semicolon,
            ':' => TokenType::Colon,
            '?' => {
                if self.peek() == Some('?') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenType::NullishAssign
                    } else {
                        TokenType::NullishCoalesce
                    }
                } else if self.peek() == Some('.') {
                    self.advance();
                    TokenType::OptionalChain
                } else {
                    TokenType::Question
                }
            }
            // Handle any remaining chars gracefully
            '@' | '#' | '\\' => {
                // Skip decorators and other special chars
                TokenType::Identifier(c.to_string())
            }
            _ => {
                // Be more lenient - skip unknown chars instead of erroring
                TokenType::Identifier(c.to_string())
            }
        };
        
        Ok(Token::new(token_type, line, column))
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn peek_next(&self) -> Option<char> {
        let mut iter = self.source[self.current_pos..].chars();
        iter.next();
        iter.next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c.is_some() {
            self.current_pos += 1;
            self.column += 1;
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numbers() {
        let mut tokenizer = Tokenizer::new("123 45.67 1e10");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert!(matches!(&tokens[0].token_type, TokenType::Number(n) if *n == 123.0));
        assert!(matches!(&tokens[1].token_type, TokenType::Number(n) if (*n - 45.67).abs() < 0.001));
    }

    #[test]
    fn test_strings() {
        let mut tokenizer = Tokenizer::new("'hello' \"world\"");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert!(matches!(&tokens[0].token_type, TokenType::String(s) if s == "hello"));
        assert!(matches!(&tokens[1].token_type, TokenType::String(s) if s == "world"));
    }

    #[test]
    fn test_keywords() {
        let mut tokenizer = Tokenizer::new("var let const function if else");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Var);
        assert_eq!(tokens[1].token_type, TokenType::Let);
        assert_eq!(tokens[2].token_type, TokenType::Const);
        assert_eq!(tokens[3].token_type, TokenType::Function);
    }

    #[test]
    fn test_operators() {
        let mut tokenizer = Tokenizer::new("+ - * / === !==");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Star);
        assert_eq!(tokens[3].token_type, TokenType::Slash);
        assert_eq!(tokens[4].token_type, TokenType::StrictEqual);
        assert_eq!(tokens[5].token_type, TokenType::StrictNotEqual);
    }
}
