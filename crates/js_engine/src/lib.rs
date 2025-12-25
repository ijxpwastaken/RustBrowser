//! JavaScript Engine
//! 
//! A basic JavaScript interpreter for the browser.
//! Supports: variables, functions, console.log, basic DOM access, React, Database

pub mod tokenizer;
pub mod parser;
pub mod ast;
pub mod interpreter;
pub mod value;
pub mod builtins;
pub mod dom_bridge;
pub mod react;
pub mod database;
pub mod nextjs;
pub mod security;

pub use tokenizer::{Tokenizer, Token, TokenType};
pub use parser::Parser;
pub use ast::*;
pub use interpreter::Interpreter;
pub use value::JsValue;
pub use dom_bridge::DomBridge;
pub use react::{JsxParser, ReactRuntime, create_react_object, create_react_dom_object};
pub use database::{create_database_object, create_indexed_db_object, create_fetch_function};
pub use nextjs::{create_nextjs_object, create_next_router_object, create_next_link_component, create_next_image_component, create_next_head_component, create_next_script_component, create_use_router_hook, create_use_pathname_hook, create_use_search_params_hook};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsError {
    #[error("Syntax error: {0}")]
    SyntaxError(String),
    
    #[error("Reference error: {0} is not defined")]
    ReferenceError(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

pub type JsResult<T> = Result<T, JsError>;

/// Execute JavaScript code and return the result
pub fn execute(code: &str) -> JsResult<JsValue> {
    let mut tokenizer = Tokenizer::new(code);
    let tokens = tokenizer.tokenize()?;
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    let mut interpreter = Interpreter::new();
    interpreter.execute(&ast)
}

/// Execute JavaScript code with DOM support
pub fn execute_with_dom(code: &str, interpreter: &mut Interpreter) -> JsResult<JsValue> {
    let mut tokenizer = Tokenizer::new(code);
    let tokens = tokenizer.tokenize()?;
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    interpreter.execute(&ast)
}

/// Create a new interpreter with DOM globals (document, window, React, etc.)
pub fn create_browser_interpreter() -> Interpreter {
    let mut interpreter = Interpreter::new();
    
    // Add DOM globals
    let dom = DomBridge::new();
    interpreter.define_global("document", dom.create_document_object());
    interpreter.define_global("window", DomBridge::create_window_object());
    
    // Add console, history, performance, storage
    interpreter.define_global("console", dom_bridge::create_console_object());
    interpreter.define_global("history", dom_bridge::create_history_object());
    interpreter.define_global("performance", dom_bridge::create_performance_object());
    interpreter.define_global("localStorage", builtins::create_local_storage());
    interpreter.define_global("sessionStorage", builtins::create_session_storage());
    
    // Add React
    interpreter.define_global("React", create_react_object());
    interpreter.define_global("ReactDOM", create_react_dom_object());
    
    // Add Next.js
    let nextjs_module = create_nextjs_object();
    interpreter.define_global("next", nextjs_module.clone());
    
    // Add module resolution for 'next' imports
    interpreter.define_global("__require__", JsValue::NativeFunction {
        name: "__require__".to_string(),
        arity: 1,
        func: |args| {
            let module_name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            if module_name == "next" || module_name.starts_with("next/") {
                create_nextjs_object()
            } else if module_name == "react" {
                create_react_object()
            } else if module_name == "react-dom" || module_name == "react-dom/client" {
                create_react_dom_object()
            } else if module_name.starts_with("@vercel/") {
                // Vercel-specific modules - return mock object
                let mut vercel_module = std::collections::HashMap::new();
                vercel_module.insert("default".to_string(), JsValue::Undefined);
                JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(vercel_module)))
            } else if module_name == "next/link" {
                // Next.js Link component
                let mut link_obj = std::collections::HashMap::new();
                link_obj.insert("default".to_string(), nextjs::create_next_link_component());
                JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(link_obj)))
            } else if module_name == "next/image" {
                // Next.js Image component
                nextjs::create_next_image_component()
            } else if module_name == "next/head" {
                // Next.js Head component
                nextjs::create_next_head_component()
            } else if module_name == "next/script" {
                // Next.js Script component
                nextjs::create_next_script_component()
            } else if module_name == "next/router" {
                // Next.js router
                nextjs::create_next_router_object()
            } else if module_name == "next/navigation" {
                // Next.js App Router navigation
                let mut nav = std::collections::HashMap::new();
                nav.insert("useRouter".to_string(), nextjs::create_use_router_hook());
                nav.insert("usePathname".to_string(), nextjs::create_use_pathname_hook());
                nav.insert("useSearchParams".to_string(), nextjs::create_use_search_params_hook());
                JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(nav)))
            } else {
                // Return empty module for unknown modules
                JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())))
            }
        },
    });
    
    // Add import() function for dynamic imports
    interpreter.define_global("import", JsValue::NativeFunction {
        name: "import".to_string(),
        arity: 1,
        func: |args| {
            let module_name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[JS] Dynamic import: {}", module_name);
            // Return a promise-like object
            let mut promise = std::collections::HashMap::new();
            promise.insert("_module".to_string(), JsValue::String(module_name));
            promise.insert("then".to_string(), JsValue::NativeFunction {
                name: "then".to_string(),
                arity: 1,
                func: |_| JsValue::Undefined,
            });
            promise.insert("catch".to_string(), JsValue::NativeFunction {
                name: "catch".to_string(),
                arity: 1,
                func: |_| JsValue::Undefined,
            });
            JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(promise)))
        },
    });
    
    // Add Database APIs
    interpreter.define_global("DB", create_database_object());
    interpreter.define_global("indexedDB", create_indexed_db_object());
    interpreter.define_global("fetch", create_fetch_function());
    
    // Add Promise mock
    interpreter.define_global("Promise", create_promise_object());
    
    // Add other browser globals
    interpreter.define_global("requestAnimationFrame", JsValue::NativeFunction {
        name: "requestAnimationFrame".to_string(),
        arity: 1,
        func: |_| JsValue::Number(1.0),
    });
    interpreter.define_global("cancelAnimationFrame", JsValue::NativeFunction {
        name: "cancelAnimationFrame".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    interpreter.define_global("clearTimeout", JsValue::NativeFunction {
        name: "clearTimeout".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    interpreter.define_global("clearInterval", JsValue::NativeFunction {
        name: "clearInterval".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    interpreter.define_global("atob", JsValue::NativeFunction {
        name: "atob".to_string(),
        arity: 1,
        func: |args| {
            let s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            JsValue::String(s) // Mock - just return the string
        },
    });
    interpreter.define_global("btoa", JsValue::NativeFunction {
        name: "btoa".to_string(),
        arity: 1,
        func: |args| {
            let s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            JsValue::String(s) // Mock - just return the string
        },
    });
    
    // Add JSON
    let mut json_obj = std::collections::HashMap::new();
    json_obj.insert("parse".to_string(), JsValue::NativeFunction {
        name: "parse".to_string(),
        arity: 1,
        func: |args| {
            let _s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())))
        },
    });
    json_obj.insert("stringify".to_string(), JsValue::NativeFunction {
        name: "stringify".to_string(),
        arity: 1,
        func: |args| {
            let v = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            JsValue::String(v)
        },
    });
    interpreter.define_global("JSON", JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(json_obj))));
    
    // Add Object
    let mut object_obj = std::collections::HashMap::new();
    object_obj.insert("keys".to_string(), JsValue::NativeFunction {
        name: "keys".to_string(),
        arity: 1,
        func: |_| JsValue::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![]))),
    });
    object_obj.insert("values".to_string(), JsValue::NativeFunction {
        name: "values".to_string(),
        arity: 1,
        func: |_| JsValue::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![]))),
    });
    object_obj.insert("assign".to_string(), JsValue::NativeFunction {
        name: "assign".to_string(),
        arity: 2,
        func: |args| args.first().cloned().unwrap_or(JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())))),
    });
    object_obj.insert("defineProperty".to_string(), JsValue::NativeFunction {
        name: "defineProperty".to_string(),
        arity: 3,
        func: |args| args.first().cloned().unwrap_or(JsValue::Undefined),
    });
    interpreter.define_global("Object", JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(object_obj))));
    
    // Add Array
    let mut array_obj = std::collections::HashMap::new();
    array_obj.insert("isArray".to_string(), JsValue::NativeFunction {
        name: "isArray".to_string(),
        arity: 1,
        func: |args| {
            match args.first() {
                Some(JsValue::Array(_)) => JsValue::Boolean(true),
                _ => JsValue::Boolean(false),
            }
        },
    });
    array_obj.insert("from".to_string(), JsValue::NativeFunction {
        name: "from".to_string(),
        arity: 1,
        func: |_| JsValue::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![]))),
    });
    interpreter.define_global("Array", JsValue::Object(std::rc::Rc::new(std::cell::RefCell::new(array_obj))));
    
    interpreter
}

/// Create Promise object
fn create_promise_object() -> JsValue {
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::cell::RefCell;
    
    let mut methods = HashMap::new();
    
    methods.insert("resolve".to_string(), JsValue::NativeFunction {
        name: "resolve".to_string(),
        arity: 1,
        func: |args| {
            let value = args.first().cloned().unwrap_or(JsValue::Undefined);
            let mut promise = HashMap::new();
            promise.insert("_resolved".to_string(), value);
            promise.insert("then".to_string(), JsValue::NativeFunction {
                name: "then".to_string(),
                arity: 1,
                func: |_| JsValue::Undefined,
            });
            promise.insert("catch".to_string(), JsValue::NativeFunction {
                name: "catch".to_string(),
                arity: 1,
                func: |_| JsValue::Undefined,
            });
            JsValue::Object(Rc::new(RefCell::new(promise)))
        },
    });
    
    methods.insert("reject".to_string(), JsValue::NativeFunction {
        name: "reject".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("all".to_string(), JsValue::NativeFunction {
        name: "all".to_string(),
        arity: 1,
        func: |_| JsValue::Array(Rc::new(RefCell::new(vec![]))),
    });
    
    methods.insert("race".to_string(), JsValue::NativeFunction {
        name: "race".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}
