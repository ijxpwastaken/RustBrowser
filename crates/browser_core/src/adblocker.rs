//! Ad Blocker and Tracker Prevention Module
//!
//! Implements EasyList-compatible ad blocking and privacy protection.

use std::collections::HashSet;

/// Ad blocker with filter list support
pub struct AdBlocker {
    /// URL patterns to block (exact match)
    blocked_urls: HashSet<String>,
    
    /// Domain patterns to block
    blocked_domains: HashSet<String>,
    
    /// URL patterns (wildcards)
    url_patterns: Vec<FilterRule>,
    
    /// Element hiding rules (CSS selectors)
    element_hiding: Vec<ElementHidingRule>,
    
    /// Whitelisted domains
    whitelist: HashSet<String>,
    
    /// Statistics
    pub stats: BlockerStats,
    
    /// Is blocker enabled
    pub enabled: bool,
}

/// Filter rule for URL matching
#[derive(Clone)]
pub struct FilterRule {
    pub pattern: String,
    pub is_regex: bool,
    pub domains: Option<Vec<String>>,      // Apply only to these domains
    pub exclude_domains: Option<Vec<String>>, // Don't apply to these
    pub third_party: Option<bool>,         // Block third-party only
    pub resource_types: Vec<ResourceType>,
}

/// Resource types for blocking
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceType {
    Script,
    Image,
    Stylesheet,
    Font,
    Media,
    XmlHttpRequest,
    WebSocket,
    Document,
    Subdocument,
    Ping,
    Other,
}

/// Element hiding rules
#[derive(Clone)]
pub struct ElementHidingRule {
    pub domains: Option<Vec<String>>,
    pub selector: String,
}

/// Blocker statistics
#[derive(Default, Debug)]
pub struct BlockerStats {
    pub total_blocked: u64,
    pub ads_blocked: u64,
    pub trackers_blocked: u64,
    pub scripts_blocked: u64,
    pub cookies_blocked: u64,
    pub fingerprints_prevented: u64,
}

impl AdBlocker {
    pub fn new() -> Self {
        let mut blocker = AdBlocker {
            blocked_urls: HashSet::new(),
            blocked_domains: HashSet::new(),
            url_patterns: Vec::new(),
            element_hiding: Vec::new(),
            whitelist: HashSet::new(),
            stats: BlockerStats::default(),
            enabled: true,
        };
        
        // Load default filter lists
        blocker.load_default_lists();
        
        blocker
    }
    
    /// Load default ad/tracker blocking rules
    fn load_default_lists(&mut self) {
        // Common ad domains
        let ad_domains = [
            "doubleclick.net",
            "googlesyndication.com",
            "googleadservices.com",
            "google-analytics.com",
            "googletagmanager.com",
            "facebook.net",
            "facebook.com/tr",
            "fbcdn.net/signals",
            "amazon-adsystem.com",
            "adsrvr.org",
            "adnxs.com",
            "rubiconproject.com",
            "pubmatic.com",
            "openx.net",
            "taboola.com",
            "outbrain.com",
            "criteo.com",
            "criteo.net",
            "scorecardresearch.com",
            "quantserve.com",
            "advertising.com",
            "bidswitch.net",
            "bluekai.com",
            "chartbeat.com",
            "hotjar.com",
            "mouseflow.com",
            "fullstory.com",
            "crazyegg.com",
            "mixpanel.com",
            "segment.io",
            "amplitude.com",
            "branch.io",
            "appsflyer.com",
            "adjust.com",
            "mxpnl.com",
            "omtrdc.net",
            "2o7.net",
            "demdex.net",
            "everesttech.net",
            "moatads.com",
            "adsafeprotected.com",
            "doubleverify.com",
            "serving-sys.com",
            "sizmek.com",
            "mediaplex.com",
            "tradedoubler.com",
            "awin1.com",
            "shareasale.com",
            "cj.com",
            "go2cloud.org",
            "track.adform.net",
            "adroll.com",
            "perfectaudience.com",
            "retargeter.com",
        ];
        
        for domain in ad_domains {
            self.blocked_domains.insert(domain.to_string());
        }
        
        // Common ad URL patterns
        let ad_patterns = [
            "/ads/",
            "/ad/",
            "/advert/",
            "/banner/",
            "/banners/",
            "/sponsor/",
            "/sponsored/",
            "/tracking/",
            "/pixel/",
            "/beacon/",
            "/analytics/",
            "/telemetry/",
            "/metrics/",
            "_ads_",
            "-ads-",
            "?ad=",
            "&ad=",
            "adserver",
            "adtrack",
            "adclick",
            "adview",
            "pagead",
            "doubleclick",
            "googleads",
            "googlesyndication",
            "amazon-adsystem",
            "facebook.com/tr",
            "fbevents",
            "pixel.facebook",
        ];
        
        for pattern in ad_patterns {
            self.url_patterns.push(FilterRule {
                pattern: pattern.to_string(),
                is_regex: false,
                domains: None,
                exclude_domains: None,
                third_party: None,
                resource_types: vec![],
            });
        }
        
        // Element hiding for common ad containers
        let hiding_selectors = [
            // Google ads
            ".adsbygoogle",
            "[data-ad]",
            "[data-ad-slot]",
            "#google_ads_iframe",
            ".google-ad",
            
            // Generic ad containers
            ".ad-container",
            ".ad-wrapper",
            ".ad-unit",
            ".ad-banner",
            ".advertisement",
            ".advertorial",
            "#ad-container",
            "#ad-wrapper",
            "[class*='ad-slot']",
            "[class*='ad_slot']",
            "[id*='ad-slot']",
            "[id*='ad_slot']",
            
            // Sponsored content
            ".sponsored",
            ".sponsored-content",
            ".native-ad",
            
            // Popups and modals
            ".popup-ad",
            ".modal-ad",
            "#cookie-banner",
            ".cookie-consent",
            ".gdpr-banner",
            
            // Social tracking buttons
            ".fb-like",
            ".twitter-share-button",
            ".linkedin-share",
            
            // Common annoyances
            ".newsletter-popup",
            ".subscribe-popup",
            "#newsletter-modal",
            ".pushcrew-popup",
            ".onesignal-popup",
        ];
        
        for selector in hiding_selectors {
            self.element_hiding.push(ElementHidingRule {
                domains: None,
                selector: selector.to_string(),
            });
        }
        
        println!("[AdBlocker] Loaded {} blocked domains, {} URL patterns, {} hide rules",
            self.blocked_domains.len(),
            self.url_patterns.len(),
            self.element_hiding.len()
        );
    }
    
    /// Check if a URL should be blocked
    pub fn should_block(&mut self, url: &str, page_origin: &str, resource_type: ResourceType) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check whitelist
        if self.is_whitelisted(page_origin) {
            return false;
        }
        
        // Check blocked domains
        if let Some(domain) = Self::extract_domain(url) {
            let domain_blocked = self.blocked_domains.iter()
                .any(|blocked| domain.ends_with(blocked) || domain == *blocked);
            
            if domain_blocked {
                self.stats.total_blocked += 1;
                self.record_block_type(resource_type);
                println!("[AdBlock] Blocked domain: {}", domain);
                return true;
            }
        }
        
        // Check URL patterns - collect match result first to avoid borrow conflict
        let url_lower = url.to_lowercase();
        let matched_pattern: Option<String> = {
            let mut result = None;
            for rule in &self.url_patterns {
                if url_lower.contains(&rule.pattern.to_lowercase()) {
                    // Check domain restrictions
                    if let Some(ref domains) = rule.domains {
                        let page_domain = Self::extract_domain(page_origin).unwrap_or_default();
                        if !domains.iter().any(|d| page_domain.ends_with(d)) {
                            continue;
                        }
                    }
                    
                    // Check exclusions
                    if let Some(ref exclude) = rule.exclude_domains {
                        let page_domain = Self::extract_domain(page_origin).unwrap_or_default();
                        if exclude.iter().any(|d| page_domain.ends_with(d)) {
                            continue;
                        }
                    }
                    
                    // Check third-party
                    if let Some(third_party_only) = rule.third_party {
                        let is_third_party = !Self::is_same_origin(url, page_origin);
                        if third_party_only != is_third_party {
                            continue;
                        }
                    }
                    
                    result = Some(rule.pattern.clone());
                    break;
                }
            }
            result
        };
        
        if let Some(pattern) = matched_pattern {
            self.stats.total_blocked += 1;
            self.record_block_type(resource_type);
            println!("[AdBlock] Blocked pattern: {} (matched: {})", url, pattern);
            return true;
        }
        
        false
    }
    
    fn record_block_type(&mut self, resource_type: ResourceType) {
        match resource_type {
            ResourceType::Script => self.stats.scripts_blocked += 1,
            ResourceType::Image | ResourceType::Media => self.stats.ads_blocked += 1,
            ResourceType::XmlHttpRequest | ResourceType::Ping => self.stats.trackers_blocked += 1,
            _ => {}
        }
    }
    
    /// Get element hiding CSS for a domain
    pub fn get_hiding_css(&self, domain: &str) -> String {
        if !self.enabled {
            return String::new();
        }
        
        let mut selectors = Vec::new();
        
        for rule in &self.element_hiding {
            // Check domain restrictions
            if let Some(ref domains) = rule.domains {
                if !domains.iter().any(|d| domain.ends_with(d) || d == "*") {
                    continue;
                }
            }
            selectors.push(rule.selector.clone());
        }
        
        if selectors.is_empty() {
            return String::new();
        }
        
        format!("{} {{ display: none !important; visibility: hidden !important; }}", 
            selectors.join(", "))
    }
    
    /// Whitelist a domain
    pub fn whitelist_domain(&mut self, domain: &str) {
        self.whitelist.insert(domain.to_string());
    }
    
    /// Remove domain from whitelist
    pub fn remove_whitelist(&mut self, domain: &str) {
        self.whitelist.remove(domain);
    }
    
    /// Check if domain is whitelisted
    pub fn is_whitelisted(&self, url: &str) -> bool {
        if let Some(domain) = Self::extract_domain(url) {
            for whitelisted in &self.whitelist {
                if domain.ends_with(whitelisted) || domain == *whitelisted {
                    return true;
                }
            }
        }
        false
    }
    
    /// Add a custom filter rule
    pub fn add_filter(&mut self, pattern: &str) {
        // Simple EasyList-like syntax
        if pattern.starts_with("||") {
            // Domain anchor
            let domain = pattern.trim_start_matches("||");
            self.blocked_domains.insert(domain.to_string());
        } else if pattern.starts_with("@@") {
            // Whitelist rule
            let domain = pattern.trim_start_matches("@@||");
            self.whitelist.insert(domain.to_string());
        } else if pattern.starts_with("##") {
            // Element hiding
            let selector = pattern.trim_start_matches("##");
            self.element_hiding.push(ElementHidingRule {
                domains: None,
                selector: selector.to_string(),
            });
        } else {
            // URL pattern
            self.url_patterns.push(FilterRule {
                pattern: pattern.to_string(),
                is_regex: false,
                domains: None,
                exclude_domains: None,
                third_party: None,
                resource_types: vec![],
            });
        }
    }
    
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
    
    fn is_same_origin(url1: &str, url2: &str) -> bool {
        let d1 = Self::extract_domain(url1).unwrap_or_default();
        let d2 = Self::extract_domain(url2).unwrap_or_default();
        d1 == d2
    }
}

impl Default for AdBlocker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TRACKER PREVENTION
// ============================================================================

/// Tracker prevention with fingerprint protection
pub struct TrackerPrevention {
    /// Known tracking cookies
    tracking_cookies: HashSet<String>,
    
    /// Known fingerprinting scripts
    fingerprint_scripts: HashSet<String>,
    
    /// Cookie whitelist
    cookie_whitelist: HashSet<String>,
    
    /// Spoof canvas fingerprint
    pub spoof_canvas: bool,
    
    /// Spoof audio fingerprint  
    pub spoof_audio: bool,
    
    /// Spoof WebGL fingerprint
    pub spoof_webgl: bool,
    
    /// Block third-party cookies
    pub block_third_party_cookies: bool,
    
    /// Statistics
    pub stats: TrackerStats,
    
    /// Enabled
    pub enabled: bool,
}

#[derive(Default, Debug)]
pub struct TrackerStats {
    pub cookies_blocked: u64,
    pub fingerprints_blocked: u64,
    pub trackers_detected: u64,
}

impl TrackerPrevention {
    pub fn new() -> Self {
        let mut tp = TrackerPrevention {
            tracking_cookies: HashSet::new(),
            fingerprint_scripts: HashSet::new(),
            cookie_whitelist: HashSet::new(),
            spoof_canvas: true,
            spoof_audio: true,
            spoof_webgl: true,
            block_third_party_cookies: true,
            stats: TrackerStats::default(),
            enabled: true,
        };
        
        tp.load_defaults();
        tp
    }
    
    fn load_defaults(&mut self) {
        // Known tracking cookies
        let tracking_cookies = [
            "_ga", "_gid", "_gat", // Google Analytics
            "_fbp", "_fbc",       // Facebook
            "_gcl_au",            // Google Ads
            "__gads", "__gpi",    // Google Ads
            "IDE", "1P_JAR",      // Google
            "NID", "CONSENT",     // Google
            "_uetsid", "_uetvid", // Microsoft
            "MUID",               // Microsoft
            "_rdt_uuid",          // Reddit
            "_pin_unauth",        // Pinterest
            "li_gc", "li_mc",     // LinkedIn
            "_tt_enable_cookie", "tt_webid", // TikTok
            "sp_t", "sp_landing", // Spotify
        ];
        
        for cookie in tracking_cookies {
            self.tracking_cookies.insert(cookie.to_string());
        }
        
        // Known fingerprinting scripts
        let fp_scripts = [
            "fingerprintjs",
            "clientjs",
            "evercookie",
            "super-cookie",
            "canvas-fingerprint",
            "webgl-fingerprint",
            "audio-fingerprint",
        ];
        
        for script in fp_scripts {
            self.fingerprint_scripts.insert(script.to_string());
        }
    }
    
    /// Check if cookie should be blocked
    pub fn should_block_cookie(&mut self, name: &str, domain: &str, is_third_party: bool) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check whitelist
        if self.cookie_whitelist.contains(domain) {
            return false;
        }
        
        // Block third-party cookies
        if is_third_party && self.block_third_party_cookies {
            self.stats.cookies_blocked += 1;
            println!("[Tracker Prevention] Blocked third-party cookie: {} from {}", name, domain);
            return true;
        }
        
        // Block known tracking cookies
        if self.tracking_cookies.contains(name) {
            self.stats.cookies_blocked += 1;
            self.stats.trackers_detected += 1;
            println!("[Tracker Prevention] Blocked tracking cookie: {}", name);
            return true;
        }
        
        false
    }
    
    /// Check if script appears to be fingerprinting
    pub fn is_fingerprinting_script(&mut self, script_content: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        let content_lower = script_content.to_lowercase();
        
        // Check for known fingerprinting libraries
        for fp_script in &self.fingerprint_scripts {
            if content_lower.contains(fp_script) {
                self.stats.fingerprints_blocked += 1;
                return true;
            }
        }
        
        // Check for fingerprinting patterns
        let fingerprint_patterns = [
            "getimagedata",
            "todataurl",
            "webglrenderingcontext",
            "audiocontext",
            "oscnode",
            "createanalyser",
            "getextension",
            "getparameter",
            "gl_vendor",
            "gl_renderer",
            "createofflineaudiocontext",
        ];
        
        for pattern in fingerprint_patterns {
            if content_lower.contains(pattern) {
                self.stats.fingerprints_blocked += 1;
                return true;
            }
        }
        
        false
    }
    
    /// Get JavaScript to inject for fingerprint spoofing
    pub fn get_spoofing_script(&self) -> &'static str {
        if !self.enabled {
            return "";
        }
        
        r#"
        // Canvas fingerprint protection
        if (typeof HTMLCanvasElement !== 'undefined') {
            const origToDataURL = HTMLCanvasElement.prototype.toDataURL;
            HTMLCanvasElement.prototype.toDataURL = function() {
                const context = this.getContext('2d');
                if (context) {
                    const imageData = context.getImageData(0, 0, this.width, this.height);
                    // Add noise to prevent fingerprinting
                    for (let i = 0; i < imageData.data.length; i += 4) {
                        imageData.data[i] ^= (Math.random() * 2) | 0;
                    }
                    context.putImageData(imageData, 0, 0);
                }
                return origToDataURL.apply(this, arguments);
            };
        }
        
        // Audio fingerprint protection
        if (typeof AudioContext !== 'undefined') {
            const origCreateAnalyser = AudioContext.prototype.createAnalyser;
            AudioContext.prototype.createAnalyser = function() {
                const analyser = origCreateAnalyser.apply(this, arguments);
                const origGetFloatFrequencyData = analyser.getFloatFrequencyData.bind(analyser);
                analyser.getFloatFrequencyData = function(array) {
                    origGetFloatFrequencyData(array);
                    for (let i = 0; i < array.length; i++) {
                        array[i] += Math.random() * 0.1 - 0.05;
                    }
                };
                return analyser;
            };
        }
        
        // WebGL fingerprint protection
        if (typeof WebGLRenderingContext !== 'undefined') {
            const origGetParameter = WebGLRenderingContext.prototype.getParameter;
            WebGLRenderingContext.prototype.getParameter = function(param) {
                // Spoof vendor and renderer strings
                if (param === 37445) return 'Generic'; // UNMASKED_VENDOR_WEBGL
                if (param === 37446) return 'Generic GPU'; // UNMASKED_RENDERER_WEBGL
                return origGetParameter.apply(this, arguments);
            };
        }
        "#
    }
    
    /// Whitelist a domain for cookies
    pub fn whitelist_cookies(&mut self, domain: &str) {
        self.cookie_whitelist.insert(domain.to_string());
    }
}

impl Default for TrackerPrevention {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// COMBINED PRIVACY SHIELD
// ============================================================================

/// Combined ad blocker and tracker prevention
pub struct PrivacyShield {
    pub ad_blocker: AdBlocker,
    pub tracker_prevention: TrackerPrevention,
    pub enabled: bool,
}

impl PrivacyShield {
    pub fn new() -> Self {
        PrivacyShield {
            ad_blocker: AdBlocker::new(),
            tracker_prevention: TrackerPrevention::new(),
            enabled: true,
        }
    }
    
    /// Check if a request should be blocked
    pub fn should_block_request(&mut self, url: &str, origin: &str, resource_type: ResourceType) -> bool {
        if !self.enabled {
            return false;
        }
        self.ad_blocker.should_block(url, origin, resource_type)
    }
    
    /// Get total blocks
    pub fn total_blocks(&self) -> u64 {
        self.ad_blocker.stats.total_blocked + self.tracker_prevention.stats.cookies_blocked
    }
    
    /// Get combined statistics summary
    pub fn get_stats_summary(&self) -> String {
        format!(
            "Blocked: {} ads, {} trackers, {} cookies, {} fingerprints",
            self.ad_blocker.stats.ads_blocked,
            self.ad_blocker.stats.trackers_blocked,
            self.tracker_prevention.stats.cookies_blocked,
            self.tracker_prevention.stats.fingerprints_blocked
        )
    }
}

impl Default for PrivacyShield {
    fn default() -> Self {
        Self::new()
    }
}
