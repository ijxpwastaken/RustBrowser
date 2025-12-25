//! Style Engine
//!
//! This crate handles CSS selector matching, style cascade, and computed style resolution.

use css_parser::{
    Stylesheet, Rule, Selector, SelectorPart, Specificity, Declaration,
    Value, Unit, Color, Combinator, AttributeOperator, parse_css,
};
use dom::{Node, NodeRef, Element, Document};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Display mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Display {
    #[default]
    Block,
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    None,
    Table,
    TableRow,
    TableCell,
    TableCaption,
    ListItem,
}

/// Position mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// Flex direction
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Flex wrap
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Justify content (main axis alignment)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Align items (cross axis alignment)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlignItems {
    #[default]
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

/// Align self (individual item alignment)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlignSelf {
    #[default]
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

/// Overflow mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
    Justify,
}

/// Float mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Float {
    #[default]
    None,
    Left,
    Right,
}

/// Box sizing mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BoxSizing {
    #[default]
    ContentBox,
    BorderBox,
}

/// Length value (resolved to pixels or auto)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f64),
    Auto,
    Percent(f64),
}

impl Default for Length {
    fn default() -> Self {
        Length::Auto
    }
}

impl Length {
    pub fn to_px(&self, reference: f64) -> f64 {
        match self {
            Length::Px(px) => *px,
            Length::Auto => 0.0,
            Length::Percent(p) => reference * p / 100.0,
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Length::Auto)
    }
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
    Numeric(u32),
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Normal
    }
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// Text decoration
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    LineThrough,
    Overline,
}

/// Border style
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
}

/// Edge sizes for margins, padding, borders
#[derive(Debug, Default, Clone, Copy)]
pub struct EdgeSizes {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Color value
#[derive(Debug, Clone, Copy, Default)]
pub struct StyleColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl StyleColor {
    pub const TRANSPARENT: StyleColor = StyleColor { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: StyleColor = StyleColor { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: StyleColor = StyleColor { r: 255, g: 255, b: 255, a: 255 };

    pub fn from_css_color(color: &Color) -> Self {
        StyleColor { r: color.r, g: color.g, b: color.b, a: color.a }
    }

    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn to_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Computed style for an element
#[derive(Debug, Clone, Default)]
pub struct ComputedStyle {
    pub display: Display,
    pub position: Position,
    pub float: Float,
    pub box_sizing: BoxSizing,

    // Flexbox
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_self: AlignSelf,
    pub flex_grow: f64,
    pub flex_shrink: f64,
    pub flex_basis: Length,
    pub gap: f64,

    // Sizing
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub min_width: Option<f64>,
    pub max_width: Option<f64>,
    pub min_height: Option<f64>,
    pub max_height: Option<f64>,

    // Box model
    pub margin: EdgeSizes,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,

    // Border styles
    pub border_style: BorderStyle,
    pub border_color: StyleColor,
    pub border_radius: f64,

    // Colors
    pub color: [u8; 4],
    pub background_color: [u8; 4],
    pub background_image: Option<String>,

    // Text
    pub font_size: f64,
    pub font_family: String,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub line_height: f64,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub letter_spacing: f64,

    // Other
    pub overflow: Overflow,
    pub opacity: f64,
    pub z_index: Option<i32>,
    pub visibility: bool,
    pub cursor: String,
    pub list_style_type: String,
}

impl ComputedStyle {
    pub fn new() -> Self {
        ComputedStyle {
            display: Display::Block,
            position: Position::Static,
            font_size: 16.0,
            font_family: "Arial".to_string(),
            line_height: 1.2,
            color: [0, 0, 0, 255],
            background_color: [0, 0, 0, 0],
            opacity: 1.0,
            visibility: true,
            flex_shrink: 1.0,
            ..Default::default()
        }
    }

    /// Apply default styles based on HTML tag
    pub fn apply_default_for_tag(&mut self, tag: &str) {
        match tag.to_lowercase().as_str() {
            "div" | "section" | "article" | "header" | "footer" | "main" | "nav" | "aside" => {
                self.display = Display::Block;
            }
            "span" | "a" | "u" | "small" => {
                self.display = Display::Inline;
            }
            "h1" => {
                self.display = Display::Block;
                self.font_size = 32.0;
                self.font_weight = FontWeight::Bold;
                self.margin.top = 21.44;
                self.margin.bottom = 21.44;
            }
            "h2" => {
                self.display = Display::Block;
                self.font_size = 24.0;
                self.font_weight = FontWeight::Bold;
                self.margin.top = 19.92;
                self.margin.bottom = 19.92;
            }
            "h3" => {
                self.display = Display::Block;
                self.font_size = 18.72;
                self.font_weight = FontWeight::Bold;
                self.margin.top = 18.72;
                self.margin.bottom = 18.72;
            }
            "h4" | "h5" | "h6" => {
                self.display = Display::Block;
                self.font_size = 16.0;
                self.font_weight = FontWeight::Bold;
                self.margin.top = 21.28;
                self.margin.bottom = 21.28;
            }
            "p" => {
                self.display = Display::Block;
                self.margin.top = 16.0;
                self.margin.bottom = 16.0;
            }
            "ul" | "ol" => {
                self.display = Display::Block;
                self.margin.top = 16.0;
                self.margin.bottom = 16.0;
                self.padding.left = 40.0;
            }
            "li" => {
                self.display = Display::ListItem;
            }
            "img" => {
                self.display = Display::InlineBlock;
            }
            "button" | "input" | "select" | "textarea" => {
                self.display = Display::InlineBlock;
            }
            "table" => {
                self.display = Display::Table;
            }
            "tr" => {
                self.display = Display::TableRow;
            }
            "td" | "th" => {
                self.display = Display::TableCell;
            }
            "pre" => {
                self.display = Display::Block;
                self.font_family = "monospace".to_string();
            }
            "code" => {
                self.display = Display::Inline;
                self.font_family = "monospace".to_string();
            }
            "strong" | "b" => {
                self.display = Display::Inline;
                self.font_weight = FontWeight::Bold;
            }
            "em" | "i" => {
                self.display = Display::Inline;
                self.font_style = FontStyle::Italic;
            }
            "script" | "style" | "head" | "meta" | "link" | "title" => {
                self.display = Display::None;
            }
            _ => {}
        }
    }

    /// Apply a CSS declaration
    pub fn apply_declaration(&mut self, decl: &Declaration, parent_font_size: f64) {
        match decl.property.as_str() {
            "display" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.display = match kw.as_str() {
                        "block" => Display::Block,
                        "inline" => Display::Inline,
                        "inline-block" => Display::InlineBlock,
                        "flex" => Display::Flex,
                        "inline-flex" => Display::InlineFlex,
                        "grid" => Display::Grid,
                        "none" => Display::None,
                        _ => self.display,
                    };
                }
            }

            "position" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.position = match kw.as_str() {
                        "static" => Position::Static,
                        "relative" => Position::Relative,
                        "absolute" => Position::Absolute,
                        "fixed" => Position::Fixed,
                        "sticky" => Position::Sticky,
                        _ => self.position,
                    };
                }
            }

            "flex-direction" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.flex_direction = match kw.as_str() {
                        "row" => FlexDirection::Row,
                        "row-reverse" => FlexDirection::RowReverse,
                        "column" => FlexDirection::Column,
                        "column-reverse" => FlexDirection::ColumnReverse,
                        _ => self.flex_direction,
                    };
                }
            }

            "justify-content" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.justify_content = match kw.as_str() {
                        "flex-start" | "start" => JustifyContent::FlexStart,
                        "flex-end" | "end" => JustifyContent::FlexEnd,
                        "center" => JustifyContent::Center,
                        "space-between" => JustifyContent::SpaceBetween,
                        "space-around" => JustifyContent::SpaceAround,
                        "space-evenly" => JustifyContent::SpaceEvenly,
                        _ => self.justify_content,
                    };
                }
            }

            "align-items" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.align_items = match kw.as_str() {
                        "flex-start" | "start" => AlignItems::FlexStart,
                        "flex-end" | "end" => AlignItems::FlexEnd,
                        "center" => AlignItems::Center,
                        "stretch" => AlignItems::Stretch,
                        "baseline" => AlignItems::Baseline,
                        _ => self.align_items,
                    };
                }
            }

            "flex-grow" => {
                if let Value::Number(n) = &decl.value {
                    self.flex_grow = *n;
                }
            }

            "flex-shrink" => {
                if let Value::Number(n) = &decl.value {
                    self.flex_shrink = *n;
                }
            }

            "gap" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.gap = px;
                }
            }

            "width" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.width = Some(px);
                }
            }

            "height" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.height = Some(px);
                }
            }

            "min-width" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.min_width = Some(px);
                }
            }

            "max-width" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.max_width = Some(px);
                }
            }

            "margin" => {
                apply_edge_shorthand(&decl.value, parent_font_size, &mut self.margin);
            }
            "margin-top" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.margin.top = px;
                }
            }
            "margin-right" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.margin.right = px;
                }
            }
            "margin-bottom" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.margin.bottom = px;
                }
            }
            "margin-left" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.margin.left = px;
                }
            }

            "padding" => {
                apply_edge_shorthand(&decl.value, parent_font_size, &mut self.padding);
            }
            "padding-top" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.padding.top = px;
                }
            }
            "padding-right" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.padding.right = px;
                }
            }
            "padding-bottom" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.padding.bottom = px;
                }
            }
            "padding-left" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.padding.left = px;
                }
            }

            "border" | "border-width" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.border.top = px;
                    self.border.right = px;
                    self.border.bottom = px;
                    self.border.left = px;
                    self.border_style = BorderStyle::Solid;
                }
            }

            "border-radius" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.border_radius = px;
                }
            }

            "border-color" => {
                if let Some(color) = extract_color(&decl.value) {
                    self.border_color = color;
                }
            }

            "color" => {
                if let Some(color) = extract_color(&decl.value) {
                    self.color = color.to_array();
                }
            }

            "background-color" | "background" => {
                if let Some(color) = extract_color(&decl.value) {
                    self.background_color = color.to_array();
                }
            }

            "font-size" => {
                if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.font_size = px;
                }
            }

            "font-family" => {
                if let Value::String(s) = &decl.value {
                    self.font_family = s.clone();
                } else if let Value::Keyword(s) = &decl.value {
                    self.font_family = s.clone();
                }
            }

            "font-weight" => {
                self.font_weight = match &decl.value {
                    Value::Keyword(kw) => match kw.as_str() {
                        "normal" => FontWeight::Normal,
                        "bold" => FontWeight::Bold,
                        _ => self.font_weight,
                    },
                    Value::Number(n) => FontWeight::Numeric(*n as u32),
                    _ => self.font_weight,
                };
            }

            "line-height" => {
                if let Value::Number(n) = &decl.value {
                    self.line_height = *n;
                } else if let Some(px) = extract_px(&decl.value, parent_font_size) {
                    self.line_height = px / self.font_size;
                }
            }

            "text-align" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.text_align = match kw.as_str() {
                        "left" => TextAlign::Left,
                        "right" => TextAlign::Right,
                        "center" => TextAlign::Center,
                        "justify" => TextAlign::Justify,
                        _ => self.text_align,
                    };
                }
            }

            "text-decoration" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.text_decoration = match kw.as_str() {
                        "none" => TextDecoration::None,
                        "underline" => TextDecoration::Underline,
                        "line-through" => TextDecoration::LineThrough,
                        _ => self.text_decoration,
                    };
                }
            }

            "overflow" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.overflow = match kw.as_str() {
                        "visible" => Overflow::Visible,
                        "hidden" => Overflow::Hidden,
                        "scroll" => Overflow::Scroll,
                        "auto" => Overflow::Auto,
                        _ => self.overflow,
                    };
                }
            }

            "opacity" => {
                if let Value::Number(n) = &decl.value {
                    self.opacity = n.clamp(0.0, 1.0);
                }
            }

            "z-index" => {
                if let Value::Number(n) = &decl.value {
                    self.z_index = Some(*n as i32);
                }
            }

            "visibility" => {
                if let Value::Keyword(kw) = &decl.value {
                    self.visibility = kw != "hidden";
                }
            }

            _ => {}
        }
    }
}

fn extract_px(value: &Value, parent_font_size: f64) -> Option<f64> {
    match value {
        Value::Number(n) => Some(*n),
        Value::Length(n, unit) => {
            let px = match unit {
                Unit::Px => *n,
                Unit::Em => *n * parent_font_size,
                Unit::Rem => *n * 16.0,
                Unit::Percent => *n * parent_font_size / 100.0,
                Unit::Pt => *n * 1.333,
                Unit::Vh => *n * 8.0,
                Unit::Vw => *n * 10.0,
                _ => *n,
            };
            Some(px)
        }
        Value::Percentage(n) => Some(*n * parent_font_size / 100.0),
        _ => None,
    }
}

fn extract_color(value: &Value) -> Option<StyleColor> {
    match value {
        Value::Color(c) => Some(StyleColor::from_css_color(c)),
        Value::Keyword(kw) => Color::from_str(kw).map(|c| StyleColor::from_css_color(&c)),
        _ => None,
    }
}

fn apply_edge_shorthand(value: &Value, parent_font_size: f64, edges: &mut EdgeSizes) {
    match value {
        Value::List(values) => {
            let px_values: Vec<f64> = values
                .iter()
                .filter_map(|v| extract_px(v, parent_font_size))
                .collect();

            match px_values.len() {
                1 => {
                    edges.top = px_values[0];
                    edges.right = px_values[0];
                    edges.bottom = px_values[0];
                    edges.left = px_values[0];
                }
                2 => {
                    edges.top = px_values[0];
                    edges.bottom = px_values[0];
                    edges.right = px_values[1];
                    edges.left = px_values[1];
                }
                3 => {
                    edges.top = px_values[0];
                    edges.right = px_values[1];
                    edges.left = px_values[1];
                    edges.bottom = px_values[2];
                }
                4 => {
                    edges.top = px_values[0];
                    edges.right = px_values[1];
                    edges.bottom = px_values[2];
                    edges.left = px_values[3];
                }
                _ => {}
            }
        }
        _ => {
            if let Some(px) = extract_px(value, parent_font_size) {
                edges.top = px;
                edges.right = px;
                edges.bottom = px;
                edges.left = px;
            }
        }
    }
}

/// Check if a selector matches an element
pub fn selector_matches(selector: &Selector, element: &Element, ancestors: &[&Element]) -> bool {
    let mut parts = selector.parts.iter().rev().peekable();
    let mut current_element = Some(element);
    let mut ancestor_idx = 0;

    while let Some(part) = parts.next() {
        match part {
            SelectorPart::Type(tag) => {
                if let Some(elem) = current_element {
                    if elem.tag_name.to_lowercase() != tag.to_lowercase() {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            SelectorPart::Class(class) => {
                if let Some(elem) = current_element {
                    if !elem.classes().contains(&class.as_str()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            SelectorPart::Id(id) => {
                if let Some(elem) = current_element {
                    if elem.id_attr().map(|s| s.as_str()) != Some(id) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            SelectorPart::Universal => {}
            SelectorPart::Attribute { name, operator, value } => {
                if let Some(elem) = current_element {
                    let attr_val = elem.get_attribute(name);
                    let matches = match (operator, value, attr_val) {
                        (None, _, Some(_)) => true,
                        (Some(AttributeOperator::Equals), Some(expected), Some(actual)) => actual == expected,
                        (Some(AttributeOperator::PrefixMatch), Some(expected), Some(actual)) => actual.starts_with(expected),
                        (Some(AttributeOperator::SuffixMatch), Some(expected), Some(actual)) => actual.ends_with(expected),
                        (Some(AttributeOperator::SubstringMatch), Some(expected), Some(actual)) => actual.contains(expected),
                        _ => false,
                    };
                    if !matches {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            SelectorPart::PseudoClass(pseudo) => {
                match pseudo.as_str() {
                    "hover" | "active" | "focus" | "visited" => return false,
                    _ => {}
                }
            }
            SelectorPart::PseudoElement(_) => {}
            SelectorPart::Combinator(comb) => {
                match comb {
                    Combinator::Descendant | Combinator::Child => {
                        if ancestor_idx < ancestors.len() {
                            current_element = Some(ancestors[ancestor_idx]);
                            ancestor_idx += 1;
                        } else {
                            return false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    true
}

/// Style tree node
#[derive(Debug)]
pub struct StyledNode {
    pub node: NodeRef,
    pub style: ComputedStyle,
    pub children: Vec<Rc<RefCell<StyledNode>>>,
}

impl StyledNode {
    pub fn new(node: NodeRef, style: ComputedStyle) -> Self {
        StyledNode { node, style, children: Vec::new() }
    }
}

/// Compute styles for a DOM tree
pub fn compute_styles(document: &Document, stylesheet: &Stylesheet) -> Option<Rc<RefCell<StyledNode>>> {
    let root = document.document_element.as_ref()?;

    fn compute_node_style(
        node: &NodeRef,
        stylesheet: &Stylesheet,
        parent_style: Option<&ComputedStyle>,
        ancestors: &[&Element],
    ) -> Rc<RefCell<StyledNode>> {
        let node_borrow = match node.read() {
            Ok(guard) => guard,
            Err(_) => {
                // If we can't read the node, return a default styled node
                return Rc::new(RefCell::new(StyledNode {
                    node: node.clone(),
                    style: ComputedStyle::new(),
                    children: Vec::new(),
                }));
            }
        };

        let mut style = if let Some(parent) = parent_style {
            let mut s = ComputedStyle::new();
            s.color = parent.color;
            s.font_size = parent.font_size;
            s.font_family = parent.font_family.clone();
            s.font_weight = parent.font_weight;
            s.line_height = parent.line_height;
            s.text_align = parent.text_align;
            s.visibility = parent.visibility;
            s
        } else {
            ComputedStyle::new()
        };

        let parent_font_size = parent_style.map(|s| s.font_size).unwrap_or(16.0);

        if let Some(element) = node_borrow.as_element() {
            style.apply_default_for_tag(&element.tag_name);

            let mut matching_rules: Vec<(&Rule, &Selector)> = Vec::new();
            for rule in &stylesheet.rules {
                for selector in &rule.selectors {
                    if selector_matches(selector, element, ancestors) {
                        matching_rules.push((rule, selector));
                    }
                }
            }

            matching_rules.sort_by(|a, b| a.1.specificity.cmp(&b.1.specificity));

            // Phase 1: Normal declarations
            for (rule, _) in &matching_rules {
                for decl in &rule.declarations {
                    if !decl.important {
                        style.apply_declaration(decl, parent_font_size);
                    }
                }
            }

            // Phase 2: Important declarations
            for (rule, _) in &matching_rules {
                for decl in &rule.declarations {
                    if decl.important {
                        style.apply_declaration(decl, parent_font_size);
                    }
                }
            }

            if let Some(inline_style) = element.get_attribute("style") {
                let inline_css = format!("* {{ {} }}", inline_style);
                let inline_sheet = parse_css(&inline_css);
                for rule in &inline_sheet.rules {
                    for decl in &rule.declarations {
                        style.apply_declaration(decl, parent_font_size);
                    }
                }
            }
        }

        let styled_node = Rc::new(RefCell::new(StyledNode::new(node.clone(), style.clone())));

        // Get children from element if this is an element node
        if let Some(element) = node_borrow.as_element() {
            for child in &element.children {
                let mut new_ancestors = ancestors.to_vec();
                new_ancestors.insert(0, element);
                let child_styled = compute_node_style(child, stylesheet, Some(&style), &new_ancestors);
                styled_node.borrow_mut().children.push(child_styled);
            }
        }

        styled_node
    }

    Some(compute_node_style(root, stylesheet, None, &[]))
}
