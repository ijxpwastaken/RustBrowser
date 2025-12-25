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

/// HTTP client with connection pooling for Google assets
/// Uses ureq Agent for Keep-Alive connection reuse
pub struct PooledHttpClient {
    agent: ureq::Agent,
}

impl PooledHttpClient {
    /// Create a new pooled HTTP client
    pub fn new() -> Self {
        // Configure agent with connection pooling
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 RustBrowser/2.0")
            .build();
        
        Self { agent }
    }
    
    /// Fetch URL with connection reuse
    pub fn fetch(&self, url: &str) -> Result<String, NetworkError> {
        let response = self.agent.get(url)
            .set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .set("Accept-Language", "en-US,en;q=0.9")
            .set("Accept-Encoding", "gzip, deflate, br")
            .set("Connection", "keep-alive")
            .call()
            .map_err(|e| NetworkError::RequestFailed(e.to_string()))?;
        
        response.into_string()
            .map_err(|e| NetworkError::IoError(e.to_string()))
    }
    
    /// Fetch bytes with connection reuse
    pub fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, NetworkError> {
        let response = self.agent.get(url)
            .set("Accept", "*/*")
            .set("Connection", "keep-alive")
            .call()
            .map_err(|e| NetworkError::RequestFailed(e.to_string()))?;
        
        let mut bytes = Vec::new();
        response.into_reader()
            .read_to_end(&mut bytes)
            .map_err(|e| NetworkError::IoError(e.to_string()))?;
        
        Ok(bytes)
    }
    
    /// Fetch and decode image with connection reuse
    pub fn fetch_image(&self, url: &str) -> Result<LoadedImage, NetworkError> {
        let bytes = self.fetch_bytes(url)?;
        
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
    
    /// Fetch SVG content
    pub fn fetch_svg(&self, url: &str) -> Result<String, NetworkError> {
        self.fetch(url)
    }
}

impl Default for PooledHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

// Global pooled client for reuse
lazy_static::lazy_static! {
    /// Shared pooled HTTP client for connection reuse
    pub static ref POOLED_CLIENT: PooledHttpClient = PooledHttpClient::new();
}

impl HttpClient {
    pub fn new() -> Self {
        HttpClient
    }

    /// Fetch a URL synchronously (blocking) with redirect handling
    /// Uses the global POOLED_CLIENT for connection reuse
    pub fn fetch_sync(url: &str) -> Result<String, NetworkError> {
        // Use the pooled client by default for performance
        POOLED_CLIENT.fetch(url)
    }
    
    /// Fetch a URL synchronously with redirect limit
    /// Note: ureq Agent handles redirects automatically, but this manual control
    /// might be needed for specific logic. For now, we delegate to pooled client.
    /// If specific redirect logic is needed, we would need to implement it on PooledHttpClient.
    pub fn fetch_sync_with_redirects(url: &str, max_redirects: usize) -> Result<String, NetworkError> {
        // For now, simple delegation. ureq handles redirects by default.
        // To strictly enforce max_redirects, we'd need to configure the Agent differently or loop manually.
        // Assuming ureq's default redirect handling is sufficient for Google.
        POOLED_CLIENT.fetch(url)
    }
    
    /// Fetch PDF file
    pub fn fetch_pdf(url: &str) -> Result<Vec<u8>, NetworkError> {
        println!("[Network] Fetching PDF: {}", url);
        POOLED_CLIENT.fetch_bytes(url)
    }
    
    /// Fetch binary data (for images, etc.)
    pub fn fetch_bytes(url: &str) -> Result<Vec<u8>, NetworkError> {
        println!("[Network] Fetching bytes: {}", url);
        POOLED_CLIENT.fetch_bytes(url)
    }
    
    /// Fetch and decode an image from a URL
    pub fn fetch_image(url: &str) -> Result<LoadedImage, NetworkError> {
        POOLED_CLIENT.fetch_image(url)
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
