//! Advanced Ad Blocker Engine with EasyList Support
//!
//! Features:
//! - Real EasyList file parsing
//! - Domain blocking, URL pattern matching
//! - Element hiding (CSS selectors)
//! - Exception rules
//! - Per-site fingerprint randomization

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// ============================================================================
// EASYLIST PARSER
// ============================================================================

/// EasyList-compatible filter rule
#[derive(Clone, Debug)]
pub struct FilterRule {
    pub raw: String,
    pub pattern: String,
    pub is_regex: bool,
    pub is_exception: bool,
    pub domains: Option<Vec<String>>,
    pub exclude_domains: Option<Vec<String>>,
    pub third_party: Option<bool>,
    pub element_hide: bool,
    pub selector: Option<String>,
    pub block_type: BlockType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockType {
    Url,
    Domain,
    Element,
    Script,
    Image,
    Stylesheet,
    XmlHttpRequest,
    Other,
}

/// EasyList parser
pub struct EasyListParser;

impl EasyListParser {
    /// Parse an EasyList file from disk
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<FilterRule>, std::io::Error> {
        let content = fs::read_to_string(path)?;
        Ok(Self::parse(&content))
    }
    
    /// Parse EasyList content
    pub fn parse(content: &str) -> Vec<FilterRule> {
        let mut rules = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and metadata
            if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
                continue;
            }
            
            if let Some(rule) = Self::parse_rule(line) {
                rules.push(rule);
            }
        }
        
        rules
    }
    
    fn parse_rule(line: &str) -> Option<FilterRule> {
        let is_exception = line.starts_with("@@");
        let line = if is_exception { &line[2..] } else { line };
        
        // Element hiding rule: ##selector or domain##selector
        if line.contains("##") {
            return Self::parse_element_hiding(line, is_exception);
        }
        
        // URL filtering rule
        Self::parse_url_rule(line, is_exception)
    }
    
    fn parse_element_hiding(line: &str, is_exception: bool) -> Option<FilterRule> {
        let parts: Vec<&str> = line.splitn(2, "##").collect();
        
        if parts.len() == 2 {
            let domains = if parts[0].is_empty() {
                None
            } else {
                Some(parts[0].split(',').map(|s| s.to_string()).collect())
            };
            
            Some(FilterRule {
                raw: line.to_string(),
                pattern: String::new(),
                is_regex: false,
                is_exception,
                domains,
                exclude_domains: None,
                third_party: None,
                element_hide: true,
                selector: Some(parts[1].to_string()),
                block_type: BlockType::Element,
            })
        } else {
            None
        }
    }
    
    fn parse_url_rule(line: &str, is_exception: bool) -> Option<FilterRule> {
        // Check for options section
        let (pattern_part, options) = if let Some(idx) = line.rfind('$') {
            (&line[..idx], Some(&line[idx+1..]))
        } else {
            (line, None)
        };
        
        let mut rule = FilterRule {
            raw: line.to_string(),
            pattern: String::new(),
            is_regex: false,
            is_exception,
            domains: None,
            exclude_domains: None,
            third_party: None,
            element_hide: false,
            selector: None,
            block_type: BlockType::Url,
        };
        
        // Parse pattern
        if pattern_part.starts_with("||") {
            // Domain anchor
            rule.pattern = pattern_part[2..].to_string();
            rule.block_type = BlockType::Domain;
        } else if pattern_part.starts_with('|') && pattern_part.ends_with('|') {
            // Exact match
            rule.pattern = pattern_part[1..pattern_part.len()-1].to_string();
        } else if pattern_part.starts_with('/') && pattern_part.ends_with('/') {
            // Regex
            rule.pattern = pattern_part[1..pattern_part.len()-1].to_string();
            rule.is_regex = true;
        } else {
            rule.pattern = pattern_part.to_string();
        }
        
        // Parse options
        if let Some(opts) = options {
            for opt in opts.split(',') {
                match opt {
                    "third-party" | "3p" => rule.third_party = Some(true),
                    "~third-party" | "~3p" => rule.third_party = Some(false),
                    "script" => rule.block_type = BlockType::Script,
                    "image" => rule.block_type = BlockType::Image,
                    "stylesheet" => rule.block_type = BlockType::Stylesheet,
                    "xmlhttprequest" | "xhr" => rule.block_type = BlockType::XmlHttpRequest,
                    _ if opt.starts_with("domain=") => {
                        let domains_str = &opt[7..];
                        let (include, exclude): (Vec<_>, Vec<_>) = domains_str
                            .split('|')
                            .partition(|d| !d.starts_with('~'));
                        
                        if !include.is_empty() {
                            rule.domains = Some(include.iter().map(|s| s.to_string()).collect());
                        }
                        if !exclude.is_empty() {
                            rule.exclude_domains = Some(exclude.iter().map(|s| s[1..].to_string()).collect());
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Some(rule)
    }
}

// ============================================================================
// ADVANCED AD BLOCKER
// ============================================================================

/// Advanced ad blocker with EasyList support
pub struct AdvancedAdBlocker {
    /// Domain blocking rules
    domain_rules: HashSet<String>,
    
    /// URL pattern rules  
    url_rules: Vec<FilterRule>,
    
    /// Element hiding rules
    element_rules: Vec<FilterRule>,
    
    /// Exception/whitelist rules
    exceptions: Vec<FilterRule>,
    
    /// Statistics
    pub stats: BlockerStats,
    
    /// Enabled
    pub enabled: bool,
}

#[derive(Default, Debug, Clone)]
pub struct BlockerStats {
    pub total_blocked: u64,
    pub ads_blocked: u64,
    pub trackers_blocked: u64,
    pub scripts_blocked: u64,
    pub rules_loaded: usize,
}

impl AdvancedAdBlocker {
    pub fn new() -> Self {
        let mut blocker = AdvancedAdBlocker {
            domain_rules: HashSet::new(),
            url_rules: Vec::new(),
            element_rules: Vec::new(),
            exceptions: Vec::new(),
            stats: BlockerStats::default(),
            enabled: true,
        };
        
        blocker.load_builtin_rules();
        blocker
    }
    
    /// Load EasyList from file
    pub fn load_easylist<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, std::io::Error> {
        let rules = EasyListParser::parse_file(path)?;
        let count = rules.len();
        
        for rule in rules {
            self.add_rule(rule);
        }
        
        self.stats.rules_loaded += count;
        println!("[AdBlock] Loaded {} rules from EasyList", count);
        Ok(count)
    }
    
    /// Load EasyList from string content
    pub fn load_easylist_content(&mut self, content: &str) -> usize {
        let rules = EasyListParser::parse(content);
        let count = rules.len();
        
        for rule in rules {
            self.add_rule(rule);
        }
        
        self.stats.rules_loaded += count;
        count
    }
    
    fn add_rule(&mut self, rule: FilterRule) {
        if rule.is_exception {
            self.exceptions.push(rule);
        } else if rule.element_hide {
            self.element_rules.push(rule);
        } else if rule.block_type == BlockType::Domain {
            self.domain_rules.insert(rule.pattern.clone());
        } else {
            self.url_rules.push(rule);
        }
    }
    
    fn load_builtin_rules(&mut self) {
        // Built-in ad domains
        let ad_domains = [
            "doubleclick.net", "googlesyndication.com", "googleadservices.com",
            "google-analytics.com", "googletagmanager.com", "facebook.net",
            "facebook.com/tr", "amazon-adsystem.com", "adsrvr.org", "adnxs.com",
            "rubiconproject.com", "pubmatic.com", "openx.net", "taboola.com",
            "outbrain.com", "criteo.com", "criteo.net", "scorecardresearch.com",
            "quantserve.com", "moatads.com", "doubleverify.com", "chartbeat.com",
            "hotjar.com", "mixpanel.com", "segment.io", "amplitude.com",
            "fullstory.com", "mouseflow.com", "crazyegg.com", "optimizely.com",
            "branch.io", "appsflyer.com", "adjust.com", "kochava.com",
            "ads.twitter.com", "ads.linkedin.com", "ads.pinterest.com",
            "pixel.facebook.com", "connect.facebook.net", "static.ads-twitter.com",
        ];
        
        for domain in ad_domains {
            self.domain_rules.insert(domain.to_string());
        }
        
        // Built-in URL patterns
        let patterns = [
            "/ads/", "/ad/", "/advert/", "/banner/", "/sponsor/", "/tracking/",
            "/pixel/", "/beacon/", "/analytics/", "/telemetry/", "pagead",
            "doubleclick", "googleads", "adserver", "adtrack",
        ];
        
        for pattern in patterns {
            self.url_rules.push(FilterRule {
                raw: pattern.to_string(),
                pattern: pattern.to_string(),
                is_regex: false,
                is_exception: false,
                domains: None,
                exclude_domains: None,
                third_party: None,
                element_hide: false,
                selector: None,
                block_type: BlockType::Url,
            });
        }
        
        self.stats.rules_loaded = self.domain_rules.len() + self.url_rules.len();
    }
    
    /// Check if URL should be blocked
    pub fn should_block(&mut self, url: &str, page_domain: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check exceptions first
        for exc in &self.exceptions {
            if self.matches_rule(url, page_domain, exc) {
                return false;
            }
        }
        
        // Check domain rules
        if let Some(domain) = extract_domain(url) {
            for blocked in &self.domain_rules {
                if domain.ends_with(blocked) || domain == *blocked {
                    self.stats.total_blocked += 1;
                    self.stats.ads_blocked += 1;
                    return true;
                }
            }
        }
        
        // Check URL pattern rules
        let url_lower = url.to_lowercase();
        for rule in &self.url_rules {
            if self.matches_rule(&url_lower, page_domain, rule) {
                self.stats.total_blocked += 1;
                match rule.block_type {
                    BlockType::Script => self.stats.scripts_blocked += 1,
                    _ => self.stats.ads_blocked += 1,
                }
                return true;
            }
        }
        
        false
    }
    
    fn matches_rule(&self, url: &str, page_domain: &str, rule: &FilterRule) -> bool {
        // Check domain restrictions
        if let Some(ref domains) = rule.domains {
            if !domains.iter().any(|d| page_domain.ends_with(d)) {
                return false;
            }
        }
        
        if let Some(ref exclude) = rule.exclude_domains {
            if exclude.iter().any(|d| page_domain.ends_with(d)) {
                return false;
            }
        }
        
        // Match pattern
        if rule.is_regex {
            // Simplified regex matching (contains)
            url.contains(&rule.pattern)
        } else {
            url.contains(&rule.pattern)
        }
    }
    
    /// Get element hiding CSS for domain
    pub fn get_hiding_css(&self, domain: &str) -> String {
        let mut selectors = Vec::new();
        
        for rule in &self.element_rules {
            // Check domain restrictions
            if let Some(ref domains) = rule.domains {
                if !domains.iter().any(|d| domain.ends_with(d) || d == "*") {
                    continue;
                }
            }
            
            if let Some(ref selector) = rule.selector {
                selectors.push(selector.clone());
            }
        }
        
        if selectors.is_empty() {
            String::new()
        } else {
            format!("{} {{ display: none !important; }}", selectors.join(", "))
        }
    }
}

impl Default for AdvancedAdBlocker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PER-SITE FINGERPRINT RANDOMIZATION
// ============================================================================

/// Per-site fingerprint generator - each domain sees a different "browser"
pub struct FingerprintRandomizer {
    /// Cache of generated fingerprints per domain
    fingerprints: HashMap<String, SpoofedFingerprint>,
    
    /// Base seed for randomization
    base_seed: u64,
    
    /// Enabled
    pub enabled: bool,
}

/// Spoofed browser fingerprint
#[derive(Clone, Debug)]
pub struct SpoofedFingerprint {
    pub user_agent: String,
    pub platform: String,
    pub vendor: String,
    pub language: String,
    pub languages: Vec<String>,
    pub timezone: String,
    pub timezone_offset: i32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub color_depth: u8,
    pub device_memory: u8,
    pub hardware_concurrency: u8,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
    pub canvas_noise_seed: u32,
    pub audio_noise_seed: u32,
    pub do_not_track: bool,
    pub cookies_enabled: bool,
}

// User agent templates
const USER_AGENTS: &[(&str, &str, &str)] = &[
    // (User-Agent, Platform, Vendor)
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36", "Win32", "Google Inc."),
    ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36", "MacIntel", "Google Inc."),
    ("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36", "Linux x86_64", "Google Inc."),
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0", "Win32", ""),
    ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0", "MacIntel", ""),
    ("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0", "Linux x86_64", ""),
    ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15", "MacIntel", "Apple Computer, Inc."),
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0", "Win32", "Google Inc."),
];

const SCREEN_RESOLUTIONS: &[(u32, u32)] = &[
    (1920, 1080), (2560, 1440), (1366, 768), (1536, 864),
    (1440, 900), (1280, 720), (3840, 2160), (2880, 1800),
];

const TIMEZONES: &[(&str, i32)] = &[
    ("America/New_York", -300), ("America/Los_Angeles", -480),
    ("America/Chicago", -360), ("Europe/London", 0),
    ("Europe/Paris", 60), ("Europe/Berlin", 60),
    ("Asia/Tokyo", 540), ("Asia/Shanghai", 480),
    ("Australia/Sydney", 660), ("Pacific/Auckland", 780),
];

const LANGUAGES: &[&str] = &[
    "en-US", "en-GB", "de-DE", "fr-FR", "es-ES", "ja-JP", "zh-CN", "pt-BR",
];

const WEBGL_VENDORS: &[&str] = &[
    "Google Inc. (NVIDIA)", "Google Inc. (AMD)", "Google Inc. (Intel)",
    "Intel Inc.", "NVIDIA Corporation", "ATI Technologies Inc.",
];

const WEBGL_RENDERERS: &[&str] = &[
    "ANGLE (NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0)",
    "ANGLE (AMD Radeon RX 6800 XT Direct3D11 vs_5_0 ps_5_0)",
    "ANGLE (Intel(R) UHD Graphics 770 Direct3D11 vs_5_0 ps_5_0)",
    "AMD Radeon Pro 5500M OpenGL Engine",
    "Apple M1 Pro",
    "Mesa Intel(R) UHD Graphics 630",
];

impl FingerprintRandomizer {
    pub fn new() -> Self {
        // Use current time as base seed
        let base_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        FingerprintRandomizer {
            fingerprints: HashMap::new(),
            base_seed,
            enabled: true,
        }
    }
    
    /// Get or generate fingerprint for a domain
    pub fn get_fingerprint(&mut self, domain: &str) -> SpoofedFingerprint {
        if !self.enabled {
            return self.get_default_fingerprint();
        }
        
        // Normalize domain
        let normalized = Self::normalize_domain(domain);
        
        if let Some(fp) = self.fingerprints.get(&normalized) {
            return fp.clone();
        }
        
        // Generate new fingerprint based on domain hash
        let fp = self.generate_fingerprint(&normalized);
        self.fingerprints.insert(normalized, fp.clone());
        fp
    }
    
    fn normalize_domain(domain: &str) -> String {
        // Extract base domain (e.g., "www.google.com" -> "google.com")
        let domain = domain.trim_start_matches("www.");
        
        // Get effective TLD+1
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[parts.len()-2], parts[parts.len()-1])
        } else {
            domain.to_string()
        }
    }
    
    fn generate_fingerprint(&self, domain: &str) -> SpoofedFingerprint {
        // Create domain-specific seed
        let seed = self.hash_domain(domain);
        
        // Select user agent
        let ua_idx = (seed % USER_AGENTS.len() as u64) as usize;
        let (user_agent, platform, vendor) = USER_AGENTS[ua_idx];
        
        // Select screen resolution
        let screen_idx = ((seed >> 8) % SCREEN_RESOLUTIONS.len() as u64) as usize;
        let (width, height) = SCREEN_RESOLUTIONS[screen_idx];
        
        // Select timezone
        let tz_idx = ((seed >> 16) % TIMEZONES.len() as u64) as usize;
        let (timezone, tz_offset) = TIMEZONES[tz_idx];
        
        // Select language
        let lang_idx = ((seed >> 24) % LANGUAGES.len() as u64) as usize;
        let language = LANGUAGES[lang_idx];
        
        // Select WebGL info
        let gl_vendor_idx = ((seed >> 32) % WEBGL_VENDORS.len() as u64) as usize;
        let gl_renderer_idx = ((seed >> 40) % WEBGL_RENDERERS.len() as u64) as usize;
        
        // Generate noise seeds for canvas/audio
        let canvas_noise = (seed >> 48) as u32;
        let audio_noise = (seed >> 56) as u32;
        
        // Random hardware specs
        let device_memory = [4, 8, 16, 32][(seed >> 4) as usize % 4];
        let hardware_concurrency = [4, 8, 12, 16][(seed >> 6) as usize % 4];
        
        SpoofedFingerprint {
            user_agent: user_agent.to_string(),
            platform: platform.to_string(),
            vendor: vendor.to_string(),
            language: language.to_string(),
            languages: vec![language.to_string(), "en".to_string()],
            timezone: timezone.to_string(),
            timezone_offset: tz_offset,
            screen_width: width,
            screen_height: height,
            color_depth: 24,
            device_memory,
            hardware_concurrency,
            webgl_vendor: WEBGL_VENDORS[gl_vendor_idx].to_string(),
            webgl_renderer: WEBGL_RENDERERS[gl_renderer_idx].to_string(),
            canvas_noise_seed: canvas_noise,
            audio_noise_seed: audio_noise,
            do_not_track: (seed % 2) == 0,
            cookies_enabled: true,
        }
    }
    
    fn hash_domain(&self, domain: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        domain.hash(&mut hasher);
        self.base_seed.hash(&mut hasher);
        hasher.finish()
    }
    
    fn get_default_fingerprint(&self) -> SpoofedFingerprint {
        SpoofedFingerprint {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            platform: "Win32".to_string(),
            vendor: "Google Inc.".to_string(),
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string()],
            timezone: "America/New_York".to_string(),
            timezone_offset: -300,
            screen_width: 1920,
            screen_height: 1080,
            color_depth: 24,
            device_memory: 8,
            hardware_concurrency: 8,
            webgl_vendor: "Google Inc.".to_string(),
            webgl_renderer: "ANGLE".to_string(),
            canvas_noise_seed: 0,
            audio_noise_seed: 0,
            do_not_track: false,
            cookies_enabled: true,
        }
    }
    
    /// Get JavaScript to inject for fingerprint spoofing
    pub fn get_spoofing_script(&self, fingerprint: &SpoofedFingerprint) -> String {
        format!(r#"
// RustBrowser Fingerprint Protection - Each site sees different browser!
(function() {{
    'use strict';
    
    // Navigator spoofing
    const nav = {{
        userAgent: "{}",
        platform: "{}",
        vendor: "{}",
        language: "{}",
        languages: {},
        hardwareConcurrency: {},
        deviceMemory: {},
        doNotTrack: "{}",
        cookieEnabled: {}
    }};
    
    Object.defineProperty(navigator, 'userAgent', {{ get: () => nav.userAgent }});
    Object.defineProperty(navigator, 'platform', {{ get: () => nav.platform }});
    Object.defineProperty(navigator, 'vendor', {{ get: () => nav.vendor }});
    Object.defineProperty(navigator, 'language', {{ get: () => nav.language }});
    Object.defineProperty(navigator, 'languages', {{ get: () => Object.freeze([...nav.languages]) }});
    Object.defineProperty(navigator, 'hardwareConcurrency', {{ get: () => nav.hardwareConcurrency }});
    Object.defineProperty(navigator, 'deviceMemory', {{ get: () => nav.deviceMemory }});
    Object.defineProperty(navigator, 'doNotTrack', {{ get: () => nav.doNotTrack }});
    
    // Screen spoofing
    Object.defineProperty(screen, 'width', {{ get: () => {} }});
    Object.defineProperty(screen, 'height', {{ get: () => {} }});
    Object.defineProperty(screen, 'availWidth', {{ get: () => {} }});
    Object.defineProperty(screen, 'availHeight', {{ get: () => {} }});
    Object.defineProperty(screen, 'colorDepth', {{ get: () => {} }});
    Object.defineProperty(screen, 'pixelDepth', {{ get: () => {} }});
    
    // Canvas fingerprint protection with noise
    const canvasNoise = {};
    if (HTMLCanvasElement.prototype.toDataURL) {{
        const origToDataURL = HTMLCanvasElement.prototype.toDataURL;
        HTMLCanvasElement.prototype.toDataURL = function() {{
            const ctx = this.getContext('2d');
            if (ctx) {{
                const imageData = ctx.getImageData(0, 0, this.width, this.height);
                for (let i = 0; i < imageData.data.length; i += 4) {{
                    imageData.data[i] ^= ((canvasNoise + i) & 3);
                }}
                ctx.putImageData(imageData, 0, 0);
            }}
            return origToDataURL.apply(this, arguments);
        }};
    }}
    
    // WebGL fingerprint protection
    if (WebGLRenderingContext) {{
        const origGetParameter = WebGLRenderingContext.prototype.getParameter;
        WebGLRenderingContext.prototype.getParameter = function(param) {{
            if (param === 37445) return "{}"; // UNMASKED_VENDOR
            if (param === 37446) return "{}"; // UNMASKED_RENDERER
            return origGetParameter.apply(this, arguments);
        }};
    }}
    if (WebGL2RenderingContext) {{
        const origGetParameter2 = WebGL2RenderingContext.prototype.getParameter;
        WebGL2RenderingContext.prototype.getParameter = function(param) {{
            if (param === 37445) return "{}";
            if (param === 37446) return "{}";
            return origGetParameter2.apply(this, arguments);
        }};
    }}
    
    // Audio fingerprint protection
    if (AudioContext) {{
        const origCreateAnalyser = AudioContext.prototype.createAnalyser;
        AudioContext.prototype.createAnalyser = function() {{
            const analyser = origCreateAnalyser.apply(this, arguments);
            const origGetFloatFrequency = analyser.getFloatFrequencyData.bind(analyser);
            analyser.getFloatFrequencyData = function(array) {{
                origGetFloatFrequency(array);
                const noise = {};
                for (let i = 0; i < array.length; i++) {{
                    array[i] += ((noise + i) % 10) * 0.001 - 0.005;
                }}
            }};
            return analyser;
        }};
    }}
    
    // Timezone spoofing
    Date.prototype.getTimezoneOffset = function() {{ return {}; }};
    
    // Plugins spoofing (empty to look like mobile or privacy browser)
    Object.defineProperty(navigator, 'plugins', {{ get: () => [] }});
    Object.defineProperty(navigator, 'mimeTypes', {{ get: () => [] }});
    
    console.log('[RustBrowser] Fingerprint protection active - site sees: ' + nav.platform);
}})();
"#,
            fingerprint.user_agent,
            fingerprint.platform,
            fingerprint.vendor,
            fingerprint.language,
            format!("{:?}", fingerprint.languages),
            fingerprint.hardware_concurrency,
            fingerprint.device_memory,
            if fingerprint.do_not_track { "1" } else { "null" },
            fingerprint.cookies_enabled,
            fingerprint.screen_width,
            fingerprint.screen_height,
            fingerprint.screen_width,
            fingerprint.screen_height - 40,
            fingerprint.color_depth,
            fingerprint.color_depth,
            fingerprint.canvas_noise_seed,
            fingerprint.webgl_vendor,
            fingerprint.webgl_renderer,
            fingerprint.webgl_vendor,
            fingerprint.webgl_renderer,
            fingerprint.audio_noise_seed,
            fingerprint.timezone_offset
        )
    }
    
    /// Get fingerprint summary for debugging
    pub fn get_fingerprint_summary(&self, domain: &str) -> String {
        if let Some(fp) = self.fingerprints.get(&Self::normalize_domain(domain)) {
            format!(
                "Domain: {} sees {} on {} ({} cores, {}GB RAM)",
                domain, 
                fp.platform,
                if fp.user_agent.contains("Chrome") { "Chrome" }
                else if fp.user_agent.contains("Firefox") { "Firefox" }
                else if fp.user_agent.contains("Safari") { "Safari" }
                else { "Unknown" },
                fp.hardware_concurrency,
                fp.device_memory
            )
        } else {
            format!("Domain: {} (no fingerprint generated yet)", domain)
        }
    }
}

impl Default for FingerprintRandomizer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// COMBINED PRIVACY SHIELD
// ============================================================================

/// Advanced privacy shield with per-site fingerprinting
pub struct AdvancedPrivacyShield {
    pub ad_blocker: AdvancedAdBlocker,
    pub fingerprinter: FingerprintRandomizer,
    pub enabled: bool,
}

impl AdvancedPrivacyShield {
    pub fn new() -> Self {
        AdvancedPrivacyShield {
            ad_blocker: AdvancedAdBlocker::new(),
            fingerprinter: FingerprintRandomizer::new(),
            enabled: true,
        }
    }
    
    /// Load EasyList filter
    pub fn load_easylist<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, std::io::Error> {
        self.ad_blocker.load_easylist(path)
    }
    
    /// Check if request should be blocked
    pub fn should_block(&mut self, url: &str, page_domain: &str) -> bool {
        self.enabled && self.ad_blocker.should_block(url, page_domain)
    }
    
    /// Get fingerprint protection script for domain
    pub fn get_protection_script(&mut self, domain: &str) -> String {
        if !self.enabled {
            return String::new();
        }
        
        let fp = self.fingerprinter.get_fingerprint(domain);
        self.fingerprinter.get_spoofing_script(&fp)
    }
    
    /// Get stats summary
    pub fn get_stats(&self) -> String {
        format!(
            "Blocked: {} (ads: {}, scripts: {}) | Rules: {} | Sites with unique fingerprints: {}",
            self.ad_blocker.stats.total_blocked,
            self.ad_blocker.stats.ads_blocked,
            self.ad_blocker.stats.scripts_blocked,
            self.ad_blocker.stats.rules_loaded,
            self.fingerprinter.fingerprints.len()
        )
    }
    
    /// Get stats summary (alias)
    pub fn get_stats_summary(&self) -> String {
        self.get_stats()
    }
    
    /// Get total number of blocks
    pub fn total_blocks(&self) -> u64 {
        self.ad_blocker.stats.total_blocked
    }
}

impl Default for AdvancedPrivacyShield {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn extract_domain(url: &str) -> Option<String> {
    let url = url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("//");
    
    if let Some(slash_pos) = url.find('/') {
        Some(url[..slash_pos].to_string())
    } else {
        Some(url.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_randomization() {
        let mut fp = FingerprintRandomizer::new();
        
        // Same domain should get same fingerprint
        let fp1 = fp.get_fingerprint("google.com");
        let fp2 = fp.get_fingerprint("google.com");
        assert_eq!(fp1.user_agent, fp2.user_agent);
        
        // Different domains should get different fingerprints
        let fp3 = fp.get_fingerprint("youtube.com");
        // May or may not be different, but likely different
        println!("Google sees: {}", fp1.platform);
        println!("YouTube sees: {}", fp3.platform);
    }
    
    #[test]
    fn test_easylist_parsing() {
        let content = r#"
! EasyList Test
||doubleclick.net^
||googlesyndication.com^
@@||example.com^$document
##.ad-banner
example.com##.sidebar-ad
/ads/*$script,third-party
"#;
        
        let rules = EasyListParser::parse(content);
        assert!(rules.len() >= 4);
    }
}
