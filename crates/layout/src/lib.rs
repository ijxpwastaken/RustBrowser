//! Layout Engine
//!
//! This crate handles layout computation including the box model,
//! block/inline layout, flexbox, and positioning.

use style::{ComputedStyle, Display, Position, FlexDirection, JustifyContent, AlignItems, FontWeight};

/// A layout box in the render tree
#[derive(Debug)]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    pub box_type: BoxType,
    pub children: Vec<LayoutBox>,
    pub style: ComputedStyle,
    pub text: Option<String>,
    pub image_data: Option<ImageData>,
}

/// Image data for rendering
#[derive(Debug, Clone)]
pub struct ImageData {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Box dimensions
#[derive(Debug, Default, Clone, Copy)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

/// Rectangle
#[derive(Debug, Default, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Edge sizes (for margin, padding, border)
#[derive(Debug, Default, Clone, Copy)]
pub struct EdgeSizes {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Type of layout box
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    InlineBlock,
    Flex,
    FlexItem,
    Anonymous,
    Text,
    Image,
    None,
}

impl Dimensions {
    pub fn padding_box(&self) -> Rect {
        self.content.expanded_by(&self.padding)
    }

    pub fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(&self.border)
    }

    pub fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(&self.margin)
    }

    pub fn total_height(&self) -> f64 {
        self.content.height + self.padding.top + self.padding.bottom +
        self.border.top + self.border.bottom + self.margin.top + self.margin.bottom
    }

    pub fn total_width(&self) -> f64 {
        self.content.width + self.padding.left + self.padding.right +
        self.border.left + self.border.right + self.margin.left + self.margin.right
    }
}

impl Rect {
    pub fn expanded_by(&self, edge: &EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
}

impl LayoutBox {
    pub fn new(box_type: BoxType, style: ComputedStyle) -> Self {
        LayoutBox {
            dimensions: Dimensions::default(),
            box_type,
            children: Vec::new(),
            style,
            text: None,
            image_data: None,
        }
    }

    /// Calculate layout for this box and its children
    pub fn layout(&mut self, containing_block: &Dimensions) {
        match self.box_type {
            BoxType::Block | BoxType::InlineBlock => self.layout_block(containing_block),
            BoxType::Inline | BoxType::Text => self.layout_inline(containing_block),
            BoxType::Flex => self.layout_flex(containing_block),
            BoxType::FlexItem => self.layout_flex_item(containing_block),
            BoxType::Image => self.layout_image(containing_block),
            BoxType::Anonymous => self.layout_anonymous(containing_block),
            BoxType::None => {}
        }
    }

    fn layout_block(&mut self, containing_block: &Dimensions) {
        // Calculate width
        self.calculate_block_width(containing_block);

        // Calculate position
        self.calculate_block_position(containing_block);

        // Layout children
        self.layout_block_children();

        // Calculate height
        self.calculate_block_height();
    }

    fn calculate_block_width(&mut self, containing_block: &Dimensions) {
        let s = &self.style;

        // Margins, padding, borders
        self.dimensions.margin.left = s.margin.left;
        self.dimensions.margin.right = s.margin.right;
        self.dimensions.margin.top = s.margin.top;
        self.dimensions.margin.bottom = s.margin.bottom;

        self.dimensions.padding.left = s.padding.left;
        self.dimensions.padding.right = s.padding.right;
        self.dimensions.padding.top = s.padding.top;
        self.dimensions.padding.bottom = s.padding.bottom;

        self.dimensions.border.left = s.border.left;
        self.dimensions.border.right = s.border.right;
        self.dimensions.border.top = s.border.top;
        self.dimensions.border.bottom = s.border.bottom;

        let total_extra = self.dimensions.margin.left + self.dimensions.margin.right +
                         self.dimensions.padding.left + self.dimensions.padding.right +
                         self.dimensions.border.left + self.dimensions.border.right;

        // Calculate width
        if let Some(w) = s.width {
            self.dimensions.content.width = w;
        } else {
            // Auto width - fill containing block
            self.dimensions.content.width = (containing_block.content.width - total_extra).max(0.0);
        }

        // Apply min/max width
        if let Some(min_w) = s.min_width {
            if self.dimensions.content.width < min_w {
                self.dimensions.content.width = min_w;
            }
        }
        if let Some(max_w) = s.max_width {
            if self.dimensions.content.width > max_w {
                self.dimensions.content.width = max_w;
            }
        }
    }

    fn calculate_block_position(&mut self, containing_block: &Dimensions) {
        self.dimensions.content.x = containing_block.content.x +
                                    self.dimensions.margin.left +
                                    self.dimensions.padding.left +
                                    self.dimensions.border.left;

        self.dimensions.content.y = containing_block.content.y +
                                    containing_block.content.height +
                                    self.dimensions.margin.top +
                                    self.dimensions.padding.top +
                                    self.dimensions.border.top;
    }

    fn layout_block_children(&mut self) {
        let mut d = self.dimensions;

        for child in &mut self.children {
            child.layout(&d);
            // Stack children vertically
            d.content.height += child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        // If explicit height is set, use it
        if let Some(h) = self.style.height {
            self.dimensions.content.height = h;
        }
        // Otherwise height is sum of children (calculated in layout_block_children)
    }

    fn layout_inline(&mut self, containing_block: &Dimensions) {
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y;

        // For text, calculate width based on content
        if let Some(ref text) = self.text {
            let char_width = self.style.font_size * 0.6;
            self.dimensions.content.width = text.len() as f64 * char_width;
            self.dimensions.content.height = self.style.font_size * self.style.line_height;
        }
    }

    fn layout_flex(&mut self, containing_block: &Dimensions) {
        // Calculate container dimensions first
        self.calculate_block_width(containing_block);
        self.calculate_block_position(containing_block);

        let s = &self.style;
        let is_row = matches!(s.flex_direction, FlexDirection::Row | FlexDirection::RowReverse);
        let is_reverse = matches!(s.flex_direction, FlexDirection::RowReverse | FlexDirection::ColumnReverse);

        // First pass: calculate base sizes
        let mut total_flex_grow = 0.0;
        let mut total_base_size = 0.0;
        let gap = s.gap;

        for child in &mut self.children {
            child.dimensions.margin = EdgeSizes {
                top: child.style.margin.top,
                right: child.style.margin.right,
                bottom: child.style.margin.bottom,
                left: child.style.margin.left,
            };
            child.dimensions.padding = EdgeSizes {
                top: child.style.padding.top,
                right: child.style.padding.right,
                bottom: child.style.padding.bottom,
                left: child.style.padding.left,
            };
            child.dimensions.border = EdgeSizes {
                top: child.style.border.top,
                right: child.style.border.right,
                bottom: child.style.border.bottom,
                left: child.style.border.left,
            };

            // Base size from width/height or content
            let base_size = if is_row {
                child.style.width.unwrap_or(100.0)
            } else {
                child.style.height.unwrap_or(50.0)
            };

            child.dimensions.content.width = if is_row { base_size } else { self.dimensions.content.width };
            child.dimensions.content.height = if is_row { 50.0 } else { base_size };

            total_base_size += if is_row {
                child.dimensions.total_width()
            } else {
                child.dimensions.total_height()
            };
            total_flex_grow += child.style.flex_grow;
        }

        // Add gaps
        if !self.children.is_empty() {
            total_base_size += gap * (self.children.len() - 1) as f64;
        }

        // Calculate available space
        let available_space = if is_row {
            self.dimensions.content.width
        } else {
            self.style.height.unwrap_or(self.dimensions.content.height)
        };

        let free_space = (available_space - total_base_size).max(0.0);

        // Second pass: distribute space and position
        let mut pos = if is_row { 0.0 } else { 0.0 };

        // Justify content positioning
        let (start_offset, spacing) = match s.justify_content {
            JustifyContent::FlexStart => (0.0, 0.0),
            JustifyContent::FlexEnd => (free_space, 0.0),
            JustifyContent::Center => (free_space / 2.0, 0.0),
            JustifyContent::SpaceBetween => {
                let spacing = if self.children.len() > 1 {
                    free_space / (self.children.len() - 1) as f64
                } else {
                    0.0
                };
                (0.0, spacing)
            }
            JustifyContent::SpaceAround => {
                let spacing = free_space / self.children.len() as f64;
                (spacing / 2.0, spacing)
            }
            JustifyContent::SpaceEvenly => {
                let spacing = free_space / (self.children.len() + 1) as f64;
                (spacing, spacing)
            }
        };

        pos += start_offset;

        for (i, child) in self.children.iter_mut().enumerate() {
            // Flex grow distribution
            if total_flex_grow > 0.0 && child.style.flex_grow > 0.0 {
                let extra = free_space * (child.style.flex_grow / total_flex_grow);
                if is_row {
                    child.dimensions.content.width += extra;
                } else {
                    child.dimensions.content.height += extra;
                }
            }

            // Position
            if is_row {
                child.dimensions.content.x = self.dimensions.content.x + pos +
                    child.dimensions.margin.left + child.dimensions.padding.left + child.dimensions.border.left;

                // Cross axis alignment
                let cross_size = child.dimensions.total_height();
                let cross_space = self.dimensions.content.height - cross_size;

                child.dimensions.content.y = self.dimensions.content.y +
                    child.dimensions.margin.top + child.dimensions.padding.top + child.dimensions.border.top +
                    match s.align_items {
                        AlignItems::FlexStart => 0.0,
                        AlignItems::FlexEnd => cross_space,
                        AlignItems::Center => cross_space / 2.0,
                        AlignItems::Stretch => {
                            child.dimensions.content.height = self.dimensions.content.height -
                                child.dimensions.margin.top - child.dimensions.margin.bottom -
                                child.dimensions.padding.top - child.dimensions.padding.bottom -
                                child.dimensions.border.top - child.dimensions.border.bottom;
                            0.0
                        }
                        AlignItems::Baseline => 0.0,
                    };

                pos += child.dimensions.total_width() + gap + spacing;
            } else {
                child.dimensions.content.y = self.dimensions.content.y + pos +
                    child.dimensions.margin.top + child.dimensions.padding.top + child.dimensions.border.top;

                // Cross axis alignment
                let cross_size = child.dimensions.total_width();
                let cross_space = self.dimensions.content.width - cross_size;

                child.dimensions.content.x = self.dimensions.content.x +
                    child.dimensions.margin.left + child.dimensions.padding.left + child.dimensions.border.left +
                    match s.align_items {
                        AlignItems::FlexStart => 0.0,
                        AlignItems::FlexEnd => cross_space,
                        AlignItems::Center => cross_space / 2.0,
                        AlignItems::Stretch => {
                            child.dimensions.content.width = self.dimensions.content.width -
                                child.dimensions.margin.left - child.dimensions.margin.right -
                                child.dimensions.padding.left - child.dimensions.padding.right -
                                child.dimensions.border.left - child.dimensions.border.right;
                            0.0
                        }
                        AlignItems::Baseline => 0.0,
                    };

                pos += child.dimensions.total_height() + gap + spacing;
            }

            // Recursively layout children
            let child_containing = child.dimensions;
            for grandchild in &mut child.children {
                grandchild.layout(&child_containing);
            }
        }

        // Calculate container height if not set
        if self.style.height.is_none() {
            if is_row {
                let max_height = self.children.iter()
                    .map(|c| c.dimensions.total_height())
                    .fold(0.0, f64::max);
                self.dimensions.content.height = max_height;
            } else {
                self.dimensions.content.height = pos;
            }
        }
    }

    fn layout_flex_item(&mut self, containing_block: &Dimensions) {
        self.layout_block(containing_block);
    }

    fn layout_image(&mut self, containing_block: &Dimensions) {
        self.dimensions.margin = EdgeSizes {
            top: self.style.margin.top,
            right: self.style.margin.right,
            bottom: self.style.margin.bottom,
            left: self.style.margin.left,
        };

        self.dimensions.content.x = containing_block.content.x + self.dimensions.margin.left;
        self.dimensions.content.y = containing_block.content.y + containing_block.content.height +
                                    self.dimensions.margin.top;

        // Use explicit size or image size
        if let Some(ref img) = self.image_data {
            self.dimensions.content.width = self.style.width.unwrap_or(img.width as f64);
            self.dimensions.content.height = self.style.height.unwrap_or(img.height as f64);
        } else {
            self.dimensions.content.width = self.style.width.unwrap_or(100.0);
            self.dimensions.content.height = self.style.height.unwrap_or(100.0);
        }
    }

    fn layout_anonymous(&mut self, containing_block: &Dimensions) {
        self.layout_block(containing_block);
    }
}

/// Layout context for building the layout tree
pub struct LayoutContext {
    pub viewport_width: f64,
    pub viewport_height: f64,
}

impl LayoutContext {
    pub fn new(width: f64, height: f64) -> Self {
        LayoutContext {
            viewport_width: width,
            viewport_height: height,
        }
    }

    pub fn initial_containing_block(&self) -> Dimensions {
        Dimensions {
            content: Rect {
                x: 0.0,
                y: 0.0,
                width: self.viewport_width,
                height: 0.0,
            },
            ..Default::default()
        }
    }
}

/// Build a layout tree from a styled tree
pub fn build_layout_tree(styled_root: &std::rc::Rc<std::cell::RefCell<style::StyledNode>>) -> Option<LayoutBox> {
    build_layout_box(styled_root)
}

fn build_layout_box(styled_node: &std::rc::Rc<std::cell::RefCell<style::StyledNode>>) -> Option<LayoutBox> {
    let styled_node_ref = styled_node.borrow();
    let style = styled_node_ref.style.clone();

    // Skip nodes with display: none
    if style.display == Display::None {
        return None;
    }

    let box_type = match style.display {
        Display::Block => BoxType::Block,
        Display::Inline => BoxType::Inline,
        Display::InlineBlock => BoxType::InlineBlock,
        Display::Flex | Display::InlineFlex => BoxType::Flex,
        Display::None => return None,
        _ => BoxType::Block,
    };

    let mut layout_box = LayoutBox::new(box_type, style);

    // Check if this is a text node
    if let Ok(node_guard) = styled_node_ref.node.read() {
        if let Some(text) = node_guard.as_text() {
            let trimmed = text.content.trim();
            if !trimmed.is_empty() {
                layout_box.text = Some(trimmed.to_string());
                layout_box.box_type = BoxType::Text;
            }
        }
    }

    // Build children
    for child in &styled_node_ref.children {
        if let Some(child_box) = build_layout_box(child) {
            layout_box.children.push(child_box);
        }
    }

    Some(layout_box)
}
