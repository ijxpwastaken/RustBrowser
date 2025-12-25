//! Rendering Engine
//!
//! This crate handles painting the layout tree to a pixel buffer.

pub use layout;
use layout::{LayoutBox, BoxType, Rect as LayoutRect};
use style::{FontWeight, BorderStyle, TextDecoration};

/// A display command for painting
#[derive(Debug, Clone)]
pub enum DisplayCommand {
    /// Fill a rectangle with a solid color
    SolidColor(Color, LayoutRect),
    /// Draw text
    Text {
        text: String,
        x: f64,
        y: f64,
        color: Color,
        font_size: f64,
        font_weight: FontWeight,
        underline: bool,
    },
    /// Draw an image
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
        rect: LayoutRect,
    },
    /// Draw a border
    Border {
        rect: LayoutRect,
        color: Color,
        width: EdgeWidths,
        style: BorderStyleSet,
        radius: f64,
    },
    /// Draw a rounded rectangle
    RoundedRect {
        rect: LayoutRect,
        color: Color,
        radius: f64,
    },
    /// Draw a line
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
        width: f64,
    },
}

/// Border widths for each side
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeWidths {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Border styles for each side
#[derive(Debug, Clone, Copy, Default)]
pub struct BorderStyleSet {
    pub top: BorderStyle,
    pub right: BorderStyle,
    pub bottom: BorderStyle,
    pub left: BorderStyle,
}

/// RGBA Color
#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
    pub const LIGHT_GRAY: Color = Color { r: 211, g: 211, b: 211, a: 255 };

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn from_array(arr: [u8; 4]) -> Self {
        Color { r: arr[0], g: arr[1], b: arr[2], a: arr[3] }
    }

    /// Convert to u32 (0x00RRGGBB format for softbuffer)
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Convert to u32 with alpha (0xAARRGGBB format)
    pub fn to_argb_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Blend this color over another (alpha compositing)
    pub fn blend_over(&self, bg: &Color) -> Color {
        if self.a == 255 {
            return *self;
        }
        if self.a == 0 {
            return *bg;
        }

        let fg_a = self.a as f64 / 255.0;
        let bg_a = bg.a as f64 / 255.0;
        let out_a = fg_a + bg_a * (1.0 - fg_a);

        if out_a == 0.0 {
            return Color::TRANSPARENT;
        }

        let r = ((self.r as f64 * fg_a + bg.r as f64 * bg_a * (1.0 - fg_a)) / out_a) as u8;
        let g = ((self.g as f64 * fg_a + bg.g as f64 * bg_a * (1.0 - fg_a)) / out_a) as u8;
        let b = ((self.b as f64 * fg_a + bg.b as f64 * bg_a * (1.0 - fg_a)) / out_a) as u8;
        let a = (out_a * 255.0) as u8;

        Color { r, g, b, a }
    }
}

/// Display list - a list of display commands to execute
#[derive(Debug, Default)]
pub struct DisplayList {
    pub commands: Vec<DisplayCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        DisplayList { commands: Vec::new() }
    }

    pub fn push(&mut self, command: DisplayCommand) {
        self.commands.push(command);
    }

    /// Build display list from layout tree
    pub fn from_layout(root: &LayoutBox) -> Self {
        let mut list = DisplayList::new();
        render_layout_box(&mut list, root);
        list
    }
}

/// Render a layout box to display commands
fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);
    render_content(list, layout_box);

    // Render children
    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    let color = Color::from_array(layout_box.style.background_color);

    // Skip transparent backgrounds
    if color.a == 0 {
        return;
    }

    let rect = layout_box.dimensions.border_box();

    if layout_box.style.border_radius > 0.0 {
        list.push(DisplayCommand::RoundedRect {
            rect,
            color,
            radius: layout_box.style.border_radius,
        });
    } else {
        list.push(DisplayCommand::SolidColor(color, rect));
    }
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    let d = &layout_box.dimensions;
    let s = &layout_box.style;

    // Only render if there's any border
    let has_border = d.border.top > 0.0 || d.border.right > 0.0 ||
                     d.border.bottom > 0.0 || d.border.left > 0.0;

    if !has_border || matches!(s.border_style, BorderStyle::None) {
        return;
    }

    let border_color = Color {
        r: s.border_color.r,
        g: s.border_color.g,
        b: s.border_color.b,
        a: s.border_color.a,
    };

    let rect = d.border_box();

    list.push(DisplayCommand::Border {
        rect,
        color: border_color,
        width: EdgeWidths {
            top: d.border.top,
            right: d.border.right,
            bottom: d.border.bottom,
            left: d.border.left,
        },
        style: BorderStyleSet {
            top: s.border_style,
            right: s.border_style,
            bottom: s.border_style,
            left: s.border_style,
        },
        radius: s.border_radius,
    });
}

fn render_content(list: &mut DisplayList, layout_box: &LayoutBox) {
    // Render text
    if let Some(ref text) = layout_box.text {
        let color = Color::from_array(layout_box.style.color);
        let underline = matches!(layout_box.style.text_decoration, TextDecoration::Underline);

        list.push(DisplayCommand::Text {
            text: text.clone(),
            x: layout_box.dimensions.content.x,
            y: layout_box.dimensions.content.y,
            color,
            font_size: layout_box.style.font_size,
            font_weight: layout_box.style.font_weight,
            underline,
        });
    }

    // Render image
    if let Some(ref img) = layout_box.image_data {
        list.push(DisplayCommand::Image {
            data: img.data.clone(),
            width: img.width,
            height: img.height,
            rect: layout_box.dimensions.content,
        });
    }
}

/// Software renderer - renders display list to a pixel buffer
pub struct SoftwareRenderer {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u32>,
}

impl SoftwareRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let buffer = vec![0xFFFFFF; (width * height) as usize]; // White background
        SoftwareRenderer { width, height, buffer }
    }

    pub fn clear(&mut self, color: Color) {
        let c = color.to_u32();
        for pixel in &mut self.buffer {
            *pixel = c;
        }
    }

    pub fn render(&mut self, display_list: &DisplayList) {
        for command in &display_list.commands {
            match command {
                DisplayCommand::SolidColor(color, rect) => {
                    self.fill_rect(rect, color);
                }
                DisplayCommand::RoundedRect { rect, color, radius } => {
                    self.fill_rounded_rect(rect, color, *radius);
                }
                DisplayCommand::Border { rect, color, width, style: _, radius } => {
                    self.draw_border(rect, color, width, *radius);
                }
                DisplayCommand::Text { text, x, y, color, font_size, font_weight, underline } => {
                    self.draw_text(text, *x, *y, color, *font_size, *font_weight, *underline);
                }
                DisplayCommand::Image { data, width: img_w, height: img_h, rect } => {
                    self.draw_image(data, *img_w, *img_h, rect);
                }
                DisplayCommand::Line { x1, y1, x2, y2, color, width: _ } => {
                    self.draw_line(*x1, *y1, *x2, *y2, color);
                }
            }
        }
    }

    fn fill_rect(&mut self, rect: &LayoutRect, color: &Color) {
        let x0 = rect.x.max(0.0) as u32;
        let y0 = rect.y.max(0.0) as u32;
        let x1 = ((rect.x + rect.width) as u32).min(self.width);
        let y1 = ((rect.y + rect.height) as u32).min(self.height);

        let c = color.to_u32();

        for y in y0..y1 {
            for x in x0..x1 {
                let idx = (y * self.width + x) as usize;
                if idx < self.buffer.len() {
                    if color.a == 255 {
                        self.buffer[idx] = c;
                    } else if color.a > 0 {
                        // Alpha blend
                        let bg = self.buffer[idx];
                        let bg_color = Color {
                            r: ((bg >> 16) & 0xFF) as u8,
                            g: ((bg >> 8) & 0xFF) as u8,
                            b: (bg & 0xFF) as u8,
                            a: 255,
                        };
                        let blended = color.blend_over(&bg_color);
                        self.buffer[idx] = blended.to_u32();
                    }
                }
            }
        }
    }

    fn fill_rounded_rect(&mut self, rect: &LayoutRect, color: &Color, radius: f64) {
        let x0 = rect.x.max(0.0) as i32;
        let y0 = rect.y.max(0.0) as i32;
        let x1 = ((rect.x + rect.width) as i32).min(self.width as i32);
        let y1 = ((rect.y + rect.height) as i32).min(self.height as i32);
        let r = radius as f64;

        let c = color.to_u32();

        for y in y0..y1 {
            for x in x0..x1 {
                // Check if pixel is inside rounded rectangle
                let px = x as f64 - rect.x;
                let py = y as f64 - rect.y;

                let in_rect = if px < r && py < r {
                    // Top-left corner
                    let dx = r - px;
                    let dy = r - py;
                    dx * dx + dy * dy <= r * r
                } else if px > rect.width - r && py < r {
                    // Top-right corner
                    let dx = px - (rect.width - r);
                    let dy = r - py;
                    dx * dx + dy * dy <= r * r
                } else if px < r && py > rect.height - r {
                    // Bottom-left corner
                    let dx = r - px;
                    let dy = py - (rect.height - r);
                    dx * dx + dy * dy <= r * r
                } else if px > rect.width - r && py > rect.height - r {
                    // Bottom-right corner
                    let dx = px - (rect.width - r);
                    let dy = py - (rect.height - r);
                    dx * dx + dy * dy <= r * r
                } else {
                    true
                };

                if in_rect {
                    let idx = (y as u32 * self.width + x as u32) as usize;
                    if idx < self.buffer.len() {
                        self.buffer[idx] = c;
                    }
                }
            }
        }
    }

    fn draw_border(&mut self, rect: &LayoutRect, color: &Color, width: &EdgeWidths, _radius: f64) {
        // Top border
        if width.top > 0.0 {
            self.fill_rect(&LayoutRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: width.top,
            }, color);
        }

        // Bottom border
        if width.bottom > 0.0 {
            self.fill_rect(&LayoutRect {
                x: rect.x,
                y: rect.y + rect.height - width.bottom,
                width: rect.width,
                height: width.bottom,
            }, color);
        }

        // Left border
        if width.left > 0.0 {
            self.fill_rect(&LayoutRect {
                x: rect.x,
                y: rect.y,
                width: width.left,
                height: rect.height,
            }, color);
        }

        // Right border
        if width.right > 0.0 {
            self.fill_rect(&LayoutRect {
                x: rect.x + rect.width - width.right,
                y: rect.y,
                width: width.right,
                height: rect.height,
            }, color);
        }
    }

    fn draw_text(&mut self, text: &str, x: f64, y: f64, color: &Color, font_size: f64, font_weight: FontWeight, underline: bool) {
        // Simple bitmap font rendering
        let char_width = (font_size * 0.6) as u32;
        let char_height = font_size as u32;
        let is_bold = matches!(font_weight, FontWeight::Bold) || 
            matches!(font_weight, FontWeight::Numeric(n) if n >= 700);

        let mut cursor_x = x as u32;
        let cursor_y = y as u32;

        for ch in text.chars() {
            if ch == ' ' {
                cursor_x += char_width;
                continue;
            }

            // Draw character using simple box representation
            self.draw_simple_char(ch, cursor_x, cursor_y, char_width, char_height, color, is_bold);
            cursor_x += char_width;
        }

        // Draw underline if needed
        if underline {
            let underline_y = cursor_y + char_height + 2;
            self.draw_line(x, underline_y as f64, cursor_x as f64, underline_y as f64, color);
        }
    }

    fn draw_simple_char(&mut self, ch: char, x: u32, y: u32, w: u32, h: u32, color: &Color, bold: bool) {
        // Very simple character rendering - draws basic shapes
        let c = color.to_u32();
        let thickness = if bold { 2 } else { 1 };

        // Basic character patterns (simplified)
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' => {
                // Draw a simple filled rectangle for letters/numbers
                for dy in 0..h {
                    for dx in 0..w {
                        let px = x + dx;
                        let py = y + dy;
                        if px < self.width && py < self.height {
                            // Create letter-like patterns
                            let show_pixel = match ch.to_ascii_uppercase() {
                                'I' | 'L' | '1' => dx == w / 2 || dy == 0 || dy == h - 1,
                                'O' | '0' => (dx == 0 || dx == w - 1 || dy == 0 || dy == h - 1) &&
                                           !((dx == 0 && dy == 0) || (dx == w-1 && dy == 0) ||
                                             (dx == 0 && dy == h-1) || (dx == w-1 && dy == h-1)),
                                'T' => dy == 0 || dx == w / 2,
                                'E' | 'F' => dx == 0 || dy == 0 || (ch != 'F' && dy == h - 1) || dy == h / 2,
                                'H' => dx == 0 || dx == w - 1 || dy == h / 2,
                                'C' => (dx == 0 || dy == 0 || dy == h - 1) && dx < w - 1,
                                _ => {
                                    // Default: draw outline + some fill
                                    dx < thickness as u32 || dx >= w - thickness as u32 ||
                                    dy < thickness as u32 || dy >= h - thickness as u32 ||
                                    (dy > h/3 && dy < 2*h/3)
                                }
                            };

                            if show_pixel {
                                let idx = (py * self.width + px) as usize;
                                if idx < self.buffer.len() {
                                    self.buffer[idx] = c;
                                }
                            }
                        }
                    }
                }
            }
            '.' => {
                // Draw a dot at the bottom
                let dot_size = w.min(h) / 4;
                for dy in 0..dot_size {
                    for dx in 0..dot_size {
                        let px = x + w / 2 - dot_size / 2 + dx;
                        let py = y + h - dot_size + dy;
                        if px < self.width && py < self.height {
                            let idx = (py * self.width + px) as usize;
                            if idx < self.buffer.len() {
                                self.buffer[idx] = c;
                            }
                        }
                    }
                }
            }
            ',' => {
                let dot_size = w.min(h) / 4;
                for dy in 0..dot_size + 2 {
                    let dx = w / 2;
                    let px = x + dx;
                    let py = y + h - dot_size + dy;
                    if px < self.width && py < self.height {
                        let idx = (py * self.width + px) as usize;
                        if idx < self.buffer.len() {
                            self.buffer[idx] = c;
                        }
                    }
                }
            }
            '-' => {
                for dx in 0..w {
                    let px = x + dx;
                    let py = y + h / 2;
                    if px < self.width && py < self.height {
                        let idx = (py * self.width + px) as usize;
                        if idx < self.buffer.len() {
                            self.buffer[idx] = c;
                        }
                    }
                }
            }
            ':' => {
                let dot_size = w.min(h) / 5;
                // Top dot
                for dy in 0..dot_size {
                    for dx in 0..dot_size {
                        let px = x + w / 2 - dot_size / 2 + dx;
                        let py = y + h / 3 + dy;
                        if px < self.width && py < self.height {
                            let idx = (py * self.width + px) as usize;
                            if idx < self.buffer.len() {
                                self.buffer[idx] = c;
                            }
                        }
                    }
                }
                // Bottom dot
                for dy in 0..dot_size {
                    for dx in 0..dot_size {
                        let px = x + w / 2 - dot_size / 2 + dx;
                        let py = y + 2 * h / 3 + dy;
                        if px < self.width && py < self.height {
                            let idx = (py * self.width + px) as usize;
                            if idx < self.buffer.len() {
                                self.buffer[idx] = c;
                            }
                        }
                    }
                }
            }
            _ => {
                // Draw a generic block for unknown characters
                for dy in (h/4)..(3*h/4) {
                    for dx in (w/4)..(3*w/4) {
                        let px = x + dx;
                        let py = y + dy;
                        if px < self.width && py < self.height {
                            let idx = (py * self.width + px) as usize;
                            if idx < self.buffer.len() {
                                self.buffer[idx] = c;
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_image(&mut self, data: &[u8], img_width: u32, img_height: u32, rect: &LayoutRect) {
        let dest_x = rect.x as i32;
        let dest_y = rect.y as i32;
        let dest_w = rect.width as u32;
        let dest_h = rect.height as u32;

        // Simple nearest-neighbor scaling
        for dy in 0..dest_h {
            for dx in 0..dest_w {
                let px = dest_x + dx as i32;
                let py = dest_y + dy as i32;

                if px >= 0 && py >= 0 && (px as u32) < self.width && (py as u32) < self.height {
                    // Map to source pixel
                    let src_x = ((dx as f64 / dest_w as f64) * img_width as f64) as u32;
                    let src_y = ((dy as f64 / dest_h as f64) * img_height as f64) as u32;

                    let src_idx = ((src_y * img_width + src_x) * 4) as usize;
                    if src_idx + 3 < data.len() {
                        let r = data[src_idx];
                        let g = data[src_idx + 1];
                        let b = data[src_idx + 2];
                        let a = data[src_idx + 3];

                        if a > 0 {
                            let color = Color { r, g, b, a };
                            let idx = (py as u32 * self.width + px as u32) as usize;
                            if idx < self.buffer.len() {
                                if a == 255 {
                                    self.buffer[idx] = color.to_u32();
                                } else {
                                    let bg = self.buffer[idx];
                                    let bg_color = Color {
                                        r: ((bg >> 16) & 0xFF) as u8,
                                        g: ((bg >> 8) & 0xFF) as u8,
                                        b: (bg & 0xFF) as u8,
                                        a: 255,
                                    };
                                    self.buffer[idx] = color.blend_over(&bg_color).to_u32();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: &Color) {
        // Bresenham's line algorithm
        let mut x1 = x1 as i32;
        let mut y1 = y1 as i32;
        let x2 = x2 as i32;
        let y2 = y2 as i32;

        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        let c = color.to_u32();

        loop {
            if x1 >= 0 && y1 >= 0 && (x1 as u32) < self.width && (y1 as u32) < self.height {
                let idx = (y1 as u32 * self.width + x1 as u32) as usize;
                if idx < self.buffer.len() {
                    self.buffer[idx] = c;
                }
            }

            if x1 == x2 && y1 == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x1 += sx;
            }
            if e2 <= dx {
                err += dx;
                y1 += sy;
            }
        }
    }

    /// Get the pixel buffer for display
    pub fn get_buffer(&self) -> &[u32] {
        &self.buffer
    }
}
