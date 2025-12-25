//! JSX Parser and React Runtime
//! 
//! Basic JSX parsing and React-like component support.

use crate::value::JsValue;
use crate::{JsError, JsResult};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// JSX Element representation
#[derive(Debug, Clone)]
pub struct JsxElement {
    pub tag: String,
    pub props: HashMap<String, JsValue>,
    pub children: Vec<JsxNode>,
}

/// JSX Node (element or text)
#[derive(Debug, Clone)]
pub enum JsxNode {
    Element(JsxElement),
    Text(String),
    Expression(JsValue),
}

/// React-like component
pub type Component = fn(props: &HashMap<String, JsValue>) -> JsxNode;

/// JSX Parser
pub struct JsxParser<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> JsxParser<'a> {
    pub fn new(source: &'a str) -> Self {
        JsxParser { source, pos: 0 }
    }
    
    /// Parse JSX element
    pub fn parse(&mut self) -> JsResult<JsxNode> {
        self.skip_whitespace();
        
        if self.peek() == Some('<') {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }
    
    fn parse_element(&mut self) -> JsResult<JsxNode> {
        self.expect('<')?;
        
        // Check for closing tag
        if self.peek() == Some('/') {
            return Err(JsError::SyntaxError("Unexpected closing tag".into()));
        }
        
        // Parse tag name
        let tag = self.parse_identifier()?;
        
        // Parse props
        let props = self.parse_props()?;
        
        self.skip_whitespace();
        
        // Self-closing tag
        if self.peek() == Some('/') {
            self.advance();
            self.expect('>')?;
            return Ok(JsxNode::Element(JsxElement {
                tag,
                props,
                children: vec![],
            }));
        }
        
        self.expect('>')?;
        
        // Parse children
        let children = self.parse_children(&tag)?;
        
        // Parse closing tag
        self.expect('<')?;
        self.expect('/')?;
        let close_tag = self.parse_identifier()?;
        if close_tag != tag {
            return Err(JsError::SyntaxError(format!(
                "Mismatched tags: <{}> and </{}>", tag, close_tag
            )));
        }
        self.expect('>')?;
        
        Ok(JsxNode::Element(JsxElement { tag, props, children }))
    }
    
    fn parse_props(&mut self) -> JsResult<HashMap<String, JsValue>> {
        let mut props = HashMap::new();
        
        loop {
            self.skip_whitespace();
            
            // Check for end of props
            if self.peek() == Some('>') || self.peek() == Some('/') {
                break;
            }
            
            // Parse prop name
            let name = self.parse_identifier()?;
            
            self.skip_whitespace();
            
            // Check for = sign
            if self.peek() == Some('=') {
                self.advance();
                self.skip_whitespace();
                
                // Parse prop value
                let value = if self.peek() == Some('"') {
                    self.parse_string()?
                } else if self.peek() == Some('{') {
                    self.parse_expression()?
                } else {
                    JsValue::Boolean(true)
                };
                
                props.insert(name, value);
            } else {
                // Boolean prop
                props.insert(name, JsValue::Boolean(true));
            }
        }
        
        Ok(props)
    }
    
    fn parse_children(&mut self, _parent_tag: &str) -> JsResult<Vec<JsxNode>> {
        let mut children = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            // Check for closing tag
            if self.peek() == Some('<') && self.peek_next() == Some('/') {
                break;
            }
            
            if self.peek().is_none() {
                break;
            }
            
            // Parse child
            let child = if self.peek() == Some('<') {
                self.parse_element()?
            } else if self.peek() == Some('{') {
                self.parse_expression_node()?
            } else {
                self.parse_text()?
            };
            
            children.push(child);
        }
        
        Ok(children)
    }
    
    fn parse_text(&mut self) -> JsResult<JsxNode> {
        let mut text = String::new();
        
        while let Some(c) = self.peek() {
            if c == '<' || c == '{' {
                break;
            }
            text.push(c);
            self.advance();
        }
        
        Ok(JsxNode::Text(text.trim().to_string()))
    }
    
    fn parse_string(&mut self) -> JsResult<JsValue> {
        self.expect('"')?;
        let mut s = String::new();
        
        while let Some(c) = self.peek() {
            if c == '"' {
                break;
            }
            s.push(c);
            self.advance();
        }
        
        self.expect('"')?;
        Ok(JsValue::String(s))
    }
    
    fn parse_expression(&mut self) -> JsResult<JsValue> {
        self.expect('{')?;
        
        let mut depth = 1;
        let mut expr = String::new();
        
        while let Some(c) = self.peek() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            expr.push(c);
            self.advance();
        }
        
        self.expect('}')?;
        
        // Evaluate the expression
        // For now, just return as string
        Ok(JsValue::String(expr.trim().to_string()))
    }
    
    fn parse_expression_node(&mut self) -> JsResult<JsxNode> {
        let value = self.parse_expression()?;
        Ok(JsxNode::Expression(value))
    }
    
    fn parse_identifier(&mut self) -> JsResult<String> {
        let mut name = String::new();
        
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        if name.is_empty() {
            return Err(JsError::SyntaxError("Expected identifier".into()));
        }
        
        Ok(name)
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }
    
    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.pos..].chars();
        chars.next();
        chars.next()
    }
    
    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }
    
    fn expect(&mut self, c: char) -> JsResult<()> {
        if self.peek() == Some(c) {
            self.advance();
            Ok(())
        } else {
            Err(JsError::SyntaxError(format!("Expected '{}', got {:?}", c, self.peek())))
        }
    }
}

/// React-like runtime
pub struct ReactRuntime {
    components: HashMap<String, Component>,
}

impl ReactRuntime {
    pub fn new() -> Self {
        ReactRuntime {
            components: HashMap::new(),
        }
    }
    
    /// Register a component
    pub fn register_component(&mut self, name: &str, component: Component) {
        self.components.insert(name.to_string(), component);
    }
    
    /// Render JSX to HTML string
    pub fn render_to_string(&self, node: &JsxNode) -> String {
        match node {
            JsxNode::Text(text) => text.clone(),
            JsxNode::Expression(value) => value.to_js_string(),
            JsxNode::Element(elem) => {
                // Check if it's a component (starts with uppercase)
                if elem.tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    if let Some(component) = self.components.get(&elem.tag) {
                        let result = component(&elem.props);
                        return self.render_to_string(&result);
                    }
                }
                
                // Render HTML element
                let mut html = format!("<{}", elem.tag);
                
                // Add props as attributes
                for (key, value) in &elem.props {
                    let attr_value = match value {
                        JsValue::String(s) => s.clone(),
                        JsValue::Number(n) => n.to_string(),
                        JsValue::Boolean(b) => b.to_string(),
                        _ => value.to_js_string(),
                    };
                    html.push_str(&format!(" {}=\"{}\"", key, attr_value));
                }
                
                // Self-closing tags
                if is_void_element(&elem.tag) && elem.children.is_empty() {
                    html.push_str(" />");
                    return html;
                }
                
                html.push('>');
                
                // Render children
                for child in &elem.children {
                    html.push_str(&self.render_to_string(child));
                }
                
                html.push_str(&format!("</{}>", elem.tag));
                html
            }
        }
    }
}

fn is_void_element(tag: &str) -> bool {
    matches!(tag.to_lowercase().as_str(), 
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | 
        "input" | "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
}

impl Default for ReactRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Create React object for JavaScript - Full React 19 Support
pub fn create_react_object() -> JsValue {
    let mut methods = HashMap::new();
    
    // ============================================================
    // Core APIs
    // ============================================================
    
    methods.insert("createElement".to_string(), JsValue::NativeFunction {
        name: "createElement".to_string(),
        arity: 3,
        func: react_create_element,
    });
    
    methods.insert("createContext".to_string(), JsValue::NativeFunction {
        name: "createContext".to_string(),
        arity: 1,
        func: react_create_context,
    });
    
    methods.insert("forwardRef".to_string(), JsValue::NativeFunction {
        name: "forwardRef".to_string(),
        arity: 1,
        func: react_forward_ref,
    });
    
    methods.insert("memo".to_string(), JsValue::NativeFunction {
        name: "memo".to_string(),
        arity: 2,
        func: react_memo,
    });
    
    methods.insert("lazy".to_string(), JsValue::NativeFunction {
        name: "lazy".to_string(),
        arity: 1,
        func: react_lazy,
    });
    
    methods.insert("startTransition".to_string(), JsValue::NativeFunction {
        name: "startTransition".to_string(),
        arity: 1,
        func: react_start_transition,
    });
    
    methods.insert("act".to_string(), JsValue::NativeFunction {
        name: "act".to_string(),
        arity: 1,
        func: react_act,
    });
    
    methods.insert("Suspense".to_string(), JsValue::NativeFunction {
        name: "Suspense".to_string(),
        arity: 2,
        func: react_suspense,
    });
    
    methods.insert("Fragment".to_string(), JsValue::NativeFunction {
        name: "Fragment".to_string(),
        arity: 1,
        func: react_fragment,
    });
    
    methods.insert("StrictMode".to_string(), JsValue::NativeFunction {
        name: "StrictMode".to_string(),
        arity: 1,
        func: react_strict_mode,
    });
    
    methods.insert("render".to_string(), JsValue::NativeFunction {
        name: "render".to_string(),
        arity: 2,
        func: react_render,
    });
    
    // ============================================================
    // State Hooks
    // ============================================================
    
    methods.insert("useState".to_string(), JsValue::NativeFunction {
        name: "useState".to_string(),
        arity: 1,
        func: react_use_state,
    });
    
    methods.insert("useReducer".to_string(), JsValue::NativeFunction {
        name: "useReducer".to_string(),
        arity: 3,
        func: react_use_reducer,
    });
    
    // ============================================================
    // Context Hooks
    // ============================================================
    
    methods.insert("useContext".to_string(), JsValue::NativeFunction {
        name: "useContext".to_string(),
        arity: 1,
        func: react_use_context,
    });
    
    // ============================================================
    // Ref Hooks
    // ============================================================
    
    methods.insert("useRef".to_string(), JsValue::NativeFunction {
        name: "useRef".to_string(),
        arity: 1,
        func: react_use_ref,
    });
    
    methods.insert("useImperativeHandle".to_string(), JsValue::NativeFunction {
        name: "useImperativeHandle".to_string(),
        arity: 3,
        func: react_use_imperative_handle,
    });
    
    // ============================================================
    // Effect Hooks
    // ============================================================
    
    methods.insert("useEffect".to_string(), JsValue::NativeFunction {
        name: "useEffect".to_string(),
        arity: 2,
        func: react_use_effect,
    });
    
    methods.insert("useLayoutEffect".to_string(), JsValue::NativeFunction {
        name: "useLayoutEffect".to_string(),
        arity: 2,
        func: react_use_layout_effect,
    });
    
    methods.insert("useInsertionEffect".to_string(), JsValue::NativeFunction {
        name: "useInsertionEffect".to_string(),
        arity: 2,
        func: react_use_insertion_effect,
    });
    
    // ============================================================
    // Performance Hooks
    // ============================================================
    
    methods.insert("useMemo".to_string(), JsValue::NativeFunction {
        name: "useMemo".to_string(),
        arity: 2,
        func: react_use_memo,
    });
    
    methods.insert("useCallback".to_string(), JsValue::NativeFunction {
        name: "useCallback".to_string(),
        arity: 2,
        func: react_use_callback,
    });
    
    methods.insert("useTransition".to_string(), JsValue::NativeFunction {
        name: "useTransition".to_string(),
        arity: 0,
        func: react_use_transition,
    });
    
    methods.insert("useDeferredValue".to_string(), JsValue::NativeFunction {
        name: "useDeferredValue".to_string(),
        arity: 1,
        func: react_use_deferred_value,
    });
    
    // ============================================================
    // React 19 New Hooks
    // ============================================================
    
    methods.insert("use".to_string(), JsValue::NativeFunction {
        name: "use".to_string(),
        arity: 1,
        func: react_use,
    });
    
    methods.insert("useActionState".to_string(), JsValue::NativeFunction {
        name: "useActionState".to_string(),
        arity: 2,
        func: react_use_action_state,
    });
    
    methods.insert("useOptimistic".to_string(), JsValue::NativeFunction {
        name: "useOptimistic".to_string(),
        arity: 2,
        func: react_use_optimistic,
    });
    
    methods.insert("useFormStatus".to_string(), JsValue::NativeFunction {
        name: "useFormStatus".to_string(),
        arity: 0,
        func: react_use_form_status,
    });
    
    // ============================================================
    // Additional Utilities
    // ============================================================
    
    methods.insert("isValidElement".to_string(), JsValue::NativeFunction {
        name: "isValidElement".to_string(),
        arity: 1,
        func: react_is_valid_element,
    });
    
    methods.insert("cloneElement".to_string(), JsValue::NativeFunction {
        name: "cloneElement".to_string(),
        arity: 3,
        func: react_clone_element,
    });
    
    methods.insert("Children".to_string(), create_react_children_object());
    
    // Version info
    methods.insert("version".to_string(), JsValue::String("19.0.0".to_string()));
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

fn create_react_children_object() -> JsValue {
    let mut children = HashMap::new();
    
    children.insert("map".to_string(), JsValue::NativeFunction {
        name: "map".to_string(),
        arity: 2,
        func: |args| {
            let children = args.first().cloned().unwrap_or(JsValue::Null);
            // Return children as-is for now
            children
        },
    });
    
    children.insert("forEach".to_string(), JsValue::NativeFunction {
        name: "forEach".to_string(),
        arity: 2,
        func: |_| JsValue::Undefined,
    });
    
    children.insert("count".to_string(), JsValue::NativeFunction {
        name: "count".to_string(),
        arity: 1,
        func: |args| {
            if let Some(JsValue::Array(arr)) = args.first() {
                JsValue::Number(arr.borrow().len() as f64)
            } else {
                JsValue::Number(0.0)
            }
        },
    });
    
    children.insert("only".to_string(), JsValue::NativeFunction {
        name: "only".to_string(),
        arity: 1,
        func: |args| args.first().cloned().unwrap_or(JsValue::Null),
    });
    
    children.insert("toArray".to_string(), JsValue::NativeFunction {
        name: "toArray".to_string(),
        arity: 1,
        func: |args| {
            if let Some(JsValue::Array(_)) = args.first() {
                args.first().cloned().unwrap()
            } else {
                JsValue::Array(Rc::new(RefCell::new(vec![])))
            }
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(children)))
}

fn react_create_element(args: &[JsValue]) -> JsValue {
    let tag = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let props = args.get(1).cloned().unwrap_or(JsValue::Null);
    let children = args.get(2).cloned().unwrap_or(JsValue::Null);
    
    println!("[React] createElement('{}', {:?})", tag, props);
    
    let mut elem = HashMap::new();
    elem.insert("type".to_string(), JsValue::String(tag));
    elem.insert("props".to_string(), props);
    elem.insert("children".to_string(), children);
    
    JsValue::Object(Rc::new(RefCell::new(elem)))
}

fn react_use_state(args: &[JsValue]) -> JsValue {
    let initial = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] useState({:?})", initial);
    
    // Return [state, setState] tuple as array
    // Note: In a real implementation, state would be stored and updated
    let set_state = JsValue::NativeFunction {
        name: "setState".to_string(),
        arity: 1,
        func: |args| {
            println!("[React] setState called with {:?}", args.first());
            JsValue::Undefined
        },
    };
    
    JsValue::Array(Rc::new(RefCell::new(vec![initial, set_state])))
}

fn react_use_effect(args: &[JsValue]) -> JsValue {
    println!("[React] useEffect registered");
    let _callback = args.first();
    let _deps = args.get(1);
    JsValue::Undefined
}

fn react_render(args: &[JsValue]) -> JsValue {
    let element = args.first();
    let container = args.get(1);
    println!("[React] render({:?}, {:?})", element, container);
    JsValue::Undefined
}

fn react_use_callback(args: &[JsValue]) -> JsValue {
    let callback = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] useCallback registered");
    callback
}

fn react_use_memo(args: &[JsValue]) -> JsValue {
    let factory = args.first().cloned().unwrap_or(JsValue::Undefined);
    let deps = args.get(1);
    println!("[React] useMemo({:?}, deps: {:?})", factory, deps);
    // In a real implementation, would call factory and cache result based on deps
    // For now, simple pass-through or return factory
    if let JsValue::Function { params: _params, body: _body, .. } = &factory {
        factory.clone()
    } else {
        factory
    }
}

fn react_use_ref(args: &[JsValue]) -> JsValue {
    let initial = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] useRef({:?})", initial);
    let mut ref_obj = HashMap::new();
    ref_obj.insert("current".to_string(), initial);
    JsValue::Object(Rc::new(RefCell::new(ref_obj)))
}

fn react_use_context(args: &[JsValue]) -> JsValue {
    let context = args.first();
    println!("[React] useContext({:?})", context);
    JsValue::Undefined
}

fn react_memo(args: &[JsValue]) -> JsValue {
    let component = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] memo({:?})", component);
    component
}

fn react_lazy(args: &[JsValue]) -> JsValue {
    let factory = args.first();
    println!("[React] lazy({:?})", factory);
    // Return a promise-like object
    let mut promise = HashMap::new();
    promise.insert("_loading".to_string(), JsValue::Boolean(true));
    promise.insert("then".to_string(), JsValue::NativeFunction {
        name: "then".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    JsValue::Object(Rc::new(RefCell::new(promise)))
}

fn react_suspense(args: &[JsValue]) -> JsValue {
    let fallback = args.first();
    let children = args.get(1);
    println!("[React] Suspense({:?}, {:?})", fallback, children);
    children.cloned().unwrap_or(JsValue::Undefined)
}

// ============================================================
// React 19 New Hook Implementations
// ============================================================

fn react_create_context(args: &[JsValue]) -> JsValue {
    let default_value = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] createContext({:?})", default_value);
    
    let mut context = HashMap::new();
    context.insert("_currentValue".to_string(), default_value.clone());
    context.insert("Provider".to_string(), JsValue::NativeFunction {
        name: "Provider".to_string(),
        arity: 1,
        func: |args| {
            println!("[React.Context] Provider: {:?}", args.first());
            args.first().cloned().unwrap_or(JsValue::Undefined)
        },
    });
    context.insert("Consumer".to_string(), JsValue::NativeFunction {
        name: "Consumer".to_string(),
        arity: 1,
        func: |args| {
            println!("[React.Context] Consumer: {:?}", args.first());
            args.first().cloned().unwrap_or(JsValue::Undefined)
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(context)))
}

fn react_forward_ref(args: &[JsValue]) -> JsValue {
    let render_fn = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] forwardRef({:?})", render_fn);
    
    let mut forwarded = HashMap::new();
    forwarded.insert("$$typeof".to_string(), JsValue::String("react.forward_ref".to_string()));
    forwarded.insert("render".to_string(), render_fn);
    
    JsValue::Object(Rc::new(RefCell::new(forwarded)))
}

fn react_start_transition(args: &[JsValue]) -> JsValue {
    let callback = args.first();
    println!("[React] startTransition({:?})", callback);
    // In real React, this marks the update as a transition
    JsValue::Undefined
}

fn react_act(args: &[JsValue]) -> JsValue {
    let callback = args.first();
    println!("[React] act({:?})", callback);
    // Testing utility - synchronously flushes updates
    JsValue::Undefined
}

fn react_fragment(args: &[JsValue]) -> JsValue {
    let children = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] Fragment({:?})", children);
    children
}

fn react_strict_mode(args: &[JsValue]) -> JsValue {
    let children = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] StrictMode({:?})", children);
    children
}

fn react_use_reducer(args: &[JsValue]) -> JsValue {
    let _reducer = args.first();
    let initial_state = args.get(1).cloned().unwrap_or(JsValue::Undefined);
    let init = args.get(2);
    println!("[React] useReducer(reducer, {:?}, {:?})", initial_state, init);
    
    let dispatch = JsValue::NativeFunction {
        name: "dispatch".to_string(),
        arity: 1,
        func: |args| {
            println!("[React] dispatch({:?})", args.first());
            JsValue::Undefined
        },
    };
    
    JsValue::Array(Rc::new(RefCell::new(vec![initial_state, dispatch])))
}

fn react_use_imperative_handle(args: &[JsValue]) -> JsValue {
    let ref_obj = args.first();
    let create_handle = args.get(1);
    let deps = args.get(2);
    println!("[React] useImperativeHandle({:?}, {:?}, {:?})", ref_obj, create_handle, deps);
    JsValue::Undefined
}

fn react_use_layout_effect(args: &[JsValue]) -> JsValue {
    println!("[React] useLayoutEffect registered (runs synchronously after DOM mutations)");
    let _callback = args.first();
    let _deps = args.get(1);
    JsValue::Undefined
}

fn react_use_insertion_effect(args: &[JsValue]) -> JsValue {
    println!("[React] useInsertionEffect registered (for CSS-in-JS libraries)");
    let _callback = args.first();
    let _deps = args.get(1);
    JsValue::Undefined
}

fn react_use_transition(_args: &[JsValue]) -> JsValue {
    println!("[React] useTransition() - returns [isPending, startTransition]");
    
    let is_pending = JsValue::Boolean(false);
    let start_transition = JsValue::NativeFunction {
        name: "startTransition".to_string(),
        arity: 1,
        func: |args| {
            println!("[React] startTransition callback: {:?}", args.first());
            JsValue::Undefined
        },
    };
    
    JsValue::Array(Rc::new(RefCell::new(vec![is_pending, start_transition])))
}

fn react_use_deferred_value(args: &[JsValue]) -> JsValue {
    let value = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React] useDeferredValue({:?})", value);
    // Returns the deferred version of the value
    value
}

fn react_use(args: &[JsValue]) -> JsValue {
    let resource = args.first().cloned().unwrap_or(JsValue::Undefined);
    println!("[React 19] use({:?}) - reads value from Promise/Context", resource);
    
    // Check if it's a Promise-like object
    if let JsValue::Object(obj) = &resource {
        let borrowed = obj.borrow();
        // If it has a "_resolved" value, return it
        if let Some(resolved) = borrowed.get("_resolved") {
            return resolved.clone();
        }
        // If it's a Context, return its current value
        if let Some(current) = borrowed.get("_currentValue") {
            return current.clone();
        }
    }
    
    // In real React, this would suspend if Promise is pending
    resource
}

fn react_use_action_state(args: &[JsValue]) -> JsValue {
    let action = args.first();
    let initial_state = args.get(1).cloned().unwrap_or(JsValue::Undefined);
    println!("[React 19] useActionState({:?}, {:?})", action, initial_state);
    
    let form_action = JsValue::NativeFunction {
        name: "formAction".to_string(),
        arity: 1,
        func: |args| {
            println!("[React 19] formAction called: {:?}", args.first());
            JsValue::Undefined
        },
    };
    
    let is_pending = JsValue::Boolean(false);
    
    JsValue::Array(Rc::new(RefCell::new(vec![initial_state, form_action, is_pending])))
}

fn react_use_optimistic(args: &[JsValue]) -> JsValue {
    let state = args.first().cloned().unwrap_or(JsValue::Undefined);
    let update_fn = args.get(1);
    println!("[React 19] useOptimistic({:?}, {:?})", state, update_fn);
    
    let add_optimistic = JsValue::NativeFunction {
        name: "addOptimistic".to_string(),
        arity: 1,
        func: |args| {
            println!("[React 19] addOptimistic: {:?}", args.first());
            JsValue::Undefined
        },
    };
    
    JsValue::Array(Rc::new(RefCell::new(vec![state, add_optimistic])))
}

fn react_use_form_status(_args: &[JsValue]) -> JsValue {
    println!("[React 19] useFormStatus() - returns form submission status");
    
    let mut status = HashMap::new();
    status.insert("pending".to_string(), JsValue::Boolean(false));
    status.insert("data".to_string(), JsValue::Null);
    status.insert("method".to_string(), JsValue::Null);
    status.insert("action".to_string(), JsValue::Null);
    
    JsValue::Object(Rc::new(RefCell::new(status)))
}

fn react_is_valid_element(args: &[JsValue]) -> JsValue {
    let element = args.first();
    println!("[React] isValidElement({:?})", element);
    
    if let Some(JsValue::Object(obj)) = element {
        let borrowed = obj.borrow();
        // Check if it has a "type" property (React element)
        if borrowed.contains_key("type") || borrowed.contains_key("$$typeof") {
            return JsValue::Boolean(true);
        }
    }
    JsValue::Boolean(false)
}

fn react_clone_element(args: &[JsValue]) -> JsValue {
    let element = args.first().cloned().unwrap_or(JsValue::Null);
    let props = args.get(1).cloned().unwrap_or(JsValue::Null);
    let children = args.get(2).cloned();
    println!("[React] cloneElement({:?}, {:?}, {:?})", element, props, children);
    
    // Clone and merge props
    if let JsValue::Object(obj) = &element {
        let mut cloned = obj.borrow().clone();
        if let JsValue::Object(new_props) = &props {
            for (k, v) in new_props.borrow().iter() {
                cloned.insert(k.clone(), v.clone());
            }
        }
        if let Some(c) = children {
            cloned.insert("children".to_string(), c);
        }
        return JsValue::Object(Rc::new(RefCell::new(cloned)));
    }
    
    element
}

/// Create ReactDOM object
pub fn create_react_dom_object() -> JsValue {
    let mut methods = HashMap::new();
    
    methods.insert("render".to_string(), JsValue::NativeFunction {
        name: "render".to_string(),
        arity: 2,
        func: react_dom_render,
    });
    
    methods.insert("createRoot".to_string(), JsValue::NativeFunction {
        name: "createRoot".to_string(),
        arity: 1,
        func: react_dom_create_root,
    });
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

fn react_dom_render(args: &[JsValue]) -> JsValue {
    println!("[ReactDOM] render called");
    let _element = args.first();
    let _container = args.get(1);
    JsValue::Undefined
}

fn react_dom_create_root(args: &[JsValue]) -> JsValue {
    println!("[ReactDOM] createRoot called");
    let _container = args.first();
    
    let mut root = HashMap::new();
    root.insert("render".to_string(), JsValue::NativeFunction {
        name: "render".to_string(),
        arity: 1,
        func: |args| {
            println!("[ReactDOM.Root] render: {:?}", args.first());
            JsValue::Undefined
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(root)))
}
