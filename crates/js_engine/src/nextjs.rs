//! Next.js Runtime Support
//! 
//! Provides Next.js APIs for client-side routing, navigation, and components.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::value::JsValue;

// Global router state (simplified - in real implementation would be per-page)
// Using a simple static RefCell wrapped in a Mutex for thread safety
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref ROUTER_STATE: Mutex<RouterState> = Mutex::new(RouterState {
        pathname: "/".to_string(),
        query: HashMap::new(),
        as_path: "/".to_string(),
    });
}

struct RouterState {
    pathname: String,
    query: HashMap<String, String>,
    as_path: String,
}

/// Create Next.js router object
pub fn create_next_router_object() -> JsValue {
    let mut router = HashMap::new();
    
    router.insert("push".to_string(), JsValue::NativeFunction {
        name: "push".to_string(),
        arity: 1,
        func: next_router_push,
    });
    
    router.insert("replace".to_string(), JsValue::NativeFunction {
        name: "replace".to_string(),
        arity: 1,
        func: next_router_replace,
    });
    
    router.insert("back".to_string(), JsValue::NativeFunction {
        name: "back".to_string(),
        arity: 0,
        func: next_router_back,
    });
    
    router.insert("reload".to_string(), JsValue::NativeFunction {
        name: "reload".to_string(),
        arity: 0,
        func: next_router_reload,
    });
    
    router.insert("prefetch".to_string(), JsValue::NativeFunction {
        name: "prefetch".to_string(),
        arity: 1,
        func: next_router_prefetch,
    });
    
    router.insert("pathname".to_string(), JsValue::NativeFunction {
        name: "pathname".to_string(),
        arity: 0,
        func: |_| {
            ROUTER_STATE.lock().map(|state| {
                JsValue::String(state.pathname.clone())
            }).unwrap_or_else(|_| JsValue::String("/".to_string()))
        },
    });
    
    router.insert("query".to_string(), JsValue::NativeFunction {
        name: "query".to_string(),
        arity: 0,
        func: |_| {
            ROUTER_STATE.lock().map(|state| {
                let query_map: HashMap<String, JsValue> = state.query.iter()
                    .map(|(k, v)| (k.clone(), JsValue::String(v.clone())))
                    .collect();
                JsValue::Object(Rc::new(RefCell::new(query_map)))
            }).unwrap_or_else(|_| JsValue::Object(Rc::new(RefCell::new(HashMap::new()))))
        },
    });
    
    router.insert("asPath".to_string(), JsValue::NativeFunction {
        name: "asPath".to_string(),
        arity: 0,
        func: |_| {
            ROUTER_STATE.lock().map(|state| {
                JsValue::String(state.as_path.clone())
            }).unwrap_or_else(|_| JsValue::String("/".to_string()))
        },
    });
    
    router.insert("events".to_string(), create_next_router_events());
    
    JsValue::Object(Rc::new(RefCell::new(router)))
}

fn next_router_push(args: &[JsValue]) -> JsValue {
    let url = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[Next.js Router] push: {}", url);
    
    // Update router state
    if let Ok(mut state) = ROUTER_STATE.lock() {
        // Parse URL to extract pathname and query
        if let Some(path_end) = url.find('?') {
            state.pathname = url[..path_end].to_string();
            // Parse query string
            let query_str = &url[path_end + 1..];
            state.query.clear();
            for pair in query_str.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = &pair[..eq_pos];
                    let value = &pair[eq_pos + 1..];
                    state.query.insert(key.to_string(), value.to_string());
                }
            }
        } else {
            state.pathname = url.clone();
            state.query.clear();
        }
        state.as_path = url.clone();
    }
    
    // Trigger navigation via window.location if available
    // This will be handled by the browser's navigation system
    JsValue::Undefined
}

fn next_router_replace(args: &[JsValue]) -> JsValue {
    let url = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[Next.js Router] replace: {}", url);
    
    // Update router state
    if let Ok(mut state) = ROUTER_STATE.lock() {
        state.pathname = url.clone();
        state.as_path = url.clone();
    }
    
    JsValue::Undefined
}

fn next_router_back(_args: &[JsValue]) -> JsValue {
    println!("[Next.js Router] back");
    // In real browser, would use window.history.back()
    JsValue::Undefined
}

fn next_router_reload(_args: &[JsValue]) -> JsValue {
    println!("[Next.js Router] reload");
    // In real browser, would use window.location.reload()
    JsValue::Undefined
}

fn next_router_prefetch(args: &[JsValue]) -> JsValue {
    let url = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[Next.js Router] prefetch: {}", url);
    // Prefetching would load resources in background
    JsValue::Undefined
}

fn create_next_router_events() -> JsValue {
    let mut events = HashMap::new();
    
    events.insert("on".to_string(), JsValue::NativeFunction {
        name: "on".to_string(),
        arity: 2,
        func: |_| JsValue::Undefined,
    });
    
    events.insert("off".to_string(), JsValue::NativeFunction {
        name: "off".to_string(),
        arity: 2,
        func: |_| JsValue::Undefined,
    });
    
    JsValue::Object(Rc::new(RefCell::new(events)))
}

/// Create Next.js Link component factory
pub fn create_next_link_component() -> JsValue {
    let mut link = HashMap::new();
    
    link.insert("default".to_string(), JsValue::NativeFunction {
        name: "Link".to_string(),
        arity: 2,
        func: next_link_component,
    });
    
    JsValue::Object(Rc::new(RefCell::new(link)))
}

fn next_link_component(args: &[JsValue]) -> JsValue {
    let props = args.first().cloned().unwrap_or(JsValue::Null);
    let children = args.get(1).cloned();
    
    println!("[Next.js Link] href: {:?}, children: {:?}", props, children);
    
    // Return a React element-like object
    let mut elem = HashMap::new();
    elem.insert("type".to_string(), JsValue::String("a".to_string()));
    elem.insert("props".to_string(), props);
    elem.insert("children".to_string(), children.unwrap_or(JsValue::Null));
    
    JsValue::Object(Rc::new(RefCell::new(elem)))
}

/// Create Next.js Image component
pub fn create_next_image_component() -> JsValue {
    JsValue::NativeFunction {
        name: "Image".to_string(),
        arity: 1,
        func: next_image_component,
    }
}

fn next_image_component(args: &[JsValue]) -> JsValue {
    let props = args.first().cloned().unwrap_or(JsValue::Null);
    println!("[Next.js Image] props: {:?}", props);
    
    let mut elem = HashMap::new();
    elem.insert("type".to_string(), JsValue::String("img".to_string()));
    elem.insert("props".to_string(), props);
    
    JsValue::Object(Rc::new(RefCell::new(elem)))
}

/// Create Next.js Head component
pub fn create_next_head_component() -> JsValue {
    JsValue::NativeFunction {
        name: "Head".to_string(),
        arity: 1,
        func: next_head_component,
    }
}

fn next_head_component(args: &[JsValue]) -> JsValue {
    let children = args.first().cloned();
    println!("[Next.js Head] children: {:?}", children);
    
    let mut elem = HashMap::new();
    elem.insert("type".to_string(), JsValue::String("head".to_string()));
    elem.insert("children".to_string(), children.unwrap_or(JsValue::Null));
    
    JsValue::Object(Rc::new(RefCell::new(elem)))
}

/// Create Next.js Script component
pub fn create_next_script_component() -> JsValue {
    JsValue::NativeFunction {
        name: "Script".to_string(),
        arity: 1,
        func: next_script_component,
    }
}

fn next_script_component(args: &[JsValue]) -> JsValue {
    let props = args.first().cloned().unwrap_or(JsValue::Null);
    println!("[Next.js Script] props: {:?}", props);
    
    let mut elem = HashMap::new();
    elem.insert("type".to_string(), JsValue::String("script".to_string()));
    elem.insert("props".to_string(), props);
    
    JsValue::Object(Rc::new(RefCell::new(elem)))
}

/// Create Next.js useRouter hook
pub fn create_use_router_hook() -> JsValue {
    JsValue::NativeFunction {
        name: "useRouter".to_string(),
        arity: 0,
        func: |_| create_next_router_object(),
    }
}

/// Create Next.js usePathname hook
pub fn create_use_pathname_hook() -> JsValue {
    JsValue::NativeFunction {
        name: "usePathname".to_string(),
        arity: 0,
        func: |_| {
            ROUTER_STATE.lock().map(|state| {
                JsValue::String(state.pathname.clone())
            }).unwrap_or_else(|_| JsValue::String("/".to_string()))
        },
    }
}

/// Create Next.js useSearchParams hook
pub fn create_use_search_params_hook() -> JsValue {
    JsValue::NativeFunction {
        name: "useSearchParams".to_string(),
        arity: 0,
        func: |_| {
            ROUTER_STATE.lock().map(|state| {
                let query_map: HashMap<String, JsValue> = state.query.iter()
                    .map(|(k, v)| (k.clone(), JsValue::String(v.clone())))
                    .collect();
                JsValue::Object(Rc::new(RefCell::new(query_map)))
            }).unwrap_or_else(|_| JsValue::Object(Rc::new(RefCell::new(HashMap::new()))))
        },
    }
}

/// Create Next.js dynamic import function
pub fn create_dynamic_import() -> JsValue {
    JsValue::NativeFunction {
        name: "dynamic".to_string(),
        arity: 1,
        func: next_dynamic_import,
    }
}

fn next_dynamic_import(args: &[JsValue]) -> JsValue {
    let import_fn = args.first();
    println!("[Next.js] dynamic import: {:?}", import_fn);
    
    // Return a promise-like object that resolves to the imported module
    let mut promise = HashMap::new();
    promise.insert("_resolved".to_string(), JsValue::Object(Rc::new(RefCell::new(HashMap::new()))));
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
}

/// Create the main Next.js module object
pub fn create_nextjs_object() -> JsValue {
    let mut nextjs = HashMap::new();
    
    // Router
    nextjs.insert("router".to_string(), create_next_router_object());
    
    // Hooks
    nextjs.insert("useRouter".to_string(), create_use_router_hook());
    nextjs.insert("usePathname".to_string(), create_use_pathname_hook());
    nextjs.insert("useSearchParams".to_string(), create_use_search_params_hook());
    
    // Components
    let mut link_obj = HashMap::new();
    link_obj.insert("default".to_string(), create_next_link_component());
    nextjs.insert("link".to_string(), JsValue::Object(Rc::new(RefCell::new(link_obj))));
    
    nextjs.insert("Image".to_string(), create_next_image_component());
    nextjs.insert("Head".to_string(), create_next_head_component());
    nextjs.insert("Script".to_string(), create_next_script_component());
    
    // Dynamic imports
    nextjs.insert("dynamic".to_string(), create_dynamic_import());
    
    // App Router APIs
    let mut app_router = HashMap::new();
    app_router.insert("useRouter".to_string(), create_use_router_hook());
    app_router.insert("usePathname".to_string(), create_use_pathname_hook());
    app_router.insert("useSearchParams".to_string(), create_use_search_params_hook());
    nextjs.insert("navigation".to_string(), JsValue::Object(Rc::new(RefCell::new(app_router))));
    
    // Server Components (mock)
    nextjs.insert("headers".to_string(), JsValue::NativeFunction {
        name: "headers".to_string(),
        arity: 0,
        func: |_| JsValue::Object(Rc::new(RefCell::new(HashMap::new()))),
    });
    
    nextjs.insert("cookies".to_string(), JsValue::NativeFunction {
        name: "cookies".to_string(),
        arity: 0,
        func: |_| JsValue::Object(Rc::new(RefCell::new(HashMap::new()))),
    });
    
    // Config
    nextjs.insert("config".to_string(), JsValue::Object(Rc::new(RefCell::new(HashMap::new()))));
    
    // Metadata API
    let mut metadata = HashMap::new();
    metadata.insert("generateMetadata".to_string(), JsValue::NativeFunction {
        name: "generateMetadata".to_string(),
        arity: 1,
        func: |_| JsValue::Object(Rc::new(RefCell::new(HashMap::new()))),
    });
    nextjs.insert("metadata".to_string(), JsValue::Object(Rc::new(RefCell::new(metadata))));
    
    JsValue::Object(Rc::new(RefCell::new(nextjs)))
}

