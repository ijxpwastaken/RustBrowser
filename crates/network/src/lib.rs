//! Network Layer
//! 
//! This crate handles HTTP requests and resource loading.

use std::io::Read;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    
    #[error("Connection timeout")]
    Timeout,
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Image decode error: {0}")]
    ImageError(String),
}

/// Resource types that can be loaded
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceType {
    Html,
    Css,
    JavaScript,
    Image,
    Font,
    Other,
}

/// A loaded resource
#[derive(Debug)]
pub struct Resource {
    pub url: String,
    pub resource_type: ResourceType,
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

/// Loaded image data
#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data
    pub pixels: Vec<u8>,
}

/// Synchronous HTTP client for fetching resources
pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        HttpClient
    }

    /// Fetch a URL synchronously (blocking) with redirect handling
    pub fn fetch_sync(url: &str) -> Result<String, NetworkError> {
        Self::fetch_sync_with_redirects(url, 10)
    }
    
    /// Fetch a URL synchronously with redirect limit
    pub fn fetch_sync_with_redirects(url: &str, max_redirects: usize) -> Result<String, NetworkError> {
        let mut current_url = url.to_string();
        let mut redirect_count = 0;
        
        loop {
            if redirect_count > max_redirects {
                return Err(NetworkError::RequestFailed("Too many redirects".to_string()));
            }
            
            println!("[Network] Fetching: {} (redirect: {})", current_url, redirect_count);
            
            let response = ureq::get(&current_url)
                .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 RustBrowser/2.0")
                .set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
                .set("Accept-Language", "en-US,en;q=0.9")
                .set("Accept-Encoding", "gzip, deflate, br")
                .set("Connection", "keep-alive")
                .set("Upgrade-Insecure-Requests", "1")
                .call();
            
            match response {
                Ok(resp) => {
                    let status = resp.status();
                    println!("[Network] Status: {} for {}", status, current_url);
                    
                    // Handle redirects manually for better control
                    if status >= 300 && status < 400 {
                        if let Some(location) = resp.header("Location") {
                            redirect_count += 1;
                            // Resolve relative redirect URLs
                            if location.starts_with("http://") || location.starts_with("https://") {
                                current_url = location.to_string();
                            } else if location.starts_with("/") {
                                // Absolute path redirect
                                if let Some(scheme_end) = current_url.find("://") {
                                    if let Some(authority_end) = current_url[scheme_end + 3..].find("/") {
                                        let base = &current_url[..scheme_end + 3 + authority_end];
                                        current_url = format!("{}{}", base, location);
                                    } else {
                                        current_url = format!("{}{}", current_url, location);
                                    }
                                }
                            } else {
                                // Relative redirect
                                if let Some(last_slash) = current_url.rfind('/') {
                                    let base = &current_url[..last_slash + 1];
                                    current_url = format!("{}{}", base, location);
                                }
                            }
                            println!("[Network] Redirecting to: {}", current_url);
                            continue;
                        }
                    }
                    
                    // Check content type
                    let content_type = resp.header("Content-Type")
                        .unwrap_or("text/html")
                        .to_string();
                    
                    // Handle PDF files
                    if content_type.contains("application/pdf") || url.ends_with(".pdf") {
                        return Err(NetworkError::RequestFailed("PDF file detected - use fetch_pdf()".to_string()));
                    }
                    
                    let body = resp.into_string()
                        .map_err(|e| NetworkError::IoError(e.to_string()))?;
                    
                    return Ok(body);
                }
                Err(ureq::Error::Status(code, resp)) => {
                    // Handle redirect status codes
                    if code >= 300 && code < 400 {
                        if let Some(location) = resp.header("Location") {
                            redirect_count += 1;
                            if location.starts_with("http://") || location.starts_with("https://") {
                                current_url = location.to_string();
                            } else if location.starts_with("/") {
                                if let Some(scheme_end) = current_url.find("://") {
                                    if let Some(authority_end) = current_url[scheme_end + 3..].find("/") {
                                        let base = &current_url[..scheme_end + 3 + authority_end];
                                        current_url = format!("{}{}", base, location);
                                    }
                                }
                            } else {
                                if let Some(last_slash) = current_url.rfind('/') {
                                    let base = &current_url[..last_slash + 1];
                                    current_url = format!("{}{}", base, location);
                                }
                            }
                            println!("[Network] Redirecting to: {}", current_url);
                            continue;
                        }
                    }
                    return Err(NetworkError::RequestFailed(format!("HTTP {}: {}", code, resp.status_text())));
                }
                Err(e) => {
                    return Err(NetworkError::RequestFailed(e.to_string()));
                }
            }
        }
    }
    
    /// Fetch PDF file
    pub fn fetch_pdf(url: &str) -> Result<Vec<u8>, NetworkError> {
        println!("[Network] Fetching PDF: {}", url);
        Self::fetch_bytes(url)
    }
    
    /// Fetch binary data (for images, etc.)
    pub fn fetch_bytes(url: &str) -> Result<Vec<u8>, NetworkError> {
        println!("[Network] Fetching bytes: {}", url);
        
        let response = ureq::get(url)
            .set("User-Agent", "RustBrowser/1.0")
            .call()
            .map_err(|e| NetworkError::RequestFailed(e.to_string()))?;
        
        let mut bytes = Vec::new();
        response.into_reader()
            .read_to_end(&mut bytes)
            .map_err(|e| NetworkError::IoError(e.to_string()))?;
        
        Ok(bytes)
    }
    
    /// Fetch and decode an image from a URL
    pub fn fetch_image(url: &str) -> Result<LoadedImage, NetworkError> {
        let bytes = Self::fetch_bytes(url)?;
        
        // Decode image using the image crate
        let img = image::load_from_memory(&bytes)
            .map_err(|e| NetworkError::ImageError(e.to_string()))?;
        
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        
        Ok(LoadedImage {
            width,
            height,
            pixels: rgba.into_raw(),
        })
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Image cache for avoiding repeated downloads
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    static ref IMAGE_CACHE: Arc<RwLock<HashMap<String, LoadedImage>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

/// Get an image from cache or fetch it
pub fn get_or_fetch_image(url: &str) -> Result<LoadedImage, NetworkError> {
    // Check cache first
    if let Ok(cache) = IMAGE_CACHE.read() {
        if let Some(img) = cache.get(url) {
            return Ok(img.clone());
        }
    }
    
    // Fetch and cache
    let img = HttpClient::fetch_image(url)?;
    
    if let Ok(mut cache) = IMAGE_CACHE.write() {
        cache.insert(url.to_string(), img.clone());
    }
    
    Ok(img)
}
