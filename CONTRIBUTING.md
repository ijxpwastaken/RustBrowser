# Contributing to RustBrowser

We love your input! We want to make contributing to RustBrowser as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## 🚀 Getting Started

1.  **Fork the repo** and clone it locally.
2.  **Install Rust**: Ensure you have the latest stable Rust installed (`rustup update`).
3.  **Build the project**:
    ```bash
    cargo build
    ```
4.  **Run the browser**:
    ```bash
    cargo run --bin browser
    ```
5.  **Run tests**:
    ```bash
    cargo test
    ```

## 📂 Project Structure

- `src/`: Main application logic (`winit` event loop).
- `crates/`: Modular components.
    - `browser_core`: The heart of the browser (state, orchestration).
    - `html_parser`: Tokenizer and DOM tree builder.
    - `css_engine`: **(WIP)** CSS parsing and selector matching.
    - `js_engine`: TypeScript/JavaScript runtime (Boa wrapper).
    - `render`: Painting logic (Softbuffer).
    - `network`: HTTP requests and caching.

## 🟢 Good First Issues

If you're new to the project, here are some great places to start:

### 1. Improve CSS Parsing
**Difficulty**: Easy/Medium
**Location**: `crates/style/src/parser.rs`
**Task**: Currently, the CSS parser only handles basic selectors. We need help implementing:
- Class selectors (`.classname`)
- ID selectors (`#idname`)
- Basic property parsing (e.g., `margin: 10px;`)

### 2. Update Adblocker Lists
**Difficulty**: Easy
**Location**: `crates/browser_core/src/adblocker.rs`
**Task**: The adblocker uses a static list. Implement a feature to:
- download the latest EasyList.txt at startup
- Cache it locally
- Parse it more efficiently

### 3. Add Keyboard Shortcuts
**Difficulty**: Easy
**Location**: `src/main.rs`
**Task**: Add support for:
- `Ctrl+T` (New Tab)
- `Ctrl+W` (Close Tab)
- `Ctrl+R` (Reload)
- `Ctrl+L` (Focus Address Bar)

### 4. Implement Bookmarks System
**Difficulty**: Medium
**Location**: `crates/browser_core/src/lib.rs`
**Task**: Create a simple bookmarking system:
- Add a `bookmarks: Vec<String>` to the Browser struct
- Persist bookmarks to a JSON file on exit
- Add a UI button to "Star" the current page

### 5. Add "Find in Page"
**Difficulty**: Medium
**Location**: `src/main.rs`, `crates/render/src/lib.rs`
**Task**: Implement a simple text search:
- UI popup for search term
- Highlight matching text in the `DisplayList`
- Jump to next match

## 🎨 GitHub Topics

If you are a maintainer, please add these topics to the repository to help others find us:
`rust`, `browser`, `privacy`, `react`, `rendering-engine`, `good-first-issue`

## 📝 License
By contributing, you agree that your contributions will be licensed under the project's [UNLICENSE](UNLICENSE).
