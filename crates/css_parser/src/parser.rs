//! CSS Parser
//!
//! Parses CSS into a stylesheet with rules, selectors, and declarations.

use crate::tokenizer::{CssToken, CssTokenizer};

/// A CSS stylesheet
#[derive(Debug, Default, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

/// A CSS rule (selector + declarations)
#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

/// A CSS selector
#[derive(Debug, Clone)]
pub struct Selector {
    pub parts: Vec<SelectorPart>,
    pub specificity: Specificity,
}

/// Part of a selector
#[derive(Debug, Clone)]
pub enum SelectorPart {
    /// Type/tag selector (div, p, span)
    Type(String),
    /// Class selector (.class)
    Class(String),
    /// ID selector (#id)
    Id(String),
    /// Universal selector (*)
    Universal,
    /// Attribute selector ([attr] or [attr=value])
    Attribute {
        name: String,
        operator: Option<AttributeOperator>,
        value: Option<String>,
    },
    /// Pseudo-class (:hover, :first-child)
    PseudoClass(String),
    /// Pseudo-element (::before, ::after)
    PseudoElement(String),
    /// Combinator (space, >, +, ~)
    Combinator(Combinator),
}

/// Attribute selector operators
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeOperator {
    /// [attr=value] - exact match
    Equals,
    /// [attr~=value] - word match
    Includes,
    /// [attr|=value] - prefix match (lang)
    DashMatch,
    /// [attr^=value] - starts with
    PrefixMatch,
    /// [attr$=value] - ends with
    SuffixMatch,
    /// [attr*=value] - contains
    SubstringMatch,
}

/// Selector combinator
#[derive(Debug, Clone, PartialEq)]
pub enum Combinator {
    /// Descendant (space)
    Descendant,
    /// Child (>)
    Child,
    /// Adjacent sibling (+)
    Adjacent,
    /// General sibling (~)
    Sibling,
}

/// Selector specificity (a, b, c)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Specificity {
    /// ID selectors
    pub a: u32,
    /// Class selectors, attributes, pseudo-classes
    pub b: u32,
    /// Type selectors, pseudo-elements
    pub c: u32,
}

/// A CSS declaration (property: value)
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: Value,
    pub important: bool,
}

/// CSS value
#[derive(Debug, Clone)]
pub enum Value {
    /// Keyword (auto, inherit, block, etc.)
    Keyword(String),
    /// Length value
    Length(f64, Unit),
    /// Percentage
    Percentage(f64),
    /// Color
    Color(Color),
    /// Number
    Number(f64),
    /// String
    String(String),
    /// URL
    Url(String),
    /// Function call (calc(), rgb(), etc.)
    Function { name: String, args: Vec<Value> },
    /// Multiple values (for shorthand properties)
    List(Vec<Value>),
}

/// CSS length unit
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px,
    Em,
    Rem,
    Percent,
    Vh,
    Vw,
    Pt,
    Cm,
    Mm,
    In,
    Fr,
    Ch,
    Ex,
    Auto,
}

impl Unit {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "px" => Unit::Px,
            "em" => Unit::Em,
            "rem" => Unit::Rem,
            "%" => Unit::Percent,
            "vh" => Unit::Vh,
            "vw" => Unit::Vw,
            "pt" => Unit::Pt,
            "cm" => Unit::Cm,
            "mm" => Unit::Mm,
            "in" => Unit::In,
            "fr" => Unit::Fr,
            "ch" => Unit::Ch,
            "ex" => Unit::Ex,
            _ => Unit::Px,
        }
    }
}

/// CSS color
#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 128, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };

    /// Parse a color from a string
    pub fn from_str(s: &str) -> Option<Color> {
        let s = s.trim().to_lowercase();

        // Named colors
        let named = match s.as_str() {
            "transparent" => Some(Color::TRANSPARENT),
            "black" => Some(Color::BLACK),
            "white" => Some(Color::WHITE),
            "red" => Some(Color::RED),
            "green" => Some(Color::GREEN),
            "blue" => Some(Color::BLUE),
            "gray" | "grey" => Some(Color { r: 128, g: 128, b: 128, a: 255 }),
            "silver" => Some(Color { r: 192, g: 192, b: 192, a: 255 }),
            "navy" => Some(Color { r: 0, g: 0, b: 128, a: 255 }),
            "teal" => Some(Color { r: 0, g: 128, b: 128, a: 255 }),
            "aqua" | "cyan" => Some(Color { r: 0, g: 255, b: 255, a: 255 }),
            "maroon" => Some(Color { r: 128, g: 0, b: 0, a: 255 }),
            "purple" => Some(Color { r: 128, g: 0, b: 128, a: 255 }),
            "fuchsia" | "magenta" => Some(Color { r: 255, g: 0, b: 255, a: 255 }),
            "olive" => Some(Color { r: 128, g: 128, b: 0, a: 255 }),
            "yellow" => Some(Color { r: 255, g: 255, b: 0, a: 255 }),
            "lime" => Some(Color { r: 0, g: 255, b: 0, a: 255 }),
            "orange" => Some(Color { r: 255, g: 165, b: 0, a: 255 }),
            "pink" => Some(Color { r: 255, g: 192, b: 203, a: 255 }),
            "brown" => Some(Color { r: 165, g: 42, b: 42, a: 255 }),
            "coral" => Some(Color { r: 255, g: 127, b: 80, a: 255 }),
            "crimson" => Some(Color { r: 220, g: 20, b: 60, a: 255 }),
            "gold" => Some(Color { r: 255, g: 215, b: 0, a: 255 }),
            "indigo" => Some(Color { r: 75, g: 0, b: 130, a: 255 }),
            "violet" => Some(Color { r: 238, g: 130, b: 238, a: 255 }),
            "turquoise" => Some(Color { r: 64, g: 224, b: 208, a: 255 }),
            "tomato" => Some(Color { r: 255, g: 99, b: 71, a: 255 }),
            "skyblue" => Some(Color { r: 135, g: 206, b: 235, a: 255 }),
            "salmon" => Some(Color { r: 250, g: 128, b: 114, a: 255 }),
            "royalblue" => Some(Color { r: 65, g: 105, b: 225, a: 255 }),
            "plum" => Some(Color { r: 221, g: 160, b: 221, a: 255 }),
            "orchid" => Some(Color { r: 218, g: 112, b: 214, a: 255 }),
            "khaki" => Some(Color { r: 240, g: 230, b: 140, a: 255 }),
            "ivory" => Some(Color { r: 255, g: 255, b: 240, a: 255 }),
            "honeydew" => Some(Color { r: 240, g: 255, b: 240, a: 255 }),
            "hotpink" => Some(Color { r: 255, g: 105, b: 180, a: 255 }),
            "lightgray" | "lightgrey" => Some(Color { r: 211, g: 211, b: 211, a: 255 }),
            "darkgray" | "darkgrey" => Some(Color { r: 169, g: 169, b: 169, a: 255 }),
            "lightblue" => Some(Color { r: 173, g: 216, b: 230, a: 255 }),
            "lightgreen" => Some(Color { r: 144, g: 238, b: 144, a: 255 }),
            "darkblue" => Some(Color { r: 0, g: 0, b: 139, a: 255 }),
            "darkgreen" => Some(Color { r: 0, g: 100, b: 0, a: 255 }),
            "darkred" => Some(Color { r: 139, g: 0, b: 0, a: 255 }),
            "beige" => Some(Color { r: 245, g: 245, b: 220, a: 255 }),
            "azure" => Some(Color { r: 240, g: 255, b: 255, a: 255 }),
            "aliceblue" => Some(Color { r: 240, g: 248, b: 255, a: 255 }),
            "antiquewhite" => Some(Color { r: 250, g: 235, b: 215, a: 255 }),
            _ => None,
        };

        if named.is_some() {
            return named;
        }

        // Hex color
        if s.starts_with('#') {
            return Color::from_hex(&s[1..]);
        }

        None
    }

    /// Parse hex color
    pub fn from_hex(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color { r, g, b, a: 255 })
            }
            4 => {
                // #RGBA -> #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?;
                Some(Color { r, g, b, a })
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color { r, g, b, a: 255 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color { r, g, b, a })
            }
            _ => None,
        }
    }

    /// Parse rgb(r, g, b) or rgba(r, g, b, a)
    pub fn from_rgb(r: f64, g: f64, b: f64, a: f64) -> Color {
        Color {
            r: (r.clamp(0.0, 255.0)) as u8,
            g: (g.clamp(0.0, 255.0)) as u8,
            b: (b.clamp(0.0, 255.0)) as u8,
            a: (a * 255.0).clamp(0.0, 255.0) as u8,
        }
    }
}

/// CSS Parser
pub struct CssParser {
    tokens: Vec<CssToken>,
    current: usize,
}

impl CssParser {
    pub fn new(css: &str) -> Self {
        let mut tokenizer = CssTokenizer::new(css);
        let tokens = tokenizer.tokenize();
        CssParser { tokens, current: 0 }
    }

    /// Parse the entire stylesheet
    pub fn parse(&mut self) -> Stylesheet {
        let mut rules = Vec::new();

        while !self.is_at_end() {
            // Skip @ rules for now
            if self.check(CssToken::AtKeyword(String::new())) {
                self.skip_at_rule();
                continue;
            }

            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            } else {
                // Skip to next rule
                while !self.is_at_end() && !self.check(CssToken::RightBrace) {
                    self.advance();
                }
                if self.check(CssToken::RightBrace) {
                    self.advance();
                }
            }
        }

        Stylesheet { rules }
    }

    fn skip_at_rule(&mut self) {
        // Consume @-keyword
        self.advance();

        // Skip until { or ;
        let mut depth = 0;
        while !self.is_at_end() {
            match self.peek() {
                CssToken::LeftBrace => {
                    depth += 1;
                    self.advance();
                }
                CssToken::RightBrace => {
                    depth -= 1;
                    self.advance();
                    if depth <= 0 {
                        break;
                    }
                }
                CssToken::Semicolon if depth == 0 => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selector_list()?;

        if !self.match_token(CssToken::LeftBrace) {
            return None;
        }

        let declarations = self.parse_declarations();

        if !self.match_token(CssToken::RightBrace) {
            // Error recovery - skip until }
            while !self.is_at_end() && !self.check(CssToken::RightBrace) {
                self.advance();
            }
            self.advance();
        }

        Some(Rule { selectors, declarations })
    }

    fn parse_selector_list(&mut self) -> Option<Vec<Selector>> {
        let mut selectors = Vec::new();

        loop {
            if let Some(selector) = self.parse_selector() {
                selectors.push(selector);
            } else {
                break;
            }

            if !self.match_token(CssToken::Comma) {
                break;
            }
        }

        if selectors.is_empty() {
            None
        } else {
            Some(selectors)
        }
    }

    fn parse_selector(&mut self) -> Option<Selector> {
        let mut parts = Vec::new();
        let mut specificity = Specificity::default();
        let mut last_was_combinator = true;

        loop {
            match self.peek() {
                CssToken::LeftBrace | CssToken::Comma | CssToken::Eof => break,

                CssToken::Star => {
                    self.advance();
                    parts.push(SelectorPart::Universal);
                    last_was_combinator = false;
                }

                CssToken::Ident(name) => {
                    self.advance();
                    parts.push(SelectorPart::Type(name));
                    specificity.c += 1;
                    last_was_combinator = false;
                }

                CssToken::Hash(name) => {
                    self.advance();
                    parts.push(SelectorPart::Id(name));
                    specificity.a += 1;
                    last_was_combinator = false;
                }

                CssToken::Dot => {
                    self.advance();
                    if let CssToken::Ident(name) = self.peek() {
                        self.advance();
                        parts.push(SelectorPart::Class(name));
                        specificity.b += 1;
                    }
                    last_was_combinator = false;
                }

                CssToken::Colon => {
                    self.advance();
                    // Check for ::pseudo-element
                    if self.check(CssToken::Colon) {
                        self.advance();
                        if let CssToken::Ident(name) = self.peek() {
                            self.advance();
                            parts.push(SelectorPart::PseudoElement(name));
                            specificity.c += 1;
                        }
                    } else if let CssToken::Ident(name) = self.peek() {
                        self.advance();
                        parts.push(SelectorPart::PseudoClass(name));
                        specificity.b += 1;
                    }
                    last_was_combinator = false;
                }

                CssToken::LeftBracket => {
                    self.advance();
                    if let Some((attr_part, spec_add)) = self.parse_attribute_selector() {
                        parts.push(attr_part);
                        specificity.b += spec_add;
                    }
                    last_was_combinator = false;
                }

                CssToken::Greater => {
                    self.advance();
                    if !last_was_combinator {
                        parts.push(SelectorPart::Combinator(Combinator::Child));
                        last_was_combinator = true;
                    }
                }

                CssToken::Plus => {
                    self.advance();
                    if !last_was_combinator {
                        parts.push(SelectorPart::Combinator(Combinator::Adjacent));
                        last_was_combinator = true;
                    }
                }

                CssToken::Tilde => {
                    self.advance();
                    if !last_was_combinator {
                        parts.push(SelectorPart::Combinator(Combinator::Sibling));
                        last_was_combinator = true;
                    }
                }

                _ => {
                    // Add descendant combinator between parts if needed
                    if !last_was_combinator && !parts.is_empty() {
                        match self.peek() {
                            CssToken::Ident(_) | CssToken::Hash(_) | CssToken::Dot |
                            CssToken::Star | CssToken::LeftBracket | CssToken::Colon => {
                                parts.push(SelectorPart::Combinator(Combinator::Descendant));
                                last_was_combinator = true;
                                continue;
                            }
                            _ => {}
                        }
                    }
                    break;
                }
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(Selector { parts, specificity })
        }
    }

    fn parse_attribute_selector(&mut self) -> Option<(SelectorPart, u32)> {
        let name = match self.peek() {
            CssToken::Ident(n) => {
                self.advance();
                n
            }
            _ => return None,
        };

        let (operator, value) = match self.peek() {
            CssToken::RightBracket => {
                self.advance();
                return Some((
                    SelectorPart::Attribute { name, operator: None, value: None },
                    1,
                ));
            }
            CssToken::Equals => {
                self.advance();
                (Some(AttributeOperator::Equals), self.parse_attr_value())
            }
            CssToken::Tilde if self.peek_next() == Some(CssToken::Equals) => {
                self.advance();
                self.advance();
                (Some(AttributeOperator::Includes), self.parse_attr_value())
            }
            CssToken::Pipe if self.peek_next() == Some(CssToken::Equals) => {
                self.advance();
                self.advance();
                (Some(AttributeOperator::DashMatch), self.parse_attr_value())
            }
            CssToken::Caret if self.peek_next() == Some(CssToken::Equals) => {
                self.advance();
                self.advance();
                (Some(AttributeOperator::PrefixMatch), self.parse_attr_value())
            }
            CssToken::Dollar if self.peek_next() == Some(CssToken::Equals) => {
                self.advance();
                self.advance();
                (Some(AttributeOperator::SuffixMatch), self.parse_attr_value())
            }
            CssToken::Star if self.peek_next() == Some(CssToken::Equals) => {
                self.advance();
                self.advance();
                (Some(AttributeOperator::SubstringMatch), self.parse_attr_value())
            }
            _ => (None, None),
        };

        self.match_token(CssToken::RightBracket);

        Some((
            SelectorPart::Attribute { name, operator, value },
            1,
        ))
    }

    fn parse_attr_value(&mut self) -> Option<String> {
        match self.peek() {
            CssToken::Ident(s) | CssToken::String(s) => {
                self.advance();
                Some(s)
            }
            _ => None,
        }
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        while !self.is_at_end() && !self.check(CssToken::RightBrace) {
            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            }
        }

        declarations
    }

    fn parse_declaration(&mut self) -> Option<Declaration> {
        let property = match self.peek() {
            CssToken::Ident(name) => {
                self.advance();
                name.to_lowercase()
            }
            _ => {
                // Skip to next declaration or end
                while !self.is_at_end() {
                    match self.peek() {
                        CssToken::Semicolon => {
                            self.advance();
                            return None;
                        }
                        CssToken::RightBrace => return None,
                        _ => {
                            self.advance();
                        }
                    }
                }
                return None;
            }
        };

        if !self.match_token(CssToken::Colon) {
            return None;
        }

        let value = self.parse_value();
        let important = self.check_important();

        self.match_token(CssToken::Semicolon);

        Some(Declaration { property, value, important })
    }

    fn parse_value(&mut self) -> Value {
        let mut values = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                CssToken::Semicolon | CssToken::RightBrace | CssToken::Exclamation => break,

                CssToken::Number(n) => {
                    self.advance();
                    values.push(Value::Number(n));
                }

                CssToken::Dimension { value, unit } => {
                    self.advance();
                    values.push(Value::Length(value, Unit::from_str(&unit)));
                }

                CssToken::Percentage(n) => {
                    self.advance();
                    values.push(Value::Percentage(n));
                }

                CssToken::Ident(name) => {
                    self.advance();
                    // Check for color
                    if let Some(color) = Color::from_str(&name) {
                        values.push(Value::Color(color));
                    } else {
                        values.push(Value::Keyword(name));
                    }
                }

                CssToken::Hash(hex) => {
                    self.advance();
                    if let Some(color) = Color::from_hex(&hex) {
                        values.push(Value::Color(color));
                    } else {
                        values.push(Value::Keyword(format!("#{}", hex)));
                    }
                }

                CssToken::String(s) => {
                    self.advance();
                    values.push(Value::String(s));
                }

                CssToken::Url(url) => {
                    self.advance();
                    values.push(Value::Url(url));
                }

                CssToken::Function(name) => {
                    self.advance();
                    let args = self.parse_function_args();

                    // Handle rgb/rgba specially
                    if name == "rgb" || name == "rgba" {
                        if let Some(color) = self.parse_rgb_args(&args) {
                            values.push(Value::Color(color));
                            continue;
                        }
                    }

                    values.push(Value::Function { name, args });
                }

                CssToken::Comma => {
                    self.advance();
                    // Skip comma in value lists
                }

                _ => {
                    self.advance();
                }
            }
        }

        if values.len() == 1 {
            values.pop().unwrap()
        } else if values.is_empty() {
            Value::Keyword(String::new())
        } else {
            Value::List(values)
        }
    }

    fn parse_function_args(&mut self) -> Vec<Value> {
        let mut args = Vec::new();
        let mut depth = 1;

        while !self.is_at_end() && depth > 0 {
            match self.peek() {
                CssToken::LeftParen => {
                    depth += 1;
                    self.advance();
                }
                CssToken::RightParen => {
                    depth -= 1;
                    if depth > 0 {
                        self.advance();
                    }
                }
                CssToken::Comma => {
                    self.advance();
                }
                _ => {
                    args.push(self.parse_single_value());
                }
            }
        }

        self.match_token(CssToken::RightParen);
        args
    }

    fn parse_single_value(&mut self) -> Value {
        match self.peek() {
            CssToken::Number(n) => {
                self.advance();
                Value::Number(n)
            }
            CssToken::Dimension { value, unit } => {
                self.advance();
                Value::Length(value, Unit::from_str(&unit))
            }
            CssToken::Percentage(n) => {
                self.advance();
                Value::Percentage(n)
            }
            CssToken::Ident(name) => {
                self.advance();
                Value::Keyword(name)
            }
            CssToken::String(s) => {
                self.advance();
                Value::String(s)
            }
            _ => {
                self.advance();
                Value::Keyword(String::new())
            }
        }
    }

    fn parse_rgb_args(&self, args: &[Value]) -> Option<Color> {
        let get_num = |v: &Value| -> Option<f64> {
            match v {
                Value::Number(n) => Some(*n),
                Value::Percentage(p) => Some(p * 2.55),
                _ => None,
            }
        };

        let r = get_num(args.get(0)?)?;
        let g = get_num(args.get(1)?)?;
        let b = get_num(args.get(2)?)?;
        let a = args.get(3).and_then(get_num).unwrap_or(255.0) / 255.0;

        Some(Color::from_rgb(r, g, b, a))
    }

    fn check_important(&mut self) -> bool {
        if self.check(CssToken::Exclamation) {
            self.advance();
            if let CssToken::Ident(name) = self.peek() {
                if name.to_lowercase() == "important" {
                    self.advance();
                    return true;
                }
            }
        }
        false
    }

    // Helper methods

    fn peek(&self) -> CssToken {
        self.tokens.get(self.current).cloned().unwrap_or(CssToken::Eof)
    }

    fn peek_next(&self) -> Option<CssToken> {
        self.tokens.get(self.current + 1).cloned()
    }

    fn advance(&mut self) -> CssToken {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1).cloned().unwrap_or(CssToken::Eof)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek(), CssToken::Eof)
    }

    fn check(&self, token: CssToken) -> bool {
        std::mem::discriminant(&self.peek()) == std::mem::discriminant(&token)
    }

    fn match_token(&mut self, token: CssToken) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }
}

/// Parse CSS string into stylesheet
pub fn parse_css(css: &str) -> Stylesheet {
    let mut parser = CssParser::new(css);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rule() {
        let css = "div { color: red; }";
        let sheet = parse_css(css);

        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "color");
    }

    #[test]
    fn test_multiple_selectors() {
        let css = ".foo, #bar, p { margin: 10px; }";
        let sheet = parse_css(css);

        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors.len(), 3);
    }

    #[test]
    fn test_hex_colors() {
        let css = "div { color: #ff0000; background: #0f0; }";
        let sheet = parse_css(css);

        if let Value::Color(c) = &sheet.rules[0].declarations[0].value {
            assert_eq!(c.r, 255);
            assert_eq!(c.g, 0);
            assert_eq!(c.b, 0);
        }
    }

    #[test]
    fn test_dimensions() {
        let css = "div { width: 100px; height: 50%; margin: 1em; }";
        let sheet = parse_css(css);

        assert_eq!(sheet.rules[0].declarations.len(), 3);
    }

    #[test]
    fn test_specificity() {
        let css = "#id .class div { color: red; }";
        let sheet = parse_css(css);

        let spec = &sheet.rules[0].selectors[0].specificity;
        assert_eq!(spec.a, 1); // ID
        assert_eq!(spec.b, 1); // class
        assert_eq!(spec.c, 1); // type
    }
}
