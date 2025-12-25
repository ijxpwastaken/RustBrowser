use crate::{Color, SoftwareRenderer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeType {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_surface: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub border: Color,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            bg_primary: Color { r: 240, g: 242, b: 245, a: 255 },
            bg_secondary: Color { r: 255, g: 255, b: 255, a: 255 },
            bg_surface: Color { r: 255, g: 255, b: 255, a: 255 }, // Cards
            text_primary: Color { r: 0, g: 0, b: 0, a: 255 },
            text_secondary: Color { r: 100, g: 100, b: 110, a: 255 },
            accent: Color { r: 37, g: 99, b: 235, a: 255 }, // Blue 600
            border: Color { r: 220, g: 220, b: 230, a: 255 },
        }
    }

    pub fn dark() -> Self {
        Self {
            bg_primary: Color { r: 18, g: 18, b: 24, a: 255 },
            bg_secondary: Color { r: 28, g: 28, b: 35, a: 255 },
            bg_surface: Color { r: 45, g: 45, b: 55, a: 255 },
            text_primary: Color { r: 250, g: 250, b: 255, a: 255 },
            text_secondary: Color { r: 160, g: 160, b: 175, a: 255 },
            accent: Color { r: 99, g: 102, b: 241, a: 255 }, // Indigo
            border: Color { r: 55, g: 55, b: 70, a: 255 },
        }
    }
}

pub struct Widgets;

impl Widgets {
    pub fn draw_rect(buffer: &mut [u32], width: u32, height: u32, x: u32, y: u32, w: u32, h: u32, color: &Color) {
        let rect = crate::layout::Rect {
            x: x as f64,
            y: y as f64,
            width: w as f64,
            height: h as f64,
        };
        crate::fill_rect_buffer(buffer, width, height, &rect, color, 0, 0);
    }

    pub fn draw_text(buffer: &mut [u32], width: u32, height: u32, text: &str, x: u32, y: u32, color: &Color) {
       super::draw_simple_text_buffer(buffer, width, height, text, x, y, color);
    }

    pub fn draw_button(
        buffer: &mut [u32], width: u32, height: u32,
        x: u32, y: u32, w: u32, h: u32,
        label: &str,
        theme: &Theme,
        hover: bool,
    ) -> bool {
        let bg = if hover { &theme.bg_surface } else { &theme.bg_secondary };
        let border = if hover { &theme.accent } else { &theme.border };
        
        // Background
        Self::draw_rect(buffer, width, height, x, y, w, h, bg);
        // Border
        Self::draw_rect(buffer, width, height, x, y, w, 1, border);
        Self::draw_rect(buffer, width, height, x, y+h-1, w, 1, border);
        Self::draw_rect(buffer, width, height, x, y, 1, h, border);
        Self::draw_rect(buffer, width, height, x+w-1, y, 1, h, border);
        
        // Text centered
        let text_w = (label.len() as u32) * 8; 
        let text_x = x + (w.saturating_sub(text_w)) / 2;
        let text_y = y + (h.saturating_sub(10)) / 2;
        
        Self::draw_text(buffer, width, height, label, text_x, text_y, &theme.text_primary);
        
        false 
    }
    
    pub fn draw_toggle(
         buffer: &mut [u32], width: u32, height: u32,
         x: u32, y: u32,
         value: bool,
         theme: &Theme,
    ) {
        let w = 40;
        let h = 20;
        
        let bg = if value { &theme.accent } else { &theme.bg_surface };
        
        // Track/Background
        Self::draw_rect(buffer, width, height, x, y, w, h, bg);
        
        // Knob
        let knob_x = if value { x + w - h + 2 } else { x + 2 };
        let knob_color = &theme.text_primary;
        Self::draw_rect(buffer, width, height, knob_x, y + 2, h - 4, h - 4, knob_color);
    }

    pub fn draw_slider(
        buffer: &mut [u32], width: u32, height: u32,
        x: u32, y: u32, w: u32,
        value: f32, // 0.0 to 1.0
        theme: &Theme,
    ) {
         let h = 20;
         let track_h = 4;
         let track_y = y + (h - track_h) / 2;
         
         // Track Background
         Self::draw_rect(buffer, width, height, x, track_y, w, track_h, &theme.bg_surface);
         
         // Filled Track
         let fill_w = (w as f32 * value) as u32;
         Self::draw_rect(buffer, width, height, x, track_y, fill_w, track_h, &theme.accent);
         
         // Thumb
         let thumb_x = x + fill_w.saturating_sub(6);
         let thumb_y = y + (h - 12) / 2;
         Self::draw_rect(buffer, width, height, thumb_x, thumb_y, 12, 12, &theme.text_primary);
    }
}
