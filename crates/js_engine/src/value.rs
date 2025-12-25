//! JavaScript Value System
//!
//! Represents JavaScript runtime values including primitives and objects.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use crate::ast::Statement;

/// JavaScript Runtime Value
#[derive(Clone)]
pub enum JsValue {
    /// undefined
    Undefined,
    
    /// null
    Null,
    
    /// Boolean value
    Boolean(bool),
    
    /// Number (IEEE 754 double)
    Number(f64),
    
    /// String
    String(String),
    
    /// Object (key-value pairs)
    Object(Rc<RefCell<HashMap<String, JsValue>>>),
    
    /// Array
    Array(Rc<RefCell<Vec<JsValue>>>),
    
    /// Function (user-defined)
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    
    /// Native function (built-in)
    NativeFunction {
        name: String,
        arity: usize,
        func: fn(&[JsValue]) -> JsValue,
    },
    
    /// Symbol (ES6)
    Symbol {
        id: u64,
        description: Option<String>,
    },
    
    /// BigInt (ES2020)
    BigInt(i128),
    
    /// RegExp
    RegExp {
        pattern: String,
        flags: String,
    },
    
    /// Date
    Date(f64),  // milliseconds since epoch
    
    /// Error
    Error {
        kind: String,
        message: String,
    },
    
    /// Promise (simplified)
    Promise {
        state: PromiseState,
        value: Box<JsValue>,
    },
    
    /// Map (ES6)
    Map(Rc<RefCell<Vec<(JsValue, JsValue)>>>),
    
    /// Set (ES6)
    Set(Rc<RefCell<Vec<JsValue>>>),
    
    /// ArrayBuffer
    ArrayBuffer(Rc<RefCell<Vec<u8>>>),
    
    /// TypedArray (simplified - using u8)
    TypedArray {
        kind: TypedArrayKind,
        buffer: Rc<RefCell<Vec<u8>>>,
    },
}

/// Promise states
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}

/// TypedArray kinds
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypedArrayKind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
    BigInt64,
    BigUint64,
}

impl JsValue {
    /// Convert to JavaScript string representation
    pub fn to_js_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Number(n) => {
                if n.is_nan() {
                    "NaN".to_string()
                } else if n.is_infinite() {
                    if *n > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
                } else if *n == 0.0 {
                    "0".to_string()
                } else {
                    format!("{}", n)
                }
            }
            JsValue::String(s) => s.clone(),
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Array(arr) => {
                let items: Vec<String> = arr.borrow().iter()
                    .map(|v| v.to_js_string())
                    .collect();
                items.join(",")
            }
            JsValue::Function { name, .. } => {
                format!("function {}() {{ [code] }}", name.as_deref().unwrap_or("anonymous"))
            }
            JsValue::NativeFunction { name, .. } => {
                format!("function {}() {{ [native code] }}", name)
            }
            JsValue::Symbol { description, .. } => {
                format!("Symbol({})", description.as_deref().unwrap_or(""))
            }
            JsValue::BigInt(n) => format!("{}n", n),
            JsValue::RegExp { pattern, flags } => format!("/{}/{}", pattern, flags),
            JsValue::Date(ms) => format!("Date({})", ms),
            JsValue::Error { kind, message } => format!("{}: {}", kind, message),
            JsValue::Promise { state, .. } => format!("Promise {{ {:?} }}", state),
            JsValue::Map(m) => format!("Map({})", m.borrow().len()),
            JsValue::Set(s) => format!("Set({})", s.borrow().len()),
            JsValue::ArrayBuffer(buf) => format!("ArrayBuffer({})", buf.borrow().len()),
            JsValue::TypedArray { kind, buffer } => {
                format!("{:?}Array({})", kind, buffer.borrow().len())
            }
        }
    }
    
    /// Convert to number
    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Undefined => f64::NAN,
            JsValue::Null => 0.0,
            JsValue::Boolean(true) => 1.0,
            JsValue::Boolean(false) => 0.0,
            JsValue::Number(n) => *n,
            JsValue::String(s) => s.trim().parse::<f64>().unwrap_or(f64::NAN),
            JsValue::BigInt(n) => *n as f64,
            JsValue::Date(ms) => *ms,
            _ => f64::NAN,
        }
    }
    
    /// Convert to boolean (truthiness)
    pub fn is_truthy(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Boolean(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            JsValue::BigInt(n) => *n != 0,
            _ => true,
        }
    }
    
    /// Get JavaScript typeof
    pub fn type_of(&self) -> &'static str {
        match self {
            JsValue::Undefined => "undefined",
            JsValue::Null => "object",  // Historical quirk
            JsValue::Boolean(_) => "boolean",
            JsValue::Number(_) => "number",
            JsValue::String(_) => "string",
            JsValue::Symbol { .. } => "symbol",
            JsValue::BigInt(_) => "bigint",
            JsValue::Function { .. } | JsValue::NativeFunction { .. } => "function",
            _ => "object",
        }
    }
    
    /// Check if value is nullish (null or undefined)
    pub fn is_nullish(&self) -> bool {
        matches!(self, JsValue::Null | JsValue::Undefined)
    }
    
    /// Check if value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, 
            JsValue::Object(_) | JsValue::Array(_) | JsValue::Map(_) | 
            JsValue::Set(_) | JsValue::Promise { .. } | JsValue::Error { .. } |
            JsValue::RegExp { .. } | JsValue::Date(_) | JsValue::ArrayBuffer(_) |
            JsValue::TypedArray { .. }
        )
    }
    
    /// Check if value is callable
    pub fn is_callable(&self) -> bool {
        matches!(self, JsValue::Function { .. } | JsValue::NativeFunction { .. })
    }
    
    /// Get property from object
    pub fn get_property(&self, key: &str) -> JsValue {
        match self {
            JsValue::Object(map) => {
                map.borrow().get(key).cloned().unwrap_or(JsValue::Undefined)
            }
            JsValue::Array(arr) => {
                if key == "length" {
                    JsValue::Number(arr.borrow().len() as f64)
                } else if let Ok(idx) = key.parse::<usize>() {
                    arr.borrow().get(idx).cloned().unwrap_or(JsValue::Undefined)
                } else {
                    JsValue::Undefined
                }
            }
            JsValue::String(s) => {
                if key == "length" {
                    JsValue::Number(s.chars().count() as f64)
                } else if let Ok(idx) = key.parse::<usize>() {
                    s.chars().nth(idx)
                        .map(|c| JsValue::String(c.to_string()))
                        .unwrap_or(JsValue::Undefined)
                } else {
                    JsValue::Undefined
                }
            }
            JsValue::Map(map) => {
                for (k, v) in map.borrow().iter() {
                    if k.to_js_string() == key {
                        return v.clone();
                    }
                }
                JsValue::Undefined
            }
            _ => JsValue::Undefined,
        }
    }
    
    /// Set property on object
    pub fn set_property(&self, key: &str, value: JsValue) -> bool {
        match self {
            JsValue::Object(map) => {
                map.borrow_mut().insert(key.to_string(), value);
                true
            }
            JsValue::Array(arr) => {
                if let Ok(idx) = key.parse::<usize>() {
                    let mut arr = arr.borrow_mut();
                    while arr.len() <= idx {
                        arr.push(JsValue::Undefined);
                    }
                    arr[idx] = value;
                    true
                } else if key == "length" {
                    if let JsValue::Number(n) = value {
                        arr.borrow_mut().truncate(n as usize);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// Create a new empty object
    pub fn new_object() -> Self {
        JsValue::Object(Rc::new(RefCell::new(HashMap::new())))
    }
    
    /// Create a new empty array
    pub fn new_array() -> Self {
        JsValue::Array(Rc::new(RefCell::new(Vec::new())))
    }
    
    /// Create a new Map
    pub fn new_map() -> Self {
        JsValue::Map(Rc::new(RefCell::new(Vec::new())))
    }
    
    /// Create a new Set
    pub fn new_set() -> Self {
        JsValue::Set(Rc::new(RefCell::new(Vec::new())))
    }
    
    /// Create a resolved Promise
    pub fn resolved_promise(value: JsValue) -> Self {
        JsValue::Promise {
            state: PromiseState::Fulfilled,
            value: Box::new(value),
        }
    }
    
    /// Create a rejected Promise
    pub fn rejected_promise(reason: JsValue) -> Self {
        JsValue::Promise {
            state: PromiseState::Rejected,
            value: Box::new(reason),
        }
    }
    
    /// Create a new Symbol
    pub fn new_symbol(description: Option<String>) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        JsValue::Symbol {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            description,
        }
    }
    
    /// Create a new Date from timestamp
    pub fn new_date(ms: f64) -> Self {
        JsValue::Date(ms)
    }
    
    /// Create current date
    pub fn date_now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);
        JsValue::Date(ms)
    }
    
    /// Create a new Error
    pub fn new_error(kind: &str, message: &str) -> Self {
        JsValue::Error {
            kind: kind.to_string(),
            message: message.to_string(),
        }
    }
    
    /// Create a TypeError
    pub fn type_error(message: &str) -> Self {
        Self::new_error("TypeError", message)
    }
    
    /// Create a ReferenceError  
    pub fn reference_error(message: &str) -> Self {
        Self::new_error("ReferenceError", message)
    }
    
    /// Create a SyntaxError
    pub fn syntax_error(message: &str) -> Self {
        Self::new_error("SyntaxError", message)
    }
    
    /// Create a RangeError
    pub fn range_error(message: &str) -> Self {
        Self::new_error("RangeError", message)
    }
}

impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => {
                if a.is_nan() && b.is_nan() { return false; }
                a == b
            }
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::BigInt(a), JsValue::BigInt(b)) => a == b,
            (JsValue::Symbol { id: a, .. }, JsValue::Symbol { id: b, .. }) => a == b,
            (JsValue::Object(a), JsValue::Object(b)) => Rc::ptr_eq(a, b),
            (JsValue::Array(a), JsValue::Array(b)) => Rc::ptr_eq(a, b),
            (JsValue::Map(a), JsValue::Map(b)) => Rc::ptr_eq(a, b),
            (JsValue::Set(a), JsValue::Set(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Debug for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsValue::Undefined => write!(f, "undefined"),
            JsValue::Null => write!(f, "null"),
            JsValue::Boolean(b) => write!(f, "{}", b),
            JsValue::Number(n) => write!(f, "{}", n),
            JsValue::String(s) => write!(f, "\"{}\"", s),
            JsValue::Object(_) => write!(f, "[object Object]"),
            JsValue::Array(arr) => write!(f, "{:?}", arr.borrow()),
            JsValue::Function { name, .. } => {
                write!(f, "function {}()", name.as_deref().unwrap_or("anonymous"))
            }
            JsValue::NativeFunction { name, .. } => {
                write!(f, "function {}() {{ [native code] }}", name)
            }
            JsValue::Symbol { description, .. } => {
                write!(f, "Symbol({})", description.as_deref().unwrap_or(""))
            }
            JsValue::BigInt(n) => write!(f, "{}n", n),
            JsValue::RegExp { pattern, flags } => write!(f, "/{}/{}", pattern, flags),
            JsValue::Date(ms) => write!(f, "Date({})", ms),
            JsValue::Error { kind, message } => write!(f, "{}: {}", kind, message),
            JsValue::Promise { state, .. } => write!(f, "Promise {{ {:?} }}", state),
            JsValue::Map(m) => write!(f, "Map({})", m.borrow().len()),
            JsValue::Set(s) => write!(f, "Set({})", s.borrow().len()),
            JsValue::ArrayBuffer(buf) => write!(f, "ArrayBuffer({})", buf.borrow().len()),
            JsValue::TypedArray { kind, buffer } => {
                write!(f, "{:?}Array({})", kind, buffer.borrow().len())
            }
        }
    }
}

impl Default for JsValue {
    fn default() -> Self {
        JsValue::Undefined
    }
}
