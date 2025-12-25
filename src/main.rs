//! Modern Rust Browser with Privacy Shield
//! 
//! A browser engine built from scratch with ad blocking and tracker prevention.

use std::num::NonZeroU32;
use std::rc::Rc;

use browser_core::Browser;
use browser_core::adblocker::AdvancedPrivacyShield;
use render::{Color, DisplayCommand};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent, ElementState, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::keyboard::{Key, NamedKey};

mod settings;
mod icons;
use settings::{SettingsState, SettingsAction, draw_settings_overlay};
use render::widgets::{Theme, ThemeType};
use layout::Rect;
use icons::*;

const DEFAULT_WIDTH: u32 = 1400;
const DEFAULT_HEIGHT: u32 = 950;

// Remove hardcoded const colors, we will use settings.theme

const HEADER_HEIGHT: u32 = 48;
const TAB_BAR_HEIGHT: u32 = 40;
const TOOLBAR_HEIGHT: u32 = 56;
const TOTAL_CHROME_HEIGHT: u32 = HEADER_HEIGHT + TAB_BAR_HEIGHT + TOOLBAR_HEIGHT;
const ADDRESSBAR_HEIGHT: u32 = 38;
const BUTTON_SIZE: u32 = 36;
const TAB_WIDTH: u32 = 200;
const TAB_HEIGHT: u32 = 32;

// Tab state holding the full browser session
struct TabSession {
    browser: Browser,
    title: String,
    url: String, // Current URL for address bar (might differ from loaded if navigating)
    scroll_offset: i32,
    history: Vec<String>,
    history_index: usize,
}

impl TabSession {
    fn new(width: u32, height: u32, initial_url: &str) -> Self {
        let mut browser = Browser::new(width, height);
        // Pre-load default
        if initial_url == "about:test" {
            let _ = browser.load_html(TEST_HTML.to_string());
        }
        
        TabSession {
            browser,
            title: "New Tab".to_string(),
            url: initial_url.to_string(),
            scroll_offset: 0,
            history: vec![initial_url.to_string()],
            history_index: 0,
        }
    }

    fn navigate(&mut self, url: &str) {
        println!("[Tab] Navigating to: {}", url);
        self.url = url.to_string();
        
        // Basic history push
        if self.history_index < self.history.len().saturating_sub(1) {
             self.history.truncate(self.history_index + 1);
        }
        self.history.push(url.to_string());
        self.history_index = self.history.len() - 1;

        if url.starts_with("http") {
             match self.browser.load_url(url) {
                 Ok(_) => println!("[Tab] Loaded: {}", url),
                 Err(e) => eprintln!("[Tab] Error: {}", e),
             }
        } else if url == "about:test" {
             let _ = self.browser.load_html(TEST_HTML.to_string());
        }
        self.scroll_offset = 0;
        
        // Update title (mock)
        self.title = if url.len() > 15 { url[..15].to_string() } else { url.to_string() };
    }
}

struct TabUI {
    title: String,
    active: bool,
}

const TEST_HTML: &str = include_str!("test_page.html");

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("🦊 Rust Browser - Privacy Shield")
            .with_inner_size(LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .build(&event_loop)
            .unwrap()
    );

    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

    let privacy_shield = AdvancedPrivacyShield::new();
    
    // Settings State
    let mut settings_state = SettingsState::new();

    let mut tabs: Vec<TabSession> = Vec::new();
    // Create initial tab
    tabs.push(TabSession::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, "about:test"));
    tabs.push(TabSession::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, "https://www.google.com"));
    let mut active_tab_index = 1;

    let mut mouse_pos: (f64, f64) = (0.0, 0.0);
    let mut addressbar_focused = false;
    let mut url_input = String::from("about:test");
    let mut mouse_down = false; // Track mouse state for UI interactions

    // Load the embedded test page
    println!("[Browser] Loading test page in Tab 1...");
    // Initial load handled by TabSession::new


    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("\n[Browser] Session Stats:");
                        println!("  {}", privacy_shield.get_stats_summary());
                        elwt.exit();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_pos = (position.x, position.y);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let Some(tab) = tabs.get_mut(active_tab_index) {
                            match delta {
                                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                                    tab.scroll_offset = (tab.scroll_offset - (y * 50.0) as i32).max(0);
                                }
                                winit::event::MouseScrollDelta::PixelDelta(pos) => {
                                    tab.scroll_offset = (tab.scroll_offset - pos.y as i32).max(0);
                                }
                            }
                        }
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {
                            if event.logical_key == Key::Named(NamedKey::F1) {
                                settings_state.data.is_open = !settings_state.data.is_open;
                                println!("[Settings] Toggled: {}", settings_state.data.is_open);
                                window.request_redraw();
                            }

                            if addressbar_focused {
                                if let Some(tab) = tabs.get_mut(active_tab_index) {
                                    match &event.logical_key {
                                        Key::Named(NamedKey::Enter) => {
                                            println!("[Navigate] {}", url_input);
                                            
                                            // Memory Check (Mock)
                                            if settings_state.data.max_memory_mb < 512 {
                                                tab.browser.clear_cache(); 
                                            }
                                            
                                            let target_url = if url_input.starts_with("http") || url_input == "about:test" {
                                                url_input.clone()
                                            } else {
                                                format!("https://{}", url_input)
                                            };
                                            
                                            tab.navigate(&target_url);
                                            url_input = tab.url.clone();
                                            addressbar_focused = false;
                                        }
                                        Key::Named(NamedKey::Backspace) => {
                                            url_input.pop();
                                        }
                                        Key::Named(NamedKey::Escape) => {
                                            addressbar_focused = false;
                                            url_input = tab.url.clone();
                                        }
                                        Key::Character(c) => {
                                            url_input.push_str(&c.to_string());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                        mouse_down = state == ElementState::Pressed;
                        
                        if settings_state.data.is_open && mouse_down {
                            // Let the redraw loop handle settings interaction via calling logic or handle here?
                            // Since we have a `draw_settings_overlay` that returns actions, 
                            // we usually need to process it after drawing or have separate "process_click" logic.
                            // For simplicity, we will handle settings interaction in the Redraw loop where we have the `buffer` to draw,
                            // OR we just request redraw and let the draw function return actions.
                            // Since Winit event handling and Drawing are separate, we'll flagging `mouse_down` is enough.
                            // However, we should prevent clicks from going through to the browser if settings is open.
                            window.request_redraw();
                        } else if mouse_down {
                             let (mx, my) = mouse_pos;
                             // ... rest of existing mouse tracking ...

                        let size = window.inner_size();
                        
                        // Check toolbar interactions
                        let toolbar_y = HEADER_HEIGHT + TAB_BAR_HEIGHT;
                        
                        // Address bar click
                        let ab_x = 160;
                        let ab_y = toolbar_y + (TOOLBAR_HEIGHT - ADDRESSBAR_HEIGHT) / 2;
                        let ab_width = size.width.saturating_sub(ab_x + 120);
                        
                        if mx >= ab_x as f64 && mx <= (ab_x + ab_width) as f64
                            && my >= ab_y as f64 && my <= (ab_y + ADDRESSBAR_HEIGHT) as f64 
                        {
                            addressbar_focused = true;
                            if let Some(tab) = tabs.get(active_tab_index) {
                                url_input = tab.url.clone();
                            }
                        } else {
                            addressbar_focused = false;
                        }
                        // Handle Tab Clicks
                        if my >= HEADER_HEIGHT as f64 && my <= (HEADER_HEIGHT + TAB_BAR_HEIGHT) as f64 {
                            let mut tab_x = 12;
                            // Check tabs
                            for i in 0..tabs.len() {
                                if mx >= tab_x as f64 && mx <= (tab_x + TAB_WIDTH) as f64 {
                                    // Clicked tab i
                                    active_tab_index = i;
                                    if let Some(tab) = tabs.get(active_tab_index) {
                                        url_input = tab.url.clone();
                                    }
                                    println!("[Tabs] Switched to tab {}", i);
                                    
                                    // Close button relative to tab
                                    let close_x = tab_x + TAB_WIDTH - 24;
                                    if mx >= close_x as f64 && mx <= (close_x + 16) as f64 {
                                         println!("[Tabs] Close tab {}", i);
                                         if tabs.len() > 1 {
                                             tabs.remove(i);
                                             if active_tab_index >= tabs.len() {
                                                 active_tab_index = tabs.len().saturating_sub(1);
                                             }
                                             if let Some(tab) = tabs.get(active_tab_index) {
                                                url_input = tab.url.clone();
                                             }
                                         }
                                    }
                                    break;
                                }
                                tab_x += TAB_WIDTH + 4;
                            }
                            
                            // Check "New Tab" button
                            if mx >= tab_x as f64 && mx <= (tab_x + 28) as f64 {
                                println!("[Tabs] New Tab");
                                tabs.push(TabSession::new(size.width, size.height, "about:test"));
                                // Switch to new tab
                                active_tab_index = tabs.len() - 1;
                                url_input = "about:test".to_string();
                            }
                        }
                        
                        else if my > toolbar_y as f64 && my < (toolbar_y + TOOLBAR_HEIGHT) as f64 {
                            // Toolbar button clicks
                            let button_y = toolbar_y + (TOOLBAR_HEIGHT - BUTTON_SIZE) / 2;
                            
                            // Back
                            if mx >= 12.0 && mx <= (12 + BUTTON_SIZE) as f64
                                && my >= button_y as f64 && my <= (button_y + BUTTON_SIZE) as f64 
                            {
                                println!("[Nav] ← Back");
                                if let Some(tab) = tabs.get_mut(active_tab_index) {
                                    if let Err(e) = tab.browser.go_back() {
                                         eprintln!("Back failed: {}", e);
                                    }
                                    tab.url = tab.browser.current_url().clone().unwrap_or_default().to_string();
                                    url_input = tab.url.clone();
                                }
                            }
                            
                            // Forward
                            let fwd_x = 12 + BUTTON_SIZE + 8;
                            if mx >= fwd_x as f64 && mx <= (fwd_x + BUTTON_SIZE) as f64
                                && my >= button_y as f64 && my <= (button_y + BUTTON_SIZE) as f64 
                            {
                                println!("[Nav] → Forward");
                                if let Some(tab) = tabs.get_mut(active_tab_index) {
                                    if let Err(e) = tab.browser.go_forward() {
                                        eprintln!("Forward failed: {}", e);
                                    }
                                    tab.url = tab.browser.current_url().clone().unwrap_or_default().to_string();
                                    url_input = tab.url.clone();
                                }
                            }
                            
                            // Refresh
                            let ref_x = 12 + (BUTTON_SIZE + 8) * 2;
                            if mx >= ref_x as f64 && mx <= (ref_x + BUTTON_SIZE) as f64
                                && my >= button_y as f64 && my <= (button_y + BUTTON_SIZE) as f64 
                            {
                                println!("[Nav] ↻ Refresh");
                                if let Some(tab) = tabs.get_mut(active_tab_index) {
                                     tab.navigate(&url_input.clone()); // Re-navigate
                                }
                            }
                            // Settings Click
                            let settings_x = size.width.saturating_sub(180);
                            if mx >= settings_x as f64 && mx <= (settings_x + BUTTON_SIZE) as f64
                                && my >= button_y as f64 && my <= (button_y + BUTTON_SIZE) as f64
                            {
                                settings_state.data.is_open = !settings_state.data.is_open;
                                window.request_redraw();
                            }
                        } else {
                            addressbar_focused = false;
                        }
                        }
                    }
                    WindowEvent::Resized(size) => {
                        for tab in &mut tabs {
                            tab.browser.resize(size.width, size.height);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        let size = window.inner_size();
                        let width = size.width;
                        let height = size.height;

                        if width == 0 || height == 0 {
                            return;
                        }

                        let _ = surface.resize(
                            NonZeroU32::new(width).unwrap(),
                            NonZeroU32::new(height).unwrap(),
                        );

                        let mut buffer = match surface.buffer_mut() {
                            Ok(b) => b,
                            Err(_) => return,
                        };

                        if let Some(active_tab) = tabs.get(active_tab_index) {
                             // Prepare mock tabs for UI
                             let ui_tabs: Vec<TabUI> = tabs.iter().enumerate().map(|(i, t)| TabUI {
                                 title: t.title.clone(),
                                 active: i == active_tab_index,
                             }).collect();

                             draw_modern_ui(
                                &mut buffer, 
                                width, 
                                height, 
                                if addressbar_focused { &url_input } else { &active_tab.url },
                                &active_tab.browser,
                                &privacy_shield, // Shared shield for now
                                &ui_tabs,
                                addressbar_focused,
                                mouse_pos,
                                active_tab.scroll_offset,
                                &settings_state.data.active_theme,
                            );
                        }

                        // Draw Settings Overlay
                        if let Some(action) = draw_settings_overlay(&mut buffer, width, height, &settings_state, mouse_pos, mouse_down) {
                            match action {
                                SettingsAction::ToggleTheme => {
                                    let new_theme = if settings_state.data.theme_type == ThemeType::Dark { ThemeType::Light } else { ThemeType::Dark };
                                    settings_state.data.set_theme(new_theme);
                                }
                                SettingsAction::ToggleVerify => {
                                    settings_state.data.html_verify = !settings_state.data.html_verify;
                                }
                                SettingsAction::Close => {
                                    settings_state.data.is_open = false;
                                }
                            }
                            // Debounce click prevention
                            mouse_down = false; 
                        }

                        let _ = buffer.present();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}

// ============================================================================
// MODERN UI RENDERING
// ============================================================================

fn draw_modern_ui(
    buffer: &mut [u32], 
    width: u32, 
    height: u32, 
    url: &str, 
    browser: &Browser,
    privacy_shield: &AdvancedPrivacyShield,
    tabs: &[TabUI],
    addressbar_focused: bool,
    mouse_pos: (f64, f64),
    scroll_offset: i32,
    theme: &Theme,
) {
    // 1. Content area (white)
    let content_y = TOTAL_CHROME_HEIGHT;
    let content_height = height.saturating_sub(TOTAL_CHROME_HEIGHT);
    // Use theme.bg_secondary as content bg? Or keep it white?
    // Browser content is usually white unless dark mode reader.
    // For now we assume white page content, but fill background with theme.bg_primary
    fill_rect(buffer, width, height, 0, content_y, width, content_height, &Color::WHITE);
    
    // 2. Render page content
    render_page_content(buffer, width, height, browser, scroll_offset);
    
    // 3. Header bar (window controls area)
    fill_rect(buffer, width, height, 0, 0, width, HEADER_HEIGHT, &theme.bg_primary);
    draw_window_controls(buffer, width, height, mouse_pos, theme);
    
    // 4. Tab bar
    let tab_y = HEADER_HEIGHT;
    fill_rect(buffer, width, height, 0, tab_y, width, TAB_BAR_HEIGHT, &theme.bg_secondary);
    draw_tabs(buffer, width, height, tabs, tab_y, mouse_pos, theme);
    
    // 5. Toolbar with address bar
    let toolbar_y = HEADER_HEIGHT + TAB_BAR_HEIGHT;
    fill_rect(buffer, width, height, 0, toolbar_y, width, TOOLBAR_HEIGHT, &theme.bg_surface);
    draw_toolbar(buffer, width, height, toolbar_y, url, addressbar_focused, mouse_pos, privacy_shield, theme);
    
    // 6. Separator line
    fill_rect(buffer, width, height, 0, TOTAL_CHROME_HEIGHT - 1, width, 1, &theme.border);
}


fn render_page_content(buffer: &mut [u32], width: u32, height: u32, browser: &Browser, scroll_offset: i32) {
    let display_list = browser.get_display_list();
    
    for command in &display_list.commands {
        match command {
            DisplayCommand::Text { text, x, y, color, font_size, .. } => {
                let render_y = (*y as i32) + (TOTAL_CHROME_HEIGHT as i32) - scroll_offset;
                if render_y > (TOTAL_CHROME_HEIGHT as i32 - 10) && render_y < (height as i32 - 5) {
                    let scale = (*font_size / 14.0).min(2.5).max(0.8);
                    draw_text_scaled(buffer, width, height, text, *x as u32, render_y as u32, color, scale);
                }
            }
            DisplayCommand::SolidColor(color, rect) => {
                let render_y = (rect.y as i32) + (TOTAL_CHROME_HEIGHT as i32) - scroll_offset;
                if render_y > 0 && render_y < height as i32 {
                    fill_rect(buffer, width, height, rect.x as u32, render_y as u32, 
                              rect.width as u32, rect.height as u32, color);
                }
            }
            DisplayCommand::Image { data, rect, .. } => {
                let render_y = (rect.y as i32) + (TOTAL_CHROME_HEIGHT as i32) - scroll_offset;
                if render_y > -(rect.height as i32) && render_y < height as i32 {
                    draw_image_rgba(buffer, width, height, data, 
                                   rect.x as u32, render_y as u32,
                                   rect.width as u32, rect.height as u32);
                }
            }
            DisplayCommand::SvgImage { svg_data, rect } => {
                let render_y = (rect.y as i32) + (TOTAL_CHROME_HEIGHT as i32) - scroll_offset;
                if render_y > -(rect.height as i32) && render_y < height as i32 {
                     // Calculate offset from layout coordinates to screen coordinates
                     let offset_y = (TOTAL_CHROME_HEIGHT as i32) - scroll_offset;
                     render::draw_svg_to_buffer(buffer, width, height, svg_data, rect, 0, offset_y);
                }
            }
            _ => {}
        }
    }
}

fn draw_window_controls(buffer: &mut [u32], width: u32, height: u32, mouse_pos: (f64, f64), theme: &Theme) {
    // Left side: Brand/logo area
    draw_text(buffer, width, height, "RUST BROWSER", 16, 16, &theme.accent);
    
    // Right side: Window controls (placeholder - actual controls handled by OS)
    let controls_x = width.saturating_sub(100);
    
    // Minimize
    let min_hover = is_in_rect(mouse_pos, controls_x, 12, 24, 24);
    fill_rect(buffer, width, height, controls_x, 12, 24, 24, 
        if min_hover { &theme.bg_surface } else { &theme.bg_secondary });
    fill_rect(buffer, width, height, controls_x + 7, 22, 10, 2, &theme.text_secondary);
    
    // Maximize  
    let max_x = controls_x + 28;
    let max_hover = is_in_rect(mouse_pos, max_x, 12, 24, 24);
    fill_rect(buffer, width, height, max_x, 12, 24, 24,
        if max_hover { &theme.bg_surface } else { &theme.bg_secondary });
    draw_rect_outline(buffer, width, height, max_x + 7, 16, 10, 10, &theme.text_secondary);
    
    // Close
    let close_x = controls_x + 56;
    let close_hover = is_in_rect(mouse_pos, close_x, 12, 24, 24);
    fill_rect(buffer, width, height, close_x, 12, 24, 24,
        if close_hover { &Color::RED } else { &theme.bg_secondary }); // Use standard red for close
    draw_text(buffer, width, height, "X", close_x + 8, 16, 
        if close_hover { &theme.text_primary } else { &theme.text_secondary });
}

fn draw_tabs(buffer: &mut [u32], width: u32, height: u32, tabs: &[TabUI], y: u32, mouse_pos: (f64, f64), theme: &Theme) {
    let mut tab_x = 12u32;
    
    for (_i, tab) in tabs.iter().enumerate() {
        let is_active = tab.active;
        let is_hover = is_in_rect(mouse_pos, tab_x, y + 4, TAB_WIDTH, TAB_HEIGHT);
        
        // Tab background
        let bg = if is_active {
            &theme.bg_surface
        } else if is_hover {
            &theme.bg_secondary
        } else {
            &theme.bg_primary
        };
        
        // Rounded tab shape
        fill_rounded_rect(buffer, width, height, tab_x, y + 6, TAB_WIDTH, TAB_HEIGHT, 6, bg);
        
        // Tab title
        let title_display: String = if tab.title.len() > 20 { 
            format!("{}...", &tab.title[..17]) 
        } else { 
            tab.title.clone() 
        };
        draw_text(buffer, width, height, &title_display, tab_x + 12, y + 16, 
            if is_active { &theme.text_primary } else { &theme.text_secondary });
        
        // Close button on tab
        let close_x = tab_x + TAB_WIDTH - 24;
        let close_hover = is_in_rect(mouse_pos, close_x, y + 10, 16, 16);
        if is_hover || is_active {
            draw_text(buffer, width, height, "x", close_x + 4, y + 14,
                if close_hover { &theme.text_primary } else { &theme.text_secondary });
        }
        
        // Active indicator
        if is_active {
            fill_rect(buffer, width, height, tab_x + 8, y + TAB_HEIGHT + 4, TAB_WIDTH - 16, 2, &theme.accent);
        }
        
        tab_x += TAB_WIDTH + 4;
    }
    
    // New tab button
    let new_tab_hover = is_in_rect(mouse_pos, tab_x, y + 8, 28, 28);
    fill_rounded_rect(buffer, width, height, tab_x, y + 8, 28, 28, 4,
        if new_tab_hover { &theme.bg_surface } else { &theme.bg_secondary });
    draw_text(buffer, width, height, "+", tab_x + 9, y + 14, &theme.text_secondary);
}

fn draw_toolbar(buffer: &mut [u32], width: u32, height: u32, y: u32, url: &str, 
    focused: bool, mouse_pos: (f64, f64), privacy_shield: &AdvancedPrivacyShield, theme: &Theme) {
    
    let button_y = y + (TOOLBAR_HEIGHT - BUTTON_SIZE) / 2;
    
    // Navigation buttons with icons
    // Back
    let back_hover = is_in_rect(mouse_pos, 12, button_y, BUTTON_SIZE, BUTTON_SIZE);
    draw_nav_button(buffer, width, height, 12, button_y, ICON_BACK, back_hover, theme);
    
    // Forward
    let fwd_x = 12 + BUTTON_SIZE + 8;
    let fwd_hover = is_in_rect(mouse_pos, fwd_x, button_y, BUTTON_SIZE, BUTTON_SIZE);
    draw_nav_button(buffer, width, height, fwd_x, button_y, ICON_FORWARD, fwd_hover, theme);
    
    // Refresh
    let ref_x = 12 + (BUTTON_SIZE + 8) * 2;
    let ref_hover = is_in_rect(mouse_pos, ref_x, button_y, BUTTON_SIZE, BUTTON_SIZE);
    draw_nav_button(buffer, width, height, ref_x, button_y, ICON_REFRESH, ref_hover, theme);
    
    // Settings (New Button)
    let settings_x = width.saturating_sub(180); // Move left of shield
    let settings_hover = is_in_rect(mouse_pos, settings_x, button_y, BUTTON_SIZE, BUTTON_SIZE);
    // Draw gear icon
    draw_nav_button(buffer, width, height, settings_x, button_y, ICON_SETTINGS, settings_hover, theme);
    
    // Address bar - modern pill shape
    let ab_x = 160;
    let ab_y = y + (TOOLBAR_HEIGHT - ADDRESSBAR_HEIGHT) / 2;
    // Adjust width to account for settings button
    let ab_width = width.saturating_sub(ab_x + 200);
    
    // Address bar background with border
    let border = if focused { &theme.accent } else { &theme.border };
    fill_rounded_rect(buffer, width, height, ab_x, ab_y, ab_width, ADDRESSBAR_HEIGHT, 8, border);
    fill_rounded_rect(buffer, width, height, ab_x + 2, ab_y + 2, 
        ab_width - 4, ADDRESSBAR_HEIGHT - 4, 6, &theme.bg_primary);
    
    // Lock icon for HTTPS
    if url.starts_with("https://") {
        draw_text(buffer, width, height, "[S]", ab_x + 10, ab_y + 12, &Color::GREEN);
    }
    
    // URL text
    let url_x = ab_x + if url.starts_with("https://") { 36 } else { 12 };
    let max_chars = (ab_width.saturating_sub(60) / 8) as usize;
    let display_url = if url.len() > max_chars && max_chars > 3 { &url[..max_chars] } else { url };
    
    let url_display = if focused {
        format!("{}|", display_url)
    } else {
        display_url.to_string()
    };
    draw_text(buffer, width, height, &url_display, url_x, ab_y + 12, &theme.text_primary);
    
    // Shield icon with stats
    let shield_x = width.saturating_sub(100);
    let total_blocked = privacy_shield.total_blocks();
    
    // Shield button
    let shield_hover = is_in_rect(mouse_pos, shield_x, button_y, 70, BUTTON_SIZE);
    fill_rounded_rect(buffer, width, height, shield_x, button_y, 70, BUTTON_SIZE, 6,
        if shield_hover { &theme.bg_surface } else { &theme.bg_secondary });
    
    // Shield icon
    draw_text(buffer, width, height, "S", shield_x + 10, button_y + 10, &Color::GREEN);
    
    // Block count
    let count_str = format!("{}", total_blocked);
    draw_text(buffer, width, height, &count_str, shield_x + 28, button_y + 10, &theme.text_primary);
}

fn draw_nav_button(buffer: &mut [u32], width: u32, height: u32, x: u32, y: u32, icon: &str, hover: bool, theme: &Theme) {
    let bg = if hover { &theme.bg_surface } else { &theme.bg_secondary };
    fill_rounded_rect(buffer, width, height, x, y, BUTTON_SIZE, BUTTON_SIZE, 6, bg);
    
    // Check if it's an SVG (starts with <svg)
    if icon.starts_with("<svg") {
        let padding = 8;
        let icon_size = BUTTON_SIZE - (padding * 2);
        let rect = Rect {
            x: (x + padding) as f64,
            y: (y + padding) as f64,
            width: icon_size as f64,
            height: icon_size as f64,
        };
        // We need an offset for the draw_svg_to_buffer because it assumes page coordinates sometimes?
        // Actually, render::draw_svg_to_buffer takes a generic layout rect and offset.
        // We'll pass 0 offset since we are calculating screen coordinates directly.
        render::draw_svg_to_buffer(buffer, width, height, icon, &rect, 0, 0);
    } else {
        // Fallback for text
        draw_text(buffer, width, height, icon, x + 13, y + 10, 
            if hover { &theme.text_primary } else { &theme.text_secondary });
    }
}

// ============================================================================
// DRAWING PRIMITIVES
// ============================================================================

fn fill_rect(buffer: &mut [u32], buf_width: u32, buf_height: u32, x: u32, y: u32, w: u32, h: u32, color: &Color) {
    let color_u32 = color.to_argb_u32();
    for py in y..(y + h).min(buf_height) {
        for px in x..(x + w).min(buf_width) {
            let idx = (py * buf_width + px) as usize;
            if idx < buffer.len() {
                buffer[idx] = color_u32;
            }
        }
    }
}

fn fill_rounded_rect(buffer: &mut [u32], buf_width: u32, buf_height: u32, x: u32, y: u32, w: u32, h: u32, radius: u32, color: &Color) {
    let color_u32 = color.to_argb_u32();
    let r = radius.min(w / 2).min(h / 2);
    
    for py in y..(y + h).min(buf_height) {
        for px in x..(x + w).min(buf_width) {
            let dx = px.saturating_sub(x);
            let dy = py.saturating_sub(y);
            
            // Check corners
            let in_rect = if dx < r && dy < r {
                // Top-left corner
                let cx = r - dx;
                let cy = r - dy;
                cx * cx + cy * cy <= r * r
            } else if dx >= w - r && dy < r {
                // Top-right corner
                let cx = dx - (w - r);
                let cy = r - dy;
                cx * cx + cy * cy <= r * r
            } else if dx < r && dy >= h - r {
                // Bottom-left corner
                let cx = r - dx;
                let cy = dy - (h - r);
                cx * cx + cy * cy <= r * r
            } else if dx >= w - r && dy >= h - r {
                // Bottom-right corner
                let cx = dx - (w - r);
                let cy = dy - (h - r);
                cx * cx + cy * cy <= r * r
            } else {
                true
            };
            
            if in_rect {
                let idx = (py * buf_width + px) as usize;
                if idx < buffer.len() {
                    buffer[idx] = color_u32;
                }
            }
        }
    }
}

fn draw_rect_outline(buffer: &mut [u32], buf_width: u32, buf_height: u32, x: u32, y: u32, w: u32, h: u32, color: &Color) {
    fill_rect(buffer, buf_width, buf_height, x, y, w, 1, color);
    fill_rect(buffer, buf_width, buf_height, x, y + h - 1, w, 1, color);
    fill_rect(buffer, buf_width, buf_height, x, y, 1, h, color);
    fill_rect(buffer, buf_width, buf_height, x + w - 1, y, 1, h, color);
}

fn is_in_rect(pos: (f64, f64), x: u32, y: u32, w: u32, h: u32) -> bool {
    pos.0 >= x as f64 && pos.0 <= (x + w) as f64 && pos.1 >= y as f64 && pos.1 <= (y + h) as f64
}

fn draw_image_rgba(buffer: &mut [u32], buf_width: u32, buf_height: u32, 
                   data: &[u8], x: u32, y: u32, img_width: u32, img_height: u32) {
    let src_width = (data.len() / 4) as u32 / img_height.max(1);
    if src_width == 0 { return; }
    
    let scale_x = src_width as f64 / img_width as f64;
    let scale_y = img_height as f64 / img_height as f64;
    
    for py in 0..img_height {
        let dst_y = y.saturating_add(py);
        if dst_y >= buf_height { break; }
        
        let src_y = ((py as f64) * scale_y) as u32;
        
        for px in 0..img_width {
            let dst_x = x.saturating_add(px);
            if dst_x >= buf_width { break; }
            
            let src_x = ((px as f64) * scale_x) as u32;
            let src_idx = ((src_y * src_width + src_x) * 4) as usize;
            
            if src_idx + 3 < data.len() {
                let r = data[src_idx] as u32;
                let g = data[src_idx + 1] as u32;
                let b = data[src_idx + 2] as u32;
                let color = (r << 16) | (g << 8) | b;
                
                let dst_idx = (dst_y * buf_width + dst_x) as usize;
                if dst_idx < buffer.len() {
                    buffer[dst_idx] = color;
                }
            }
        }
    }
}

fn draw_text(buffer: &mut [u32], buf_width: u32, buf_height: u32, text: &str, x: u32, y: u32, color: &Color) {
    draw_text_scaled(buffer, buf_width, buf_height, text, x, y, color, 1.0);
}

fn draw_text_scaled(buffer: &mut [u32], buf_width: u32, buf_height: u32, text: &str, x: u32, y: u32, color: &Color, scale: f64) {
    let color_u32 = color.to_argb_u32();
    let char_width = (8.0 * scale) as u32;
    
    for (i, c) in text.chars().enumerate() {
        let cx = x + (i as u32 * char_width);
        if cx + 6 > buf_width { break; }
        draw_char_scaled(buffer, buf_width, buf_height, c, cx, y, color_u32, scale);
    }
}

fn draw_char_scaled(buffer: &mut [u32], buf_width: u32, buf_height: u32, c: char, x: u32, y: u32, color: u32, scale: f64) {
    let pattern = get_char_pattern(c);
    let scaled_height = (8.0 * scale) as u32;
    let scaled_width = (6.0 * scale) as u32;
    
    for row in 0..scaled_height {
        let src_row = ((row as f64) / scale) as usize;
        if src_row >= 8 { continue; }
        
        let py = y + row;
        if py >= buf_height { break; }
        
        for col in 0..scaled_width {
            let src_col = ((col as f64) / scale) as u32;
            if src_col >= 6 { continue; }
            
            if (pattern[src_row] >> (5 - src_col)) & 1 == 1 {
                let px = x + col;
                if px < buf_width {
                    let idx = (py * buf_width + px) as usize;
                    if idx < buffer.len() { buffer[idx] = color; }
                }
            }
        }
    }
}

fn get_char_pattern(c: char) -> [u8; 8] {
    match c {
        'A' => [0b001100, 0b010010, 0b100001, 0b111111, 0b100001, 0b100001, 0b100001, 0],
        'B' => [0b111110, 0b100001, 0b111110, 0b100001, 0b100001, 0b100001, 0b111110, 0],
        'C' => [0b011110, 0b100001, 0b100000, 0b100000, 0b100000, 0b100001, 0b011110, 0],
        'D' => [0b111100, 0b100010, 0b100001, 0b100001, 0b100001, 0b100010, 0b111100, 0],
        'E' => [0b111111, 0b100000, 0b111110, 0b100000, 0b100000, 0b100000, 0b111111, 0],
        'F' => [0b111111, 0b100000, 0b111110, 0b100000, 0b100000, 0b100000, 0b100000, 0],
        'G' => [0b011110, 0b100001, 0b100000, 0b100111, 0b100001, 0b100001, 0b011110, 0],
        'H' => [0b100001, 0b100001, 0b111111, 0b100001, 0b100001, 0b100001, 0b100001, 0],
        'I' => [0b011100, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b011100, 0],
        'J' => [0b000111, 0b000010, 0b000010, 0b000010, 0b100010, 0b100010, 0b011100, 0],
        'K' => [0b100001, 0b100010, 0b100100, 0b111000, 0b100100, 0b100010, 0b100001, 0],
        'L' => [0b100000, 0b100000, 0b100000, 0b100000, 0b100000, 0b100000, 0b111111, 0],
        'M' => [0b100001, 0b110011, 0b101101, 0b100001, 0b100001, 0b100001, 0b100001, 0],
        'N' => [0b100001, 0b110001, 0b101001, 0b100101, 0b100011, 0b100001, 0b100001, 0],
        'O' => [0b011110, 0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b011110, 0],
        'P' => [0b111110, 0b100001, 0b100001, 0b111110, 0b100000, 0b100000, 0b100000, 0],
        'Q' => [0b011110, 0b100001, 0b100001, 0b100001, 0b100101, 0b011110, 0b000001, 0],
        'R' => [0b111110, 0b100001, 0b111110, 0b100100, 0b100010, 0b100001, 0b100001, 0],
        'S' => [0b011110, 0b100001, 0b100000, 0b011110, 0b000001, 0b100001, 0b011110, 0],
        'T' => [0b111111, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0],
        'U' => [0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b011110, 0],
        'V' => [0b100001, 0b100001, 0b100001, 0b100001, 0b010010, 0b001100, 0b001100, 0],
        'W' => [0b100001, 0b100001, 0b100001, 0b101101, 0b101101, 0b110011, 0b100001, 0],
        'X' => [0b100001, 0b010010, 0b001100, 0b001100, 0b010010, 0b100001, 0b100001, 0],
        'Y' => [0b100001, 0b010010, 0b001100, 0b001000, 0b001000, 0b001000, 0b001000, 0],
        'Z' => [0b111111, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000, 0b111111, 0],
        'a'..='z' => get_char_pattern(c.to_ascii_uppercase()),
        '0' => [0b011110, 0b100011, 0b100101, 0b101001, 0b110001, 0b100001, 0b011110, 0],
        '1' => [0b001000, 0b011000, 0b001000, 0b001000, 0b001000, 0b001000, 0b011100, 0],
        '2' => [0b011110, 0b100001, 0b000010, 0b000100, 0b001000, 0b010000, 0b111111, 0],
        '3' => [0b011110, 0b100001, 0b000001, 0b001110, 0b000001, 0b100001, 0b011110, 0],
        '4' => [0b000010, 0b000110, 0b001010, 0b010010, 0b111111, 0b000010, 0b000010, 0],
        '5' => [0b111111, 0b100000, 0b111110, 0b000001, 0b000001, 0b100001, 0b011110, 0],
        '6' => [0b011110, 0b100000, 0b111110, 0b100001, 0b100001, 0b100001, 0b011110, 0],
        '7' => [0b111111, 0b000001, 0b000010, 0b000100, 0b001000, 0b001000, 0b001000, 0],
        '8' => [0b011110, 0b100001, 0b100001, 0b011110, 0b100001, 0b100001, 0b011110, 0],
        '9' => [0b011110, 0b100001, 0b100001, 0b011111, 0b000001, 0b000001, 0b011110, 0],
        ':' => [0, 0b001100, 0b001100, 0, 0b001100, 0b001100, 0, 0],
        '/' => [0b000001, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000, 0, 0],
        '.' => [0, 0, 0, 0, 0, 0b001100, 0b001100, 0],
        '-' => [0, 0, 0, 0b111111, 0, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 0b111111, 0],
        '<' => [0b000100, 0b001000, 0b010000, 0b100000, 0b010000, 0b001000, 0b000100, 0],
        '>' => [0b010000, 0b001000, 0b000100, 0b000010, 0b000100, 0b001000, 0b010000, 0],
        '!' => [0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0, 0b001000, 0],
        ',' => [0, 0, 0, 0, 0, 0b001100, 0b001000, 0b010000],
        '(' => [0b000100, 0b001000, 0b010000, 0b010000, 0b010000, 0b001000, 0b000100, 0],
        ')' => [0b010000, 0b001000, 0b000100, 0b000100, 0b000100, 0b001000, 0b010000, 0],
        '"' => [0b010010, 0b010010, 0b010010, 0, 0, 0, 0, 0],
        '\'' => [0b001000, 0b001000, 0b001000, 0, 0, 0, 0, 0],
        '#' => [0b010010, 0b111111, 0b010010, 0b010010, 0b111111, 0b010010, 0, 0],
        '@' => [0b011110, 0b100001, 0b101101, 0b101101, 0b101110, 0b100000, 0b011110, 0],
        '[' => [0b011100, 0b010000, 0b010000, 0b010000, 0b010000, 0b010000, 0b011100, 0],
        ']' => [0b011100, 0b000100, 0b000100, 0b000100, 0b000100, 0b000100, 0b011100, 0],
        '&' => [0b011000, 0b100100, 0b011000, 0b011010, 0b100100, 0b100010, 0b011101, 0],
        '?' => [0b011110, 0b100001, 0b000001, 0b000110, 0b001000, 0, 0b001000, 0],
        ';' => [0, 0b001100, 0b001100, 0, 0b001100, 0b001000, 0b010000, 0],
        '=' => [0, 0, 0b111111, 0, 0b111111, 0, 0, 0],
        '+' => [0, 0b001000, 0b001000, 0b111110, 0b001000, 0b001000, 0, 0],
        '|' => [0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0],

        ' ' => [0; 8],
        _ => [0b111111, 0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b111111, 0],
    }
}
