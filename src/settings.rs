use render::{Color, widgets::{Theme, ThemeType, Widgets}};

pub struct UserSettings {
    pub theme_type: ThemeType,
    pub max_memory_mb: u64,
    pub show_fps: bool,
    pub html_verify: bool,
    pub active_theme: Theme,
    
    // UI State
    pub is_open: bool,
}

impl UserSettings {
    pub fn new() -> Self {
        Self {
            theme_type: ThemeType::Dark,
            max_memory_mb: 1024,
            show_fps: false,
            html_verify: false,
            active_theme: Theme::dark(),
            is_open: false,
        }
    }

    pub fn set_theme(&mut self, theme_type: ThemeType) {
        self.theme_type = theme_type;
        self.active_theme = match theme_type {
            ThemeType::Light => Theme::light(),
            ThemeType::Dark => Theme::dark(),
        };
    }
}

pub struct SettingsState {
    pub data: UserSettings,
    pub last_click_processed: bool, // Anti-bounce
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            data: UserSettings::new(),
            last_click_processed: false,
        }
    }
}

pub enum SettingsAction {
    ToggleTheme,
    ToggleVerify,
    Close,
}

pub fn draw_settings_overlay(
    buffer: &mut [u32], width: u32, height: u32,
    settings: &SettingsState,
    mouse_pos: (f64, f64),
    mouse_down: bool,
) -> Option<SettingsAction> {
    if !settings.data.is_open {
        return None;
    }

    let theme = &settings.data.active_theme;

    // Overlay background (semi-transparent)
    let overlay_color = Color { r: 0, g: 0, b: 0, a: 150 };
    Widgets::draw_rect(buffer, width, height, 0, 0, width, height, &overlay_color);

    // Settings Window
    let win_w = 400;
    let win_h = 300;
    let win_x = (width - win_w) / 2;
    let win_y = (height - win_h) / 2;

    // Window Body
    Widgets::draw_rect(buffer, width, height, win_x, win_y, win_w, win_h, &theme.bg_primary);
    
    // Header
    let header_h = 40;
    Widgets::draw_rect(buffer, width, height, win_x, win_y, win_w, header_h, &theme.bg_secondary);
    Widgets::draw_text(buffer, width, height, "Settings", win_x + 16, win_y + 16, &theme.text_primary);
    
    // Close Button (top-right)
    let close_hover = is_in_rect(mouse_pos, win_x + win_w - 32, win_y + 8, 24, 24);
    if Widgets::draw_button(buffer, width, height, win_x + win_w - 32, win_y + 8, 24, 24, "X", theme, close_hover) {
        // Handle click in main loop usually, but for now we visual only
    }

    let mut content_y = win_y + header_h + 20;
    let x_left = win_x + 20;
    let x_right = win_x + win_w - 60; // For toggles

    // 1. Theme Toggle
    Widgets::draw_text(buffer, width, height, "Theme (Light / Dark)", x_left, content_y + 4, &theme.text_primary);
    let is_dark = settings.data.theme_type == ThemeType::Dark;
    Widgets::draw_toggle(buffer, width, height, x_right, content_y, is_dark, theme);
    // Detection of toggle click:
    if mouse_down && is_in_rect(mouse_pos, x_right, content_y, 40, 20) {
       return Some(SettingsAction::ToggleTheme);
    }
    content_y += 40;

    // 2. Hardware Limit (Memory)
    Widgets::draw_text(buffer, width, height, "Max Memory (MB)", x_left, content_y + 4, &theme.text_primary);
    // Slider logic
    let slider_w = 120;
    let slider_val = (settings.data.max_memory_mb as f32 / 4096.0).clamp(0.0, 1.0);
    Widgets::draw_slider(buffer, width, height, x_right - 80, content_y, slider_w, slider_val, theme);
    Widgets::draw_text(buffer, width, height, &format!("{}MB", settings.data.max_memory_mb), x_right + 50, content_y + 4, &theme.text_secondary);
    content_y += 40;
    
    // 3. HTML Verify Mode
    Widgets::draw_text(buffer, width, height, "HTML Verify Mode", x_left, content_y + 4, &theme.text_primary);
    Widgets::draw_toggle(buffer, width, height, x_right, content_y, settings.data.html_verify, theme);
    if mouse_down && is_in_rect(mouse_pos, x_right, content_y, 40, 20) {
       return Some(SettingsAction::ToggleVerify);
    }
    content_y += 40;
    
    // Check Close Click
    if mouse_down && is_in_rect(mouse_pos, win_x + win_w - 32, win_y + 8, 24, 24) {
        return Some(SettingsAction::Close);
    }
    // Check outside Click
    if mouse_down && !is_in_rect(mouse_pos, win_x, win_y, win_w, win_h) {
        return Some(SettingsAction::Close);
    }

    None
}

fn is_in_rect(pos: (f64, f64), x: u32, y: u32, w: u32, h: u32) -> bool {
    pos.0 >= x as f64 && pos.0 <= (x + w) as f64 && pos.1 >= y as f64 && pos.1 <= (y + h) as f64
}
