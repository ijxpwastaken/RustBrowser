//! Browser Core
//! 
//! Main orchestration crate for the browser engine.

pub mod video;
pub mod adblocker;

use std::collections::HashMap;
use dom::{Node, NodeRef};
use html_parser::{parse, ParseError};
use render::{Color, DisplayCommand, DisplayList};
use layout::Rect;
use js_engine::{Interpreter, execute_with_dom, create_browser_interpreter};
use style;
use base64::Engine;

/// Simple base64 decode helper
fn base64_decode(data: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .ok()
}

/// Browser engine state
pub struct Browser {
    /// Display list (paint commands)
    display_list: DisplayList,
    /// Viewport dimensions
    viewport_width: u32,
    viewport_height: u32,
    /// Cookies storage
    cookies: HashMap<String, Cookie>,
    /// Current URL
    current_url: Option<String>,
    /// Loaded images cache (URL -> RGBA pixels + dimensions)
    images: HashMap<String, ImageData>,
    /// PDF files cache (URL -> PDF data)
    pdfs: HashMap<String, Vec<u8>>,
    /// JavaScript interpreter
    js_interpreter: Interpreter,
    /// Console output from JavaScript
    js_console: Vec<String>,
    /// Navigation history
    history: Vec<String>,
    /// Current history index
    history_index: usize,
    /// Resource cache for faster loading
    resource_cache: HashMap<String, String>,
}

/// Cookie structure
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<String>,
    pub secure: bool,
    pub http_only: bool,
}

/// Cached image data
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Browser {
    /// Create a new browser instance
    pub fn new(width: u32, height: u32) -> Self {
        // Create interpreter with all APIs (DOM, React, Database, fetch)
        let js_interpreter = create_browser_interpreter();
        
        Browser {
            display_list: DisplayList::new(),
            viewport_width: width,
            viewport_height: height,
            cookies: HashMap::new(),
            current_url: None,
            images: HashMap::new(),
            pdfs: HashMap::new(),
            js_interpreter,
            js_console: Vec::new(),
            history: Vec::new(),
            history_index: 0,
            resource_cache: HashMap::new(),
        }
    }

    /// Load a URL (supports http/https via network module)
    pub fn load_url(&mut self, url: &str) -> Result<(), BrowserError> {
        println!("=== Loading URL: {} ===", url);
        
        // Add to history if it's a new navigation (not back/forward)
        if let Some(current) = &self.current_url {
            if current != url {
                // Truncate history after current index (if we're not at the end)
                if self.history_index < self.history.len() {
                    self.history.truncate(self.history_index + 1);
                }
                self.history.push(current.clone());
                self.history_index = self.history.len() - 1;
            }
        } else {
            // First navigation
            self.history.push(url.to_string());
            self.history_index = 0;
        }
        
        self.current_url = Some(url.to_string());
        
        // Check cache first for faster loading
        let url_key = url.to_string();
        let cached_content = {
            let cache_ref = &self.resource_cache;
            cache_ref.get(&url_key).cloned()
        };
        if let Some(cached) = cached_content {
            println!("[Browser] Using cached content for: {}", url);
            return self.load_html(cached);
        }
        
        // Handle redirects with improved redirect handling
        let html = match network::HttpClient::fetch_sync(url) {
            Ok(content) => {
                // Cache the content
                self.resource_cache.insert(url.to_string(), content.clone());
                content
            }
            Err(e) => {
                // Check if it's a PDF
                if url.ends_with(".pdf") || e.to_string().contains("PDF") {
                    return self.load_pdf(url);
                }
                return Err(BrowserError::NetworkError(e.to_string()));
            }
        };
        
        self.load_html(html)
    }
    
    /// Navigate back in history
    pub fn go_back(&mut self) -> Result<(), BrowserError> {
        if self.history_index > 0 {
            self.history_index -= 1;
            let url = self.history[self.history_index].clone();
            self.current_url = Some(url.clone());
            
            let html = match network::HttpClient::fetch_sync(&url) {
                Ok(content) => content,
                Err(e) => {
                    return Err(BrowserError::NetworkError(e.to_string()));
                }
            };
            
            self.load_html(html)
        } else {
            Ok(())
        }
    }
    
    /// Navigate forward in history
    pub fn go_forward(&mut self) -> Result<(), BrowserError> {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            let url = self.history[self.history_index].clone();
            self.current_url = Some(url.clone());
            
            let html = match network::HttpClient::fetch_sync(&url) {
                Ok(content) => content,
                Err(e) => {
                    return Err(BrowserError::NetworkError(e.to_string()));
                }
            };
            
            self.load_html(html)
        } else {
            Ok(())
        }
    }
    
    /// Check if back navigation is possible
    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }
    
    /// Check if forward navigation is possible
    pub fn can_go_forward(&self) -> bool {
        self.history_index < self.history.len().saturating_sub(1)
    }

    /// Load HTML content, execute scripts, and render
    pub fn load_html(&mut self, html: String) -> Result<(), BrowserError> {
        println!("=== Browser: Parsing {} bytes of HTML ===", html.len());
        
        // Parse HTML
        let doc = parse(&html).map_err(BrowserError::ParseError)?;
        
        println!("Parse complete!");
        
        // Extract and execute scripts (both inline and external)
        let (inline_scripts, external_scripts) = self.extract_scripts(&html);
        println!("Found {} inline scripts, {} external scripts", inline_scripts.len(), external_scripts.len());
        
        // Execute inline scripts first
        for (i, script) in inline_scripts.iter().enumerate() {
            println!("[JS] Executing inline script {} ({} bytes)", i + 1, script.len());
            match execute_with_dom(script, &mut self.js_interpreter) {
                Ok(_) => println!("[JS] Inline script {} completed", i + 1),
                Err(e) => println!("[JS ERROR] Inline script {}: {}", i + 1, e),
            }
        }
        
        // Load and execute external scripts
        for (i, script_url) in external_scripts.iter().enumerate() {
            println!("[JS] Loading external script {}: {}", i + 1, script_url);
            match self.load_external_script(script_url) {
                Ok(script_content) => {
                    println!("[JS] Executing external script {} ({} bytes)", i + 1, script_content.len());
                    match execute_with_dom(&script_content, &mut self.js_interpreter) {
                        Ok(_) => println!("[JS] External script {} completed", i + 1),
                        Err(e) => println!("[JS ERROR] External script {}: {}", i + 1, e),
                    }
                }
                Err(e) => println!("[JS ERROR] Failed to load external script {}: {}", i + 1, e),
            }
        }
        
        // Extract all text with formatting info
        let mut render_items: Vec<RenderItem> = Vec::new();
        
        if let Some(root) = &doc.document_element {
            self.extract_content(root, &mut render_items, RenderContext::default());
        }
        
        println!("Extracted {} render items", render_items.len());
        
        // Build display list with proper layout
        self.display_list = DisplayList::new();
        self.layout_content(&render_items);
        
        println!("Display list has {} commands", self.display_list.commands.len());
        println!("=== Page loaded! ===");
        
        Ok(())
    }
    
    /// Extract script content from HTML (returns inline scripts and external script URLs)
    fn extract_scripts(&self, html: &str) -> (Vec<String>, Vec<String>) {
        let mut inline_scripts = Vec::new();
        let mut external_scripts = Vec::new();
        let mut remaining = html;
        
        while let Some(start) = remaining.find("<script") {
            // Find the end of the opening tag
            if let Some(tag_end) = remaining[start..].find('>') {
                let tag_part = &remaining[start..start + tag_end + 1];
                let content_start = start + tag_end + 1;
                
                // Check for external script (src attribute)
                if let Some(src_start) = tag_part.find("src=") {
                    // Extract URL from src="..." or src='...'
                    let url_start = src_start + 4;
                    let quote_char = tag_part[url_start..].chars().next().unwrap_or('"');
                    if let Some(url_end) = tag_part[url_start + 1..].find(quote_char) {
                        let url = &tag_part[url_start + 1..url_start + 1 + url_end];
                        // Resolve relative URLs
                        let full_url = self.resolve_url(url);
                        external_scripts.push(full_url);
                    }
                } else {
                    // Inline script - find closing tag
                    if let Some(end) = remaining[content_start..].find("</script>") {
                        let script_content = &remaining[content_start..content_start + end];
                        let script = script_content.trim();
                        if !script.is_empty() {
                            inline_scripts.push(script.to_string());
                        }
                    }
                }
                
                // Find closing script tag to continue
                if let Some(end) = remaining[content_start..].find("</script>") {
                    remaining = &remaining[content_start + end + 9..];
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        (inline_scripts, external_scripts)
    }
    
    /// Resolve a relative URL to an absolute URL
    fn resolve_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            return url.to_string();
        }
        
        if url.starts_with("//") {
            // Protocol-relative URL
            if let Some(current) = &self.current_url {
                if current.starts_with("https://") {
                    return format!("https:{}", url);
                } else {
                    return format!("http:{}", url);
                }
            }
            return format!("https:{}", url);
        }
        
        if url.starts_with("/") {
            // Absolute path
            if let Some(current) = &self.current_url {
                if let Some(scheme_end) = current.find("://") {
                    if let Some(authority_end) = current[scheme_end + 3..].find("/") {
                        let base = &current[..scheme_end + 3 + authority_end];
                        return format!("{}{}", base, url);
                    }
                }
            }
        } else {
            // Relative path
            if let Some(current) = &self.current_url {
                if let Some(last_slash) = current.rfind('/') {
                    let base = &current[..last_slash + 1];
                    return format!("{}{}", base, url);
                }
            }
        }
        
        url.to_string()
    }
    
    /// Load an external script from a URL
    fn load_external_script(&self, url: &str) -> Result<String, BrowserError> {
        network::HttpClient::fetch_sync(url)
            .map_err(|e| BrowserError::NetworkError(e.to_string()))
    }

    /// Extract content with styling context
    fn extract_content(&mut self, node_ref: &NodeRef, items: &mut Vec<RenderItem>, ctx: RenderContext) {
        let mut stack: Vec<(NodeRef, RenderContext)> = vec![(node_ref.clone(), ctx)];
        
        while let Some((current_ref, current_ctx)) = stack.pop() {
            if let Ok(node) = current_ref.read() {
                match &*node {
                    Node::Text(text) => {
                        let content = text.content.clone();
                        if !content.trim().is_empty() {
                            items.push(RenderItem {
                                content: content.trim().to_string(),
                                item_type: current_ctx.item_type,
                                font_size: current_ctx.font_size,
                                bold: current_ctx.bold,
                                italic: current_ctx.italic,
                                link: current_ctx.link.clone(),
                                color: current_ctx.color,
                                image_url: None,
                            });
                        }
                    }
                    Node::Element(elem) => {
                        let mut child_ctx = current_ctx.clone();
                        
                        // Check for hidden elements
                        if elem.get_attribute("aria-hidden").map(|v| v == "true").unwrap_or(false) {
                            continue;
                        }
                        if elem.get_attribute("hidden").is_some() {
                            continue;
                        }
                        // Check inline style for display:none
                        if let Some(style) = elem.get_attribute("style") {
                            if style.contains("display:none") || style.contains("display: none") {
                                continue;
                            }
                            if style.contains("visibility:hidden") || style.contains("visibility: hidden") {
                                continue;
                            }
                        }
                        
                        // Update context based on element type
                        match elem.tag_name.as_str() {
                            "h1" => {
                                child_ctx.font_size = 32.0;
                                child_ctx.bold = true;
                                child_ctx.item_type = ItemType::Heading;
                            }
                            "h2" => {
                                child_ctx.font_size = 28.0;
                                child_ctx.bold = true;
                                child_ctx.item_type = ItemType::Heading;
                            }
                            "h3" => {
                                child_ctx.font_size = 24.0;
                                child_ctx.bold = true;
                                child_ctx.item_type = ItemType::Heading;
                            }
                            "h4" => {
                                child_ctx.font_size = 20.0;
                                child_ctx.bold = true;
                                child_ctx.item_type = ItemType::Heading;
                            }
                            "h5" => {
                                child_ctx.font_size = 18.0;
                                child_ctx.bold = true;
                                child_ctx.item_type = ItemType::Heading;
                            }
                            "h6" => {
                                child_ctx.font_size = 16.0;
                                child_ctx.bold = true;
                                child_ctx.item_type = ItemType::Heading;
                            }
                            "p" => {
                                child_ctx.item_type = ItemType::Paragraph;
                            }
                            "li" => {
                                child_ctx.item_type = ItemType::ListItem;
                            }
                            "a" => {
                                child_ctx.color = [0, 102, 204, 255]; // Google blue
                                if let Some(href) = elem.get_attribute("href") {
                                    child_ctx.link = Some(href.clone());
                                }
                            }
                            "span" | "div" => {
                                // Check for inline style colors
                                if let Some(style) = elem.get_attribute("style") {
                                    if style.contains("color:") {
                                        // Try to parse color
                                        if style.contains("blue") || style.contains("#00") {
                                            child_ctx.color = [0, 0, 200, 255];
                                        }
                                    }
                                }
                            }
                            "strong" | "b" => {
                                child_ctx.bold = true;
                            }
                            "em" | "i" => {
                                child_ctx.italic = true;
                            }
                            "code" | "pre" => {
                                child_ctx.font_size = 13.0;
                                child_ctx.color = [100, 100, 100, 255];
                            }
                            "blockquote" => {
                                child_ctx.italic = true;
                                child_ctx.color = [80, 80, 80, 255];
                            }
                            // Handle images
                            "img" => {
                                if let Some(src) = elem.get_attribute("src") {
                                    let alt = elem.get_attribute("alt")
                                        .map(|s| s.clone())
                                        .unwrap_or_else(|| "[Image]".to_string());
                                    
                                    // Try to load the image
                                    self.load_image_if_needed(src);
                                    
                                    items.push(RenderItem {
                                        content: alt,
                                        item_type: ItemType::Image,
                                        font_size: 14.0,
                                        bold: false,
                                        italic: false,
                                        link: None,
                                        color: [100, 100, 100, 255],
                                        image_url: Some(src.clone()),
                                    });
                                }
                                continue;
                            }
                            // Handle video element
                            "video" => {
                                let src = elem.get_attribute("src")
                                    .map(|s| s.clone())
                                    .unwrap_or_else(|| "".to_string());
                                let poster = elem.get_attribute("poster")
                                    .map(|s| s.clone());
                                
                                // Try to load video poster if available
                                if let Some(ref poster_url) = poster {
                                    self.load_image_if_needed(poster_url);
                                }
                                
                                items.push(RenderItem {
                                    content: format!("[Video: {}]", if src.is_empty() { "No source" } else { &src }),
                                    item_type: ItemType::Video,
                                    font_size: 14.0,
                                    bold: false,
                                    italic: false,
                                    link: Some(src),
                                    color: [80, 80, 80, 255],
                                    image_url: poster,
                                });
                                continue;
                            }
                            // Handle canvas placeholder
                            "canvas" => {
                                items.push(RenderItem {
                                    content: "[Canvas Element]".to_string(),
                                    item_type: ItemType::Canvas,
                                    font_size: 14.0,
                                    bold: false,
                                    italic: false,
                                    link: None,
                                    color: [80, 80, 80, 255],
                                    image_url: None,
                                });
                                continue;
                            }
                            // Handle input fields (for Google search box)
                            "input" => {
                                let input_type = elem.get_attribute("type")
                                    .map(|s| s.clone())
                                    .unwrap_or_else(|| "text".to_string());
                                let placeholder = elem.get_attribute("placeholder")
                                    .map(|s| s.clone());
                                let value = elem.get_attribute("value")
                                    .map(|s| s.clone());
                                
                                // Only show text inputs, search inputs
                                if input_type == "text" || input_type == "search" || input_type == "email" || input_type == "password" {
                                    let content = value.or(placeholder).unwrap_or_default();
                                    if !content.is_empty() {
                                        items.push(RenderItem {
                                            content: format!("[{}]", content),
                                            item_type: ItemType::Input,
                                            font_size: 14.0,
                                            bold: false,
                                            italic: false,
                                            link: None,
                                            color: [100, 100, 100, 255],
                                            image_url: None,
                                        });
                                    }
                                }
                                continue;
                            }
                            // Handle buttons
                            "button" => {
                                // Just process children as normal text
                            }
                            // Handle SVG (often used for icons on Google)
                            "svg" => {
                                // Skip inline SVGs for now, they're typically icons
                                continue;
                            }
                            // Skip script and style tags
                            "script" | "style" | "head" | "meta" | "link" | "title" | "noscript" | "template" | "iframe" => {
                                continue;
                            }
                            _ => {}
                        }
                        
                        // Add children in reverse order for correct processing
                        for child in elem.children.iter().rev() {
                            stack.push((child.clone(), child_ctx.clone()));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Load an image if not already cached
    fn load_image_if_needed(&mut self, url: &str) {
        if self.images.contains_key(url) {
            return;
        }
        
        // Handle data: URLs (base64 encoded images)
        if url.starts_with("data:") {
            if let Some(base64_data) = Self::parse_data_url(url) {
                if let Some(img_data) = Self::decode_image_data(&base64_data) {
                    self.images.insert(url.to_string(), img_data);
                    return;
                }
            }
        }
        
        // Only load http/https images
        if url.starts_with("http://") || url.starts_with("https://") {
            println!("[Browser] Loading image: {}", url);
            match network::get_or_fetch_image(url) {
                Ok(img) => {
                    self.images.insert(url.to_string(), ImageData {
                        width: img.width,
                        height: img.height,
                        pixels: img.pixels,
                    });
                    println!("[Browser] Image loaded: {}x{}", img.width, img.height);
                }
                Err(e) => {
                    eprintln!("[Browser] Failed to load image: {}", e);
                }
            }
        }
    }
    
    /// Parse a data: URL and extract the base64 data
    fn parse_data_url(url: &str) -> Option<Vec<u8>> {
        // Format: data:[<mediatype>][;base64],<data>
        if !url.starts_with("data:") {
            return None;
        }
        
        let rest = &url[5..]; // Skip "data:"
        if let Some(comma_pos) = rest.find(',') {
            let _metadata = &rest[..comma_pos];
            let data = &rest[comma_pos + 1..];
            
            // Check if base64 encoded
            if _metadata.contains("base64") {
                // Decode base64
                let decoded = base64_decode(data);
                return decoded;
            } else {
                // URL encoded or raw
                return Some(data.as_bytes().to_vec());
            }
        }
        None
    }
    
    /// Decode image data from bytes
    fn decode_image_data(data: &[u8]) -> Option<ImageData> {
        // Try to decode as image
        match image::load_from_memory(data) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                Some(ImageData {
                    width,
                    height,
                    pixels: rgba.into_raw(),
                })
            }
            Err(_) => None,
        }
    }
    
    /// Get a cached image
    pub fn get_image(&self, url: &str) -> Option<&ImageData> {
        self.images.get(url)
    }
    
    /// Layout content into display commands
    fn layout_content(&mut self, items: &[RenderItem]) {
        let margin_left = 20.0;
        let margin_top = 20.0;
        let line_height_base = 1.4;
        let max_width = (self.viewport_width as f64) - (margin_left * 2.0);
        
        let mut y = margin_top;
        let char_width = 7.0;
        
        for item in items {
            let font_size = item.font_size;
            let line_height = font_size * line_height_base;
            
            // Calculate how many chars fit per line
            let chars_per_line = (max_width / char_width) as usize;
            
            // Add spacing before headings
            if item.item_type == ItemType::Heading {
                y += line_height * 0.5;
            }
            
            // Handle images
            if item.item_type == ItemType::Image {
                if let Some(ref url) = item.image_url {
                    if let Some(img) = self.images.get(url) {
                        // Scale image to fit
                        let scale = (max_width / img.width as f64).min(1.0);
                        let display_width = (img.width as f64 * scale) as u32;
                        let display_height = (img.height as f64 * scale) as u32;
                        
                        self.display_list.push(DisplayCommand::Image {
                            data: img.pixels.clone(),
                            width: img.width,
                            height: img.height,
                            rect: Rect {
                                x: margin_left,
                                y,
                                width: display_width as f64,
                                height: display_height as f64,
                            },
                        });
                        y += display_height as f64 + 10.0;
                        continue;
                    }
                }
                // Show placeholder if image not loaded
                self.display_list.push(DisplayCommand::Text {
                    text: format!("[Image: {}]", item.content),
                    x: margin_left,
                    y,
                    color: Color { r: 100, g: 100, b: 100, a: 255 },
                    font_size: 14.0,
                    font_weight: style::FontWeight::Normal,
                    underline: false,
                });
                y += line_height;
                continue;
            }
            
            // Handle video elements
            if item.item_type == ItemType::Video {
                // Draw video placeholder with poster if available
                let video_width = 640.0;
                let video_height = 360.0;
                
                // Draw video container
                self.display_list.push(DisplayCommand::SolidColor(
                    Color { r: 20, g: 20, b: 25, a: 255 },
                    Rect { x: margin_left, y, width: video_width, height: video_height },
                ));
                
                // Draw poster image if available
                if let Some(ref poster_url) = item.image_url {
                    if let Some(img) = self.images.get(poster_url) {
                        let scale = (video_width / img.width as f64).min(video_height / img.height as f64).min(1.0);
                        let display_width = (img.width as f64 * scale) as u32;
                        let display_height = (img.height as f64 * scale) as u32;
                        
                        self.display_list.push(DisplayCommand::Image {
                            data: img.pixels.clone(),
                            width: img.width,
                            height: img.height,
                            rect: Rect {
                                x: margin_left + (video_width - display_width as f64) / 2.0,
                                y: y + (video_height - display_height as f64) / 2.0,
                                width: display_width as f64,
                                height: display_height as f64,
                            },
                        });
                    }
                }
                
                // Draw play button overlay
                self.display_list.push(DisplayCommand::Text {
                    text: "▶ PLAY".to_string(),
                    x: margin_left + video_width / 2.0 - 30.0,
                    y: y + video_height / 2.0,
                    color: Color { r: 255, g: 255, b: 255, a: 255 },
                    font_size: 18.0,
                    font_weight: style::FontWeight::Bold,
                    underline: false,
                });
                
                // Draw video info text
                if let Some(ref src) = item.link {
                    if !src.is_empty() {
                        self.display_list.push(DisplayCommand::Text {
                            text: format!("Video: {}", src),
                            x: margin_left,
                            y: y + video_height + 5.0,
                            color: Color { r: 150, g: 150, b: 150, a: 255 },
                            font_size: 12.0,
                            font_weight: style::FontWeight::Normal,
                            underline: false,
                        });
                    }
                }
                
                y += video_height + 25.0;
                continue;
            }
            
            // Handle canvas placeholders
            if item.item_type == ItemType::Canvas {
                // Draw a placeholder box
                self.display_list.push(DisplayCommand::SolidColor(
                    Color { r: 40, g: 40, b: 45, a: 255 },
                    Rect { x: margin_left, y, width: 320.0, height: 180.0 },
                ));
                self.display_list.push(DisplayCommand::Text {
                    text: item.content.clone(),
                    x: margin_left + 100.0,
                    y: y + 80.0,
                    color: Color { r: 200, g: 200, b: 200, a: 255 },
                    font_size: 14.0,
                    font_weight: style::FontWeight::Normal,
                    underline: false,
                });
                y += 190.0;
                continue;
            }
            
            // Handle input fields (search box style)
            if item.item_type == ItemType::Input {
                // Draw input box background
                self.display_list.push(DisplayCommand::SolidColor(
                    Color { r: 255, g: 255, b: 255, a: 255 },
                    Rect { x: margin_left, y, width: 500.0, height: 44.0 },
                ));
                // Draw border
                self.display_list.push(DisplayCommand::SolidColor(
                    Color { r: 200, g: 200, b: 200, a: 255 },
                    Rect { x: margin_left, y, width: 500.0, height: 1.0 },
                ));
                self.display_list.push(DisplayCommand::SolidColor(
                    Color { r: 200, g: 200, b: 200, a: 255 },
                    Rect { x: margin_left, y: y + 43.0, width: 500.0, height: 1.0 },
                ));
                // Draw placeholder text
                self.display_list.push(DisplayCommand::Text {
                    text: item.content.clone(),
                    x: margin_left + 15.0,
                    y: y + 14.0,
                    color: Color { r: 150, g: 150, b: 150, a: 255 },
                    font_size: 16.0,
                    font_weight: style::FontWeight::Normal,
                    underline: false,
                });
                y += 54.0;
                continue;
            }
            
            // Word wrap for text
            let words: Vec<&str> = item.content.split_whitespace().collect();
            let mut current_line = String::new();
            
            for word in words {
                let test_line = if current_line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", current_line, word)
                };
                
                if test_line.len() <= chars_per_line {
                    current_line = test_line;
                } else {
                    // Emit current line
                    if !current_line.is_empty() {
                        let x = if item.item_type == ItemType::ListItem {
                            margin_left + 20.0
                        } else {
                            margin_left
                        };
                        
                        self.display_list.push(DisplayCommand::Text {
                            text: current_line.clone(),
                            x,
                            y,
                            color: Color {
                                r: item.color[0],
                                g: item.color[1],
                                b: item.color[2],
                                a: item.color[3],
                            },
                            font_size,
                            font_weight: if item.bold { style::FontWeight::Bold } else { style::FontWeight::Normal },
                            underline: false,
                        });
                        y += line_height;
                    }
                    current_line = word.to_string();
                }
            }
            
            // Emit remaining line
            if !current_line.is_empty() {
                let x = if item.item_type == ItemType::ListItem {
                    margin_left + 20.0
                } else {
                    margin_left
                };
                
                // Add bullet for list items
                let text = if item.item_type == ItemType::ListItem {
                    format!("• {}", current_line)
                } else {
                    current_line
                };
                
                self.display_list.push(DisplayCommand::Text {
                    text,
                    x,
                    y,
                    color: Color {
                        r: item.color[0],
                        g: item.color[1],
                        b: item.color[2],
                        a: item.color[3],
                    },
                    font_size,
                    font_weight: if item.bold { style::FontWeight::Bold } else { style::FontWeight::Normal },
                    underline: false,
                });
                y += line_height;
            }
            
            // Add spacing after paragraphs and headings  
            if item.item_type == ItemType::Paragraph || item.item_type == ItemType::Heading {
                y += line_height * 0.5;
            }
        }
    }

    /// Set a cookie
    pub fn set_cookie(&mut self, name: &str, value: &str, domain: Option<&str>) {
        self.cookies.insert(name.to_string(), Cookie {
            name: name.to_string(),
            value: value.to_string(),
            domain: domain.map(|s| s.to_string()),
            path: None,
            expires: None,
            secure: false,
            http_only: false,
        });
    }

    /// Get a cookie
    pub fn get_cookie(&self, name: &str) -> Option<&Cookie> {
        self.cookies.get(name)
    }

    /// Get all cookies  
    pub fn get_all_cookies(&self) -> &HashMap<String, Cookie> {
        &self.cookies
    }

    /// Get the current display list for rendering
    pub fn get_display_list(&self) -> &DisplayList {
        &self.display_list
    }

    /// Update viewport size
    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }
    
    /// Get current URL
    pub fn current_url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }
    
    /// Load PDF file
    pub fn load_pdf(&mut self, url: &str) -> Result<(), BrowserError> {
        println!("[Browser] Loading PDF: {}", url);
        
        // Check cache - clone to avoid borrow issues
        let url_key = url.to_string();
        let cached_pdf = self.pdfs.get(&url_key).cloned();
        if let Some(pdf_data) = cached_pdf {
            println!("[Browser] Using cached PDF");
            return self.render_pdf(url, &pdf_data);
        }
        
        // Fetch PDF
        let pdf_data = network::HttpClient::fetch_pdf(url)
            .map_err(|e| BrowserError::NetworkError(e.to_string()))?;
        
        // Cache it
        self.pdfs.insert(url_key, pdf_data.clone());
        
        self.render_pdf(url, &pdf_data)
    }
    
    /// Render PDF content
    fn render_pdf(&mut self, url: &str, pdf_data: &[u8]) -> Result<(), BrowserError> {
        println!("[Browser] Rendering PDF: {} ({} bytes)", url, pdf_data.len());
        
        // Clear display list
        self.display_list = DisplayList::new();
        
        // Add PDF placeholder/info
        let margin_left = 20.0;
        let mut y = 20.0;
        
        // Title
        self.display_list.push(DisplayCommand::Text {
            text: format!("PDF Document: {}", url),
            x: margin_left,
            y,
            color: Color { r: 0, g: 0, b: 0, a: 255 },
            font_size: 24.0,
            font_weight: style::FontWeight::Bold,
            underline: false,
        });
        y += 40.0;
        
        // PDF info
        self.display_list.push(DisplayCommand::Text {
            text: format!("Size: {} bytes", pdf_data.len()),
            x: margin_left,
            y,
            color: Color { r: 80, g: 80, b: 80, a: 255 },
            font_size: 14.0,
            font_weight: style::FontWeight::Normal,
            underline: false,
        });
        y += 30.0;
        
        // Note about PDF viewing
        self.display_list.push(DisplayCommand::Text {
            text: "[PDF Viewer: PDF files are detected and cached. Full PDF rendering coming soon.]".to_string(),
            x: margin_left,
            y,
            color: Color { r: 100, g: 100, b: 200, a: 255 },
            font_size: 14.0,
            font_weight: style::FontWeight::Normal,
            underline: false,
        });
        
        Ok(())
    }
    
    /// Clear resource cache
    pub fn clear_cache(&mut self) {
        self.resource_cache.clear();
        self.images.clear();
        self.pdfs.clear();
        println!("[Browser] Cache cleared");
    }
}

/// Render context for styling
#[derive(Debug, Clone)]
struct RenderContext {
    item_type: ItemType,
    font_size: f64,
    bold: bool,
    italic: bool,
    link: Option<String>,
    color: [u8; 4],
}

impl Default for RenderContext {
    fn default() -> Self {
        RenderContext {
            item_type: ItemType::Text,
            font_size: 14.0,
            bold: false,
            italic: false,
            link: None,
            color: [30, 30, 35, 255],
        }
    }
}

/// Render item with styling info
#[derive(Debug, Clone)]
struct RenderItem {
    content: String,
    item_type: ItemType,
    font_size: f64,
    bold: bool,
    italic: bool,
    link: Option<String>,
    color: [u8; 4],
    image_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemType {
    Text,
    Heading,
    Paragraph,
    ListItem,
    Image,
    Video,
    Canvas,
    Input,
}

/// Browser errors
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_creation() {
        let browser = Browser::new(800, 600);
        assert!(browser.get_display_list().commands.is_empty());
    }

    #[test]
    fn test_load_html() {
        let mut browser = Browser::new(800, 600);
        let html = "<html><body><p>Hello World</p></body></html>";
        browser.load_html(html.to_string()).unwrap();
        assert!(!browser.get_display_list().commands.is_empty());
    }
    
    #[test]
    fn test_cookies() {
        let mut browser = Browser::new(800, 600);
        browser.set_cookie("session", "abc123", Some("example.com"));
        
        let cookie = browser.get_cookie("session").unwrap();
        assert_eq!(cookie.value, "abc123");
    }
}
