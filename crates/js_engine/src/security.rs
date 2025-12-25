//! Security Module for JS Engine
//!
//! Implements Content Security Policy (CSP), domain whitelisting,
//! and API access controls.

use std::collections::HashSet;

/// Content Security Policy for the JS engine
#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    /// Allowed script sources (domains)
    pub script_src: HashSet<String>,
    
    /// Allowed connect sources (for fetch/XHR)
    pub connect_src: HashSet<String>,
    
    /// Allowed frame sources
    pub frame_src: HashSet<String>,
    
    /// Allowed image sources
    pub img_src: HashSet<String>,
    
    /// Allowed style sources
    pub style_src: HashSet<String>,
    
    /// Allowed font sources
    pub font_src: HashSet<String>,
    
    /// Allowed media sources
    pub media_src: HashSet<String>,
    
    /// Allow inline scripts
    pub allow_inline_scripts: bool,
    
    /// Allow eval()
    pub allow_eval: bool,
    
    /// Allow data: URLs
    pub allow_data_urls: bool,
    
    /// Allow blob: URLs
    pub allow_blob_urls: bool,
    
    /// Block all third-party scripts by default
    pub block_third_party: bool,
    
    /// Report-only mode (log violations instead of blocking)
    pub report_only: bool,
    
    /// CSP violation log
    pub violations: Vec<CspViolation>,
}

/// CSP violation record
#[derive(Debug, Clone)]
pub struct CspViolation {
    pub directive: String,
    pub blocked_uri: String,
    pub source_file: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub timestamp: u64,
}

impl ContentSecurityPolicy {
    /// Create a strict CSP that blocks third-party content
    pub fn strict() -> Self {
        ContentSecurityPolicy {
            script_src: HashSet::from(["'self'".to_string()]),
            connect_src: HashSet::from(["'self'".to_string()]),
            frame_src: HashSet::new(), // Block all frames
            img_src: HashSet::from(["'self'".to_string(), "data:".to_string()]),
            style_src: HashSet::from(["'self'".to_string(), "'unsafe-inline'".to_string()]),
            font_src: HashSet::from(["'self'".to_string()]),
            media_src: HashSet::from(["'self'".to_string()]),
            allow_inline_scripts: false,
            allow_eval: false,
            allow_data_urls: false,
            allow_blob_urls: false,
            block_third_party: true,
            report_only: false,
            violations: Vec::new(),
        }
    }
    
    /// Create a permissive CSP (for development)
    pub fn permissive() -> Self {
        ContentSecurityPolicy {
            script_src: HashSet::from(["*".to_string()]),
            connect_src: HashSet::from(["*".to_string()]),
            frame_src: HashSet::from(["*".to_string()]),
            img_src: HashSet::from(["*".to_string()]),
            style_src: HashSet::from(["*".to_string()]),
            font_src: HashSet::from(["*".to_string()]),
            media_src: HashSet::from(["*".to_string()]),
            allow_inline_scripts: true,
            allow_eval: true,
            allow_data_urls: true,
            allow_blob_urls: true,
            block_third_party: false,
            report_only: false,
            violations: Vec::new(),
        }
    }
    
    /// Check if a script source is allowed
    pub fn is_script_allowed(&self, source: &str, page_origin: &str) -> bool {
        if self.block_third_party && !self.is_same_origin(source, page_origin) {
            return self.log_violation("script-src", source);
        }
        self.check_source(&self.script_src, source)
    }
    
    /// Check if a connect source (fetch/XHR) is allowed
    pub fn is_connect_allowed(&self, source: &str, page_origin: &str) -> bool {
        if self.block_third_party && !self.is_same_origin(source, page_origin) {
            return self.log_violation("connect-src", source);
        }
        self.check_source(&self.connect_src, source)
    }
    
    /// Check if inline script is allowed
    pub fn is_inline_script_allowed(&self) -> bool {
        if !self.allow_inline_scripts {
            return self.log_violation("script-src", "'inline'");
        }
        true
    }
    
    /// Check if eval is allowed
    pub fn is_eval_allowed(&self) -> bool {
        if !self.allow_eval {
            return self.log_violation("script-src", "'eval'");
        }
        true
    }
    
    /// Add an allowed script source
    pub fn allow_script_src(&mut self, source: &str) {
        self.script_src.insert(source.to_string());
    }
    
    /// Add an allowed connect source
    pub fn allow_connect_src(&mut self, source: &str) {
        self.connect_src.insert(source.to_string());
    }
    
    fn check_source(&self, allowed: &HashSet<String>, source: &str) -> bool {
        if allowed.contains("*") {
            return true;
        }
        if allowed.contains("'self'") {
            // Would need page origin context
            return true;
        }
        if source.starts_with("data:") && self.allow_data_urls {
            return true;
        }
        if source.starts_with("blob:") && self.allow_blob_urls {
            return true;
        }
        
        // Check explicit domains
        for allowed_src in allowed {
            if allowed_src == source || source.ends_with(allowed_src) {
                return true;
            }
        }
        
        self.log_violation("script-src", source)
    }
    
    fn is_same_origin(&self, source: &str, page_origin: &str) -> bool {
        // Extract origin from URL
        if let Some(origin) = Self::extract_origin(source) {
            if let Some(page_org) = Self::extract_origin(page_origin) {
                return origin == page_org;
            }
        }
        false
    }
    
    fn extract_origin(url: &str) -> Option<String> {
        // Simple origin extraction
        if let Some(start) = url.find("://") {
            let rest = &url[start + 3..];
            if let Some(end) = rest.find('/') {
                return Some(url[..start + 3 + end].to_string());
            }
            return Some(url.to_string());
        }
        None
    }
    
    fn log_violation(&self, directive: &str, blocked_uri: &str) -> bool {
        println!("[CSP Violation] {}: blocked {}", directive, blocked_uri);
        // In report-only mode, log but allow
        self.report_only
    }
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self::strict()
    }
}

// ============================================================================
// DOMAIN WHITELIST FOR PRIVATE APIS
// ============================================================================

/// Domain whitelist for controlling access to sensitive APIs
#[derive(Debug, Clone)]
pub struct ApiAccessControl {
    /// Domains allowed to use database APIs
    pub database_whitelist: HashSet<String>,
    
    /// Domains allowed to use IndexedDB
    pub indexeddb_whitelist: HashSet<String>,
    
    /// Domains allowed to use local storage
    pub storage_whitelist: HashSet<String>,
    
    /// Domains allowed to use fetch
    pub fetch_whitelist: HashSet<String>,
    
    /// Domains allowed to use WebSockets
    pub websocket_whitelist: HashSet<String>,
    
    /// Domains allowed to access clipboard
    pub clipboard_whitelist: HashSet<String>,
    
    /// Domains allowed to access geolocation
    pub geolocation_whitelist: HashSet<String>,
    
    /// Domains allowed to access camera/microphone
    pub media_devices_whitelist: HashSet<String>,
    
    /// Deny all by default (most secure)
    pub deny_by_default: bool,
}

impl ApiAccessControl {
    /// Create a minimal access control (deny most APIs)
    pub fn minimal() -> Self {
        ApiAccessControl {
            database_whitelist: HashSet::new(),
            indexeddb_whitelist: HashSet::new(),
            storage_whitelist: HashSet::new(),
            fetch_whitelist: HashSet::from(["'self'".to_string()]),
            websocket_whitelist: HashSet::new(),
            clipboard_whitelist: HashSet::new(),
            geolocation_whitelist: HashSet::new(),
            media_devices_whitelist: HashSet::new(),
            deny_by_default: true,
        }
    }
    
    /// Create full access (for trusted domains)
    pub fn full_access() -> Self {
        ApiAccessControl {
            database_whitelist: HashSet::from(["*".to_string()]),
            indexeddb_whitelist: HashSet::from(["*".to_string()]),
            storage_whitelist: HashSet::from(["*".to_string()]),
            fetch_whitelist: HashSet::from(["*".to_string()]),
            websocket_whitelist: HashSet::from(["*".to_string()]),
            clipboard_whitelist: HashSet::from(["*".to_string()]),
            geolocation_whitelist: HashSet::from(["*".to_string()]),
            media_devices_whitelist: HashSet::from(["*".to_string()]),
            deny_by_default: false,
        }
    }
    
    /// Check if domain can access database API
    pub fn can_access_database(&self, origin: &str) -> bool {
        self.check_whitelist(&self.database_whitelist, origin, "database")
    }
    
    /// Check if domain can access IndexedDB
    pub fn can_access_indexeddb(&self, origin: &str) -> bool {
        self.check_whitelist(&self.indexeddb_whitelist, origin, "indexeddb")
    }
    
    /// Check if domain can access localStorage/sessionStorage
    pub fn can_access_storage(&self, origin: &str) -> bool {
        self.check_whitelist(&self.storage_whitelist, origin, "storage")
    }
    
    /// Check if domain can use fetch
    pub fn can_access_fetch(&self, origin: &str) -> bool {
        self.check_whitelist(&self.fetch_whitelist, origin, "fetch")
    }
    
    /// Check if domain can use WebSockets
    pub fn can_access_websocket(&self, origin: &str) -> bool {
        self.check_whitelist(&self.websocket_whitelist, origin, "websocket")
    }
    
    /// Check if domain can access clipboard
    pub fn can_access_clipboard(&self, origin: &str) -> bool {
        self.check_whitelist(&self.clipboard_whitelist, origin, "clipboard")
    }
    
    /// Whitelist a domain for database access
    pub fn allow_database(&mut self, domain: &str) {
        self.database_whitelist.insert(domain.to_string());
    }
    
    /// Whitelist a domain for IndexedDB access
    pub fn allow_indexeddb(&mut self, domain: &str) {
        self.indexeddb_whitelist.insert(domain.to_string());
    }
    
    /// Whitelist a domain for storage access
    pub fn allow_storage(&mut self, domain: &str) {
        self.storage_whitelist.insert(domain.to_string());
    }
    
    /// Whitelist a domain for all APIs
    pub fn allow_all_apis(&mut self, domain: &str) {
        self.database_whitelist.insert(domain.to_string());
        self.indexeddb_whitelist.insert(domain.to_string());
        self.storage_whitelist.insert(domain.to_string());
        self.fetch_whitelist.insert(domain.to_string());
        self.websocket_whitelist.insert(domain.to_string());
        self.clipboard_whitelist.insert(domain.to_string());
        self.geolocation_whitelist.insert(domain.to_string());
        self.media_devices_whitelist.insert(domain.to_string());
    }
    
    fn check_whitelist(&self, whitelist: &HashSet<String>, origin: &str, api_name: &str) -> bool {
        if whitelist.contains("*") {
            return true;
        }
        if whitelist.contains(origin) {
            return true;
        }
        
        // Check for wildcard subdomains
        for allowed in whitelist {
            if allowed.starts_with("*.") {
                let domain = &allowed[2..];
                if origin.ends_with(domain) {
                    return true;
                }
            }
        }
        
        if self.deny_by_default {
            println!("[API Access Denied] {} access blocked for {}", api_name, origin);
            return false;
        }
        
        true
    }
}

impl Default for ApiAccessControl {
    fn default() -> Self {
        Self::minimal()
    }
}

// ============================================================================
// SECURITY CONTEXT
// ============================================================================

/// Combined security context for script execution
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Content Security Policy
    pub csp: ContentSecurityPolicy,
    
    /// API access controls
    pub api_access: ApiAccessControl,
    
    /// Current page origin
    pub origin: Option<String>,
    
    /// Is this a secure context (HTTPS)
    pub is_secure_context: bool,
    
    /// Is this a first-party context
    pub is_first_party: bool,
    
    /// Sandbox flags
    pub sandbox: SandboxFlags,
}

/// Sandbox flags (similar to iframe sandbox)
#[derive(Debug, Clone, Default)]
pub struct SandboxFlags {
    pub allow_forms: bool,
    pub allow_modals: bool,
    pub allow_orientation_lock: bool,
    pub allow_pointer_lock: bool,
    pub allow_popups: bool,
    pub allow_popups_to_escape_sandbox: bool,
    pub allow_presentation: bool,
    pub allow_same_origin: bool,
    pub allow_scripts: bool,
    pub allow_top_navigation: bool,
    pub allow_storage_access_by_user_activation: bool,
}

impl SecurityContext {
    /// Create a strict security context
    pub fn strict(origin: &str) -> Self {
        SecurityContext {
            csp: ContentSecurityPolicy::strict(),
            api_access: ApiAccessControl::minimal(),
            origin: Some(origin.to_string()),
            is_secure_context: origin.starts_with("https://"),
            is_first_party: true,
            sandbox: SandboxFlags::default(),
        }
    }
    
    /// Create a permissive security context (for trusted sites)
    pub fn trusted(origin: &str) -> Self {
        let mut ctx = Self::strict(origin);
        ctx.api_access = ApiAccessControl::full_access();
        ctx.csp = ContentSecurityPolicy::permissive();
        ctx.sandbox.allow_scripts = true;
        ctx.sandbox.allow_forms = true;
        ctx.sandbox.allow_storage_access_by_user_activation = true;
        ctx
    }
    
    /// Check if script from given URL can be executed
    pub fn can_execute_script(&self, script_url: &str) -> bool {
        let origin = self.origin.as_deref().unwrap_or("");
        self.csp.is_script_allowed(script_url, origin) && self.sandbox.allow_scripts
    }
    
    /// Check if inline script can be executed  
    pub fn can_execute_inline_script(&self) -> bool {
        self.csp.is_inline_script_allowed() && self.sandbox.allow_scripts
    }
    
    /// Check if eval() can be called
    pub fn can_execute_eval(&self) -> bool {
        self.csp.is_eval_allowed() && self.sandbox.allow_scripts
    }
    
    /// Check if current context can access database APIs
    pub fn can_access_database(&self) -> bool {
        let origin = self.origin.as_deref().unwrap_or("");
        self.api_access.can_access_database(origin)
    }
    
    /// Check if current context can access storage
    pub fn can_access_storage(&self) -> bool {
        let origin = self.origin.as_deref().unwrap_or("");
        self.api_access.can_access_storage(origin) 
            && self.sandbox.allow_storage_access_by_user_activation
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::strict("about:blank")
    }
}

// ============================================================================
// TRUSTED DOMAINS
// ============================================================================

/// List of trusted first-party domains
pub static TRUSTED_DOMAINS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    // Add your own trusted domains here
];

/// Check if a domain is in the trusted list
pub fn is_trusted_domain(domain: &str) -> bool {
    for trusted in TRUSTED_DOMAINS {
        if domain == *trusted || domain.ends_with(&format!(".{}", trusted)) {
            return true;
        }
    }
    false
}

/// Create appropriate security context based on origin
pub fn create_security_context(origin: &str) -> SecurityContext {
    if let Some(domain) = ContentSecurityPolicy::extract_origin(origin) {
        if is_trusted_domain(&domain) {
            return SecurityContext::trusted(origin);
        }
    }
    SecurityContext::strict(origin)
}
