RustBrowser - Ultra-Fast Privacy-First Browser
==============================================

A browser built from scratch in Rust. Zero Chrome/Firefox code. 100% memory-safe. Blazing fast.

🚀 SPEED FEATURES:
- Real HTTP fetch with connection pooling (via ureq)
- LRU response caching (500 entries, 5MB max per entry)
- Cache-Control header parsing for smart cache TTL
- Single-origin optimization for React/Next.js apps
- Zero-copy JSON parsing with serde

🛡️ PRIVACY SHIELD:
- Built-in adblocker with 50+ blocked ad domains
- EasyList-compatible filter syntax
- Tracker prevention with cookie blocking
- Fingerprint protection (Canvas, Audio, WebGL spoofing)
- Third-party cookie blocking by default
- Element hiding CSS rules

🎨 MODERN UI:
- Premium dark theme with indigo accents
- Tabbed browsing interface
- Rounded corners and glassmorphism effects
- Shield icon showing blocked ads/trackers count
- Navigation buttons with hover effects

🔒 SECURITY:
- Zero Legacy Bloat: Starting fresh avoids 30 years of C++ vulnerabilities
- 100% Safe: Rust's memory safety prevents buffer overflows and use-after-free
- Content Security Policy (CSP) enforcement
- Domain whitelisting for sensitive APIs
- Native privacy - no tracking, no telemetry

💡 CONTRIBUTING:
Free to contribute! Just started as a learning project but it's becoming real.
Focus areas: JS engine optimization, CSS parser, Web APIs.