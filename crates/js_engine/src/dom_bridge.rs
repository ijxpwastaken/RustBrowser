//! DOM Bridge
//! 
//! Provides JavaScript DOM APIs for interacting with the browser's DOM.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::value::JsValue;
use dom::{NodeRef, Document};

/// Document DOM wrapper for JavaScript
pub struct DomBridge {
    /// The underlying DOM document
    document: Option<Document>,
    /// Element cache (id -> element reference)  
    element_cache: HashMap<String, NodeRef>,
}

impl DomBridge {
    pub fn new() -> Self {
        DomBridge {
            document: None,
            element_cache: HashMap::new(),
        }
    }
    
    /// Set the current document
    pub fn set_document(&mut self, doc: Document) {
        self.element_cache.clear();
        self.document = Some(doc);
    }
    
    /// Create the document JavaScript object
    pub fn create_document_object(&self) -> JsValue {
        let mut methods = HashMap::new();
        
        methods.insert("getElementById".to_string(), JsValue::NativeFunction {
            name: "getElementById".to_string(),
            arity: 1,
            func: document_get_element_by_id,
        });
        
        methods.insert("querySelector".to_string(), JsValue::NativeFunction {
            name: "querySelector".to_string(),
            arity: 1,
            func: document_query_selector,
        });
        
        methods.insert("querySelectorAll".to_string(), JsValue::NativeFunction {
            name: "querySelectorAll".to_string(),
            arity: 1,
            func: document_query_selector_all,
        });
        
        methods.insert("createElement".to_string(), JsValue::NativeFunction {
            name: "createElement".to_string(),
            arity: 1,
            func: document_create_element,
        });
        
        methods.insert("createTextNode".to_string(), JsValue::NativeFunction {
            name: "createTextNode".to_string(),
            arity: 1,
            func: document_create_text_node,
        });
        
        methods.insert("write".to_string(), JsValue::NativeFunction {
            name: "write".to_string(),
            arity: 1,
            func: document_write,
        });
        
        JsValue::Object(Rc::new(RefCell::new(methods)))
    }
    
    /// Create the window JavaScript object
    pub fn create_window_object() -> JsValue {
        let mut methods = HashMap::new();
        
        methods.insert("alert".to_string(), JsValue::NativeFunction {
            name: "alert".to_string(),
            arity: 1,
            func: window_alert,
        });
        
        methods.insert("confirm".to_string(), JsValue::NativeFunction {
            name: "confirm".to_string(),
            arity: 1,
            func: window_confirm,
        });
        
        methods.insert("setTimeout".to_string(), JsValue::NativeFunction {
            name: "setTimeout".to_string(),
            arity: 2,
            func: window_set_timeout,
        });
        
        methods.insert("setInterval".to_string(), JsValue::NativeFunction {
            name: "setInterval".to_string(),
            arity: 2,
            func: window_set_interval,
        });
        
        methods.insert("location".to_string(), create_location_object());
        methods.insert("navigator".to_string(), create_navigator_object());
        
        JsValue::Object(Rc::new(RefCell::new(methods)))
    }
}

fn create_location_object() -> JsValue {
    let mut props = HashMap::new();
    // href is settable via assignment - interpreter handles this
    props.insert("href".to_string(), JsValue::String("https://example.com/".to_string()));
    props.insert("hostname".to_string(), JsValue::String("example.com".to_string()));
    props.insert("host".to_string(), JsValue::String("example.com".to_string()));
    props.insert("pathname".to_string(), JsValue::String("/".to_string()));
    props.insert("protocol".to_string(), JsValue::String("https:".to_string()));
    props.insert("origin".to_string(), JsValue::String("https://example.com".to_string()));
    props.insert("search".to_string(), JsValue::String("".to_string()));
    props.insert("hash".to_string(), JsValue::String("".to_string()));
    
    // Methods
    props.insert("assign".to_string(), JsValue::NativeFunction {
        name: "assign".to_string(),
        arity: 1,
        func: |args| {
            let url = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[REDIRECT] location.assign('{}')", url);
            JsValue::Undefined
        },
    });
    
    props.insert("replace".to_string(), JsValue::NativeFunction {
        name: "replace".to_string(),
        arity: 1,
        func: |args| {
            let url = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[REDIRECT] location.replace('{}')", url);
            JsValue::Undefined
        },
    });
    
    props.insert("reload".to_string(), JsValue::NativeFunction {
        name: "reload".to_string(),
        arity: 0,
        func: |_| {
            println!("[REDIRECT] location.reload()");
            JsValue::Undefined
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(props)))
}

fn create_navigator_object() -> JsValue {
    let mut props = HashMap::new();
    props.insert("userAgent".to_string(), JsValue::String("Mozilla/5.0 (Windows NT 10.0; Win64; x64) RustBrowser/1.0".to_string()));
    props.insert("language".to_string(), JsValue::String("en-US".to_string()));
    props.insert("languages".to_string(), JsValue::Array(Rc::new(RefCell::new(vec![
        JsValue::String("en-US".to_string()),
        JsValue::String("en".to_string()),
    ]))));
    props.insert("platform".to_string(), JsValue::String("Win64".to_string()));
    props.insert("cookieEnabled".to_string(), JsValue::Boolean(true));
    props.insert("onLine".to_string(), JsValue::Boolean(true));
    props.insert("vendor".to_string(), JsValue::String("RustBrowser".to_string()));
    props.insert("hardwareConcurrency".to_string(), JsValue::Number(8.0));
    props.insert("maxTouchPoints".to_string(), JsValue::Number(0.0));
    
    // userAgentData for modern browsers
    let mut uad = HashMap::new();
    uad.insert("brands".to_string(), JsValue::Array(Rc::new(RefCell::new(vec![]))));
    uad.insert("mobile".to_string(), JsValue::Boolean(false));
    uad.insert("platform".to_string(), JsValue::String("Windows".to_string()));
    props.insert("userAgentData".to_string(), JsValue::Object(Rc::new(RefCell::new(uad))));
    
    JsValue::Object(Rc::new(RefCell::new(props)))
}

/// Create console object
pub fn create_console_object() -> JsValue {
    let mut methods = HashMap::new();
    
    methods.insert("log".to_string(), JsValue::NativeFunction {
        name: "log".to_string(),
        arity: 1,
        func: |args| {
            let msg: Vec<String> = args.iter().map(|v| v.to_js_string()).collect();
            println!("[JS] {}", msg.join(" "));
            JsValue::Undefined
        },
    });
    
    methods.insert("warn".to_string(), JsValue::NativeFunction {
        name: "warn".to_string(),
        arity: 1,
        func: |args| {
            let msg: Vec<String> = args.iter().map(|v| v.to_js_string()).collect();
            println!("[JS WARN] {}", msg.join(" "));
            JsValue::Undefined
        },
    });
    
    methods.insert("error".to_string(), JsValue::NativeFunction {
        name: "error".to_string(),
        arity: 1,
        func: |args| {
            let msg: Vec<String> = args.iter().map(|v| v.to_js_string()).collect();
            eprintln!("[JS ERROR] {}", msg.join(" "));
            JsValue::Undefined
        },
    });
    
    methods.insert("info".to_string(), JsValue::NativeFunction {
        name: "info".to_string(),
        arity: 1,
        func: |args| {
            let msg: Vec<String> = args.iter().map(|v| v.to_js_string()).collect();
            println!("[JS INFO] {}", msg.join(" "));
            JsValue::Undefined
        },
    });
    
    methods.insert("debug".to_string(), JsValue::NativeFunction {
        name: "debug".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("trace".to_string(), JsValue::NativeFunction {
        name: "trace".to_string(),
        arity: 0,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("time".to_string(), JsValue::NativeFunction {
        name: "time".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("timeEnd".to_string(), JsValue::NativeFunction {
        name: "timeEnd".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create history object
pub fn create_history_object() -> JsValue {
    let mut methods = HashMap::new();
    
    methods.insert("pushState".to_string(), JsValue::NativeFunction {
        name: "pushState".to_string(),
        arity: 3,
        func: |args| {
            let url = args.get(2).map(|v| v.to_js_string()).unwrap_or_default();
            println!("[HISTORY] pushState to {}", url);
            JsValue::Undefined
        },
    });
    
    methods.insert("replaceState".to_string(), JsValue::NativeFunction {
        name: "replaceState".to_string(),
        arity: 3,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("back".to_string(), JsValue::NativeFunction {
        name: "back".to_string(),
        arity: 0,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("forward".to_string(), JsValue::NativeFunction {
        name: "forward".to_string(),
        arity: 0,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("go".to_string(), JsValue::NativeFunction {
        name: "go".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("length".to_string(), JsValue::Number(1.0));
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create performance object
pub fn create_performance_object() -> JsValue {
    let mut methods = HashMap::new();
    
    methods.insert("now".to_string(), JsValue::NativeFunction {
        name: "now".to_string(),
        arity: 0,
        func: |_| JsValue::Number(0.0),
    });
    
    methods.insert("mark".to_string(), JsValue::NativeFunction {
        name: "mark".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("measure".to_string(), JsValue::NativeFunction {
        name: "measure".to_string(),
        arity: 3,
        func: |_| JsValue::Undefined,
    });
    
    let mut timing = HashMap::new();
    timing.insert("navigationStart".to_string(), JsValue::Number(0.0));
    timing.insert("responseStart".to_string(), JsValue::Number(0.0));
    timing.insert("domContentLoadedEventEnd".to_string(), JsValue::Number(0.0));
    methods.insert("timing".to_string(), JsValue::Object(Rc::new(RefCell::new(timing))));
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create localStorage/sessionStorage
pub fn create_storage_object() -> JsValue {
    let mut methods = HashMap::new();
    
    methods.insert("getItem".to_string(), JsValue::NativeFunction {
        name: "getItem".to_string(),
        arity: 1,
        func: |_| JsValue::Null,
    });
    
    methods.insert("setItem".to_string(), JsValue::NativeFunction {
        name: "setItem".to_string(),
        arity: 2,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("removeItem".to_string(), JsValue::NativeFunction {
        name: "removeItem".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("clear".to_string(), JsValue::NativeFunction {
        name: "clear".to_string(),
        arity: 0,
        func: |_| JsValue::Undefined,
    });
    
    methods.insert("length".to_string(), JsValue::Number(0.0));
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

// Document methods

fn document_get_element_by_id(args: &[JsValue]) -> JsValue {
    let id = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] getElementById('{}')", id);
    
    // Return a mock element object
    create_element_object(&id, "div")
}

fn document_query_selector(args: &[JsValue]) -> JsValue {
    let selector = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] querySelector('{}')", selector);
    
    // Return a mock element
    create_element_object("", "div")
}

fn document_query_selector_all(args: &[JsValue]) -> JsValue {
    let selector = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] querySelectorAll('{}')", selector);
    
    // Return empty array for now
    JsValue::Array(Rc::new(RefCell::new(vec![])))
}

fn document_create_element(args: &[JsValue]) -> JsValue {
    let tag = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] createElement('{}')", tag);
    create_element_object("", &tag)
}

fn document_create_text_node(args: &[JsValue]) -> JsValue {
    let text = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] createTextNode('{}')", text);
    JsValue::Object(Rc::new(RefCell::new(HashMap::new())))
}

fn document_write(args: &[JsValue]) -> JsValue {
    let content = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] document.write: {}", content);
    JsValue::Undefined
}

// Window methods

fn window_alert(args: &[JsValue]) -> JsValue {
    let msg = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[ALERT] {}", msg);
    JsValue::Undefined
}

fn window_confirm(args: &[JsValue]) -> JsValue {
    let msg = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[CONFIRM] {}", msg);
    // Always return true for now
    JsValue::Boolean(true)
}

fn window_set_timeout(args: &[JsValue]) -> JsValue {
    let _callback = args.first();
    let delay = args.get(1).map(|v| v.to_number()).unwrap_or(0.0);
    println!("[DOM] setTimeout with delay {}ms (not implemented)", delay);
    // Return a timer ID
    JsValue::Number(1.0)
}

fn window_set_interval(args: &[JsValue]) -> JsValue {
    let _callback = args.first();
    let delay = args.get(1).map(|v| v.to_number()).unwrap_or(0.0);
    println!("[DOM] setInterval with delay {}ms (not implemented)", delay);
    // Return a timer ID
    JsValue::Number(1.0)
}

/// Create a JavaScript element object
fn create_element_object(id: &str, tag: &str) -> JsValue {
    let mut props: HashMap<String, JsValue> = HashMap::new();
    
    props.insert("id".to_string(), JsValue::String(id.to_string()));
    props.insert("tagName".to_string(), JsValue::String(tag.to_uppercase()));
    props.insert("className".to_string(), JsValue::String(String::new()));
    props.insert("innerHTML".to_string(), JsValue::String(String::new()));
    props.insert("innerText".to_string(), JsValue::String(String::new()));
    props.insert("textContent".to_string(), JsValue::String(String::new()));
    
    // Style object
    let style = create_style_object();
    props.insert("style".to_string(), style);
    
    // Methods
    props.insert("addEventListener".to_string(), JsValue::NativeFunction {
        name: "addEventListener".to_string(),
        arity: 2,
        func: element_add_event_listener,
    });
    
    props.insert("appendChild".to_string(), JsValue::NativeFunction {
        name: "appendChild".to_string(),
        arity: 1,
        func: element_append_child,
    });
    
    props.insert("setAttribute".to_string(), JsValue::NativeFunction {
        name: "setAttribute".to_string(),
        arity: 2,
        func: element_set_attribute,
    });
    
    props.insert("getAttribute".to_string(), JsValue::NativeFunction {
        name: "getAttribute".to_string(),
        arity: 1,
        func: element_get_attribute,
    });
    
    props.insert("removeChild".to_string(), JsValue::NativeFunction {
        name: "removeChild".to_string(),
        arity: 1,
        func: element_remove_child,
    });
    
    props.insert("querySelector".to_string(), JsValue::NativeFunction {
        name: "querySelector".to_string(),
        arity: 1,
        func: document_query_selector,
    });
    
    // Add video-specific methods if tag is "video"
    if tag == "video" {
        props.insert("play".to_string(), JsValue::NativeFunction {
            name: "play".to_string(),
            arity: 0,
            func: |_| {
                println!("[VIDEO] play() called");
                // Return a promise-like object
                let mut promise = HashMap::new();
                promise.insert("then".to_string(), JsValue::NativeFunction {
                    name: "then".to_string(),
                    arity: 1,
                    func: |args| {
                        let callback = args.first();
                        println!("[VIDEO] play promise resolved");
                        if let Some(JsValue::NativeFunction { func, .. }) = callback {
                            func(&[]);
                        }
                        JsValue::Undefined
                    },
                });
                JsValue::Object(Rc::new(RefCell::new(promise)))
            },
        });
        
        props.insert("pause".to_string(), JsValue::NativeFunction {
            name: "pause".to_string(),
            arity: 0,
            func: |_| {
                println!("[VIDEO] pause() called");
                JsValue::Undefined
            },
        });
        
        props.insert("load".to_string(), JsValue::NativeFunction {
            name: "load".to_string(),
            arity: 0,
            func: |_| {
                println!("[VIDEO] load() called");
                JsValue::Undefined
            },
        });
        
        props.insert("currentTime".to_string(), JsValue::Number(0.0));
        props.insert("duration".to_string(), JsValue::Number(0.0));
        props.insert("paused".to_string(), JsValue::Boolean(true));
        props.insert("ended".to_string(), JsValue::Boolean(false));
        props.insert("src".to_string(), JsValue::String("".to_string()));
        props.insert("poster".to_string(), JsValue::String("".to_string()));
    }
    
    JsValue::Object(Rc::new(RefCell::new(props)))
}

fn create_style_object() -> JsValue {
    let mut props = HashMap::new();
    props.insert("display".to_string(), JsValue::String(String::new()));
    props.insert("color".to_string(), JsValue::String(String::new()));
    props.insert("backgroundColor".to_string(), JsValue::String(String::new()));
    props.insert("width".to_string(), JsValue::String(String::new()));
    props.insert("height".to_string(), JsValue::String(String::new()));
    props.insert("margin".to_string(), JsValue::String(String::new()));
    props.insert("padding".to_string(), JsValue::String(String::new()));
    props.insert("border".to_string(), JsValue::String(String::new()));
    props.insert("fontSize".to_string(), JsValue::String(String::new()));
    props.insert("fontFamily".to_string(), JsValue::String(String::new()));
    props.insert("position".to_string(), JsValue::String(String::new()));
    props.insert("top".to_string(), JsValue::String(String::new()));
    props.insert("left".to_string(), JsValue::String(String::new()));
    props.insert("right".to_string(), JsValue::String(String::new()));
    props.insert("bottom".to_string(), JsValue::String(String::new()));
    JsValue::Object(Rc::new(RefCell::new(props)))
}

fn element_add_event_listener(args: &[JsValue]) -> JsValue {
    let event = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] addEventListener('{}', callback)", event);
    JsValue::Undefined
}

fn element_append_child(args: &[JsValue]) -> JsValue {
    println!("[DOM] appendChild(element)");
    args.first().cloned().unwrap_or(JsValue::Undefined)
}

fn element_set_attribute(args: &[JsValue]) -> JsValue {
    let name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let value = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] setAttribute('{}', '{}')", name, value);
    JsValue::Undefined
}

fn element_get_attribute(args: &[JsValue]) -> JsValue {
    let name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DOM] getAttribute('{}')", name);
    JsValue::Null
}

fn element_remove_child(args: &[JsValue]) -> JsValue {
    println!("[DOM] removeChild(element)");
    args.first().cloned().unwrap_or(JsValue::Undefined)
}

impl Default for DomBridge {
    fn default() -> Self {
        Self::new()
    }
}
