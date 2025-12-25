//! JavaScript Builtins
//!
//! Built-in functions and objects: console, Math, JSON, Array, Object, etc.

use crate::value::JsValue;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Mutex;

// Storage for localStorage/sessionStorage simulation
lazy_static::lazy_static! {
    static ref LOCAL_STORAGE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref SESSION_STORAGE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// Create console object
pub fn create_console() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("log".to_string(), JsValue::NativeFunction {
        name: "log".to_string(),
        arity: 0,
        func: console_log,
    });

    methods.insert("warn".to_string(), JsValue::NativeFunction {
        name: "warn".to_string(),
        arity: 0,
        func: console_warn,
    });

    methods.insert("error".to_string(), JsValue::NativeFunction {
        name: "error".to_string(),
        arity: 0,
        func: console_error,
    });

    methods.insert("info".to_string(), JsValue::NativeFunction {
        name: "info".to_string(),
        arity: 0,
        func: console_info,
    });

    methods.insert("debug".to_string(), JsValue::NativeFunction {
        name: "debug".to_string(),
        arity: 0,
        func: console_log,
    });

    methods.insert("trace".to_string(), JsValue::NativeFunction {
        name: "trace".to_string(),
        arity: 0,
        func: console_log,
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

    methods.insert("clear".to_string(), JsValue::NativeFunction {
        name: "clear".to_string(),
        arity: 0,
        func: |_| {
            println!("[Console cleared]");
            JsValue::Undefined
        },
    });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

fn console_log(args: &[JsValue]) -> JsValue {
    let output: Vec<String> = args.iter().map(|v| v.to_js_string()).collect();
    println!("[JS] {}", output.join(" "));
    JsValue::Undefined
}

fn console_warn(args: &[JsValue]) -> JsValue {
    let output: Vec<String> = args.iter().map(|v| v.to_js_string()).collect();
    println!("[JS WARN] {}", output.join(" "));
    JsValue::Undefined
}

fn console_error(args: &[JsValue]) -> JsValue {
    let output: Vec<String> = args.iter().map(|v| v.to_js_string()).collect();
    eprintln!("[JS ERROR] {}", output.join(" "));
    JsValue::Undefined
}

fn console_info(args: &[JsValue]) -> JsValue {
    let output: Vec<String> = args.iter().map(|v| v.to_js_string()).collect();
    println!("[JS INFO] {}", output.join(" "));
    JsValue::Undefined
}

/// Create Math object
pub fn create_math() -> JsValue {
    let mut methods = HashMap::new();

    // Constants
    methods.insert("PI".to_string(), JsValue::Number(std::f64::consts::PI));
    methods.insert("E".to_string(), JsValue::Number(std::f64::consts::E));
    methods.insert("LN2".to_string(), JsValue::Number(std::f64::consts::LN_2));
    methods.insert("LN10".to_string(), JsValue::Number(std::f64::consts::LN_10));
    methods.insert("LOG2E".to_string(), JsValue::Number(std::f64::consts::LOG2_E));
    methods.insert("LOG10E".to_string(), JsValue::Number(std::f64::consts::LOG10_E));
    methods.insert("SQRT2".to_string(), JsValue::Number(std::f64::consts::SQRT_2));
    methods.insert("SQRT1_2".to_string(), JsValue::Number(std::f64::consts::FRAC_1_SQRT_2));

    // Functions
    methods.insert("floor".to_string(), JsValue::NativeFunction { name: "floor".to_string(), arity: 1, func: math_floor });
    methods.insert("ceil".to_string(), JsValue::NativeFunction { name: "ceil".to_string(), arity: 1, func: math_ceil });
    methods.insert("round".to_string(), JsValue::NativeFunction { name: "round".to_string(), arity: 1, func: math_round });
    methods.insert("abs".to_string(), JsValue::NativeFunction { name: "abs".to_string(), arity: 1, func: math_abs });
    methods.insert("sqrt".to_string(), JsValue::NativeFunction { name: "sqrt".to_string(), arity: 1, func: math_sqrt });
    methods.insert("pow".to_string(), JsValue::NativeFunction { name: "pow".to_string(), arity: 2, func: math_pow });
    methods.insert("random".to_string(), JsValue::NativeFunction { name: "random".to_string(), arity: 0, func: math_random });
    methods.insert("min".to_string(), JsValue::NativeFunction { name: "min".to_string(), arity: 0, func: math_min });
    methods.insert("max".to_string(), JsValue::NativeFunction { name: "max".to_string(), arity: 0, func: math_max });
    methods.insert("sin".to_string(), JsValue::NativeFunction { name: "sin".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().sin()).unwrap_or(f64::NAN)) } });
    methods.insert("cos".to_string(), JsValue::NativeFunction { name: "cos".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().cos()).unwrap_or(f64::NAN)) } });
    methods.insert("tan".to_string(), JsValue::NativeFunction { name: "tan".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().tan()).unwrap_or(f64::NAN)) } });
    methods.insert("asin".to_string(), JsValue::NativeFunction { name: "asin".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().asin()).unwrap_or(f64::NAN)) } });
    methods.insert("acos".to_string(), JsValue::NativeFunction { name: "acos".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().acos()).unwrap_or(f64::NAN)) } });
    methods.insert("atan".to_string(), JsValue::NativeFunction { name: "atan".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().atan()).unwrap_or(f64::NAN)) } });
    methods.insert("atan2".to_string(), JsValue::NativeFunction { name: "atan2".to_string(), arity: 2, func: |args| { let y = args.first().map(|v| v.to_number()).unwrap_or(0.0); let x = args.get(1).map(|v| v.to_number()).unwrap_or(0.0); JsValue::Number(y.atan2(x)) } });
    methods.insert("log".to_string(), JsValue::NativeFunction { name: "log".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().ln()).unwrap_or(f64::NAN)) } });
    methods.insert("log10".to_string(), JsValue::NativeFunction { name: "log10".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().log10()).unwrap_or(f64::NAN)) } });
    methods.insert("log2".to_string(), JsValue::NativeFunction { name: "log2".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().log2()).unwrap_or(f64::NAN)) } });
    methods.insert("exp".to_string(), JsValue::NativeFunction { name: "exp".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().exp()).unwrap_or(f64::NAN)) } });
    methods.insert("sign".to_string(), JsValue::NativeFunction { name: "sign".to_string(), arity: 1, func: |args| { let n = args.first().map(|v| v.to_number()).unwrap_or(0.0); JsValue::Number(if n > 0.0 { 1.0 } else if n < 0.0 { -1.0 } else { 0.0 }) } });
    methods.insert("trunc".to_string(), JsValue::NativeFunction { name: "trunc".to_string(), arity: 1, func: |args| { JsValue::Number(args.first().map(|v| v.to_number().trunc()).unwrap_or(f64::NAN)) } });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

fn math_floor(args: &[JsValue]) -> JsValue { JsValue::Number(args.first().map(|v| v.to_number()).unwrap_or(f64::NAN).floor()) }
fn math_ceil(args: &[JsValue]) -> JsValue { JsValue::Number(args.first().map(|v| v.to_number()).unwrap_or(f64::NAN).ceil()) }
fn math_round(args: &[JsValue]) -> JsValue { JsValue::Number(args.first().map(|v| v.to_number()).unwrap_or(f64::NAN).round()) }
fn math_abs(args: &[JsValue]) -> JsValue { JsValue::Number(args.first().map(|v| v.to_number()).unwrap_or(f64::NAN).abs()) }
fn math_sqrt(args: &[JsValue]) -> JsValue { JsValue::Number(args.first().map(|v| v.to_number()).unwrap_or(f64::NAN).sqrt()) }
fn math_pow(args: &[JsValue]) -> JsValue {
    let base = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    let exp = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    JsValue::Number(base.powf(exp))
}

fn math_random(_args: &[JsValue]) -> JsValue {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
    let random = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) >> 16) as f64 / 32768.0;
    JsValue::Number(random % 1.0)
}

fn math_min(args: &[JsValue]) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::INFINITY); }
    JsValue::Number(args.iter().map(|v| v.to_number()).fold(f64::INFINITY, f64::min))
}

fn math_max(args: &[JsValue]) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::NEG_INFINITY); }
    JsValue::Number(args.iter().map(|v| v.to_number()).fold(f64::NEG_INFINITY, f64::max))
}

/// Create JSON object
pub fn create_json() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("parse".to_string(), JsValue::NativeFunction {
        name: "parse".to_string(),
        arity: 1,
        func: json_parse,
    });

    methods.insert("stringify".to_string(), JsValue::NativeFunction {
        name: "stringify".to_string(),
        arity: 1,
        func: json_stringify,
    });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

fn json_parse(args: &[JsValue]) -> JsValue {
    let s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    parse_json_value(&s).unwrap_or(JsValue::Null)
}

fn parse_json_value(s: &str) -> Option<JsValue> {
    let s = s.trim();
    if s == "null" { return Some(JsValue::Null); }
    if s == "true" { return Some(JsValue::Boolean(true)); }
    if s == "false" { return Some(JsValue::Boolean(false)); }
    if let Ok(n) = s.parse::<f64>() { return Some(JsValue::Number(n)); }
    if s.starts_with('"') && s.ends_with('"') {
        return Some(JsValue::String(s[1..s.len()-1].to_string()));
    }
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len()-1];
        let items: Vec<JsValue> = inner.split(',')
            .filter_map(|item| parse_json_value(item.trim()))
            .collect();
        return Some(JsValue::Array(Rc::new(RefCell::new(items))));
    }
    if s.starts_with('{') && s.ends_with('}') {
        let mut map = HashMap::new();
        let inner = &s[1..s.len()-1];
        for pair in inner.split(',') {
            let parts: Vec<&str> = pair.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().trim_matches('"');
                if let Some(val) = parse_json_value(parts[1]) {
                    map.insert(key.to_string(), val);
                }
            }
        }
        return Some(JsValue::Object(Rc::new(RefCell::new(map))));
    }
    None
}

fn json_stringify(args: &[JsValue]) -> JsValue {
    let val = args.first().unwrap_or(&JsValue::Undefined);
    JsValue::String(stringify_json_value(val))
}

fn stringify_json_value(val: &JsValue) -> String {
    match val {
        JsValue::Null => "null".to_string(),
        JsValue::Undefined => "undefined".to_string(),
        JsValue::Boolean(b) => b.to_string(),
        JsValue::Number(n) => n.to_string(),
        JsValue::String(s) => format!("\"{}\"", s),
        JsValue::Array(arr) => {
            let items: Vec<String> = arr.borrow().iter().map(stringify_json_value).collect();
            format!("[{}]", items.join(","))
        }
        JsValue::Object(map) => {
            let pairs: Vec<String> = map.borrow().iter()
                .map(|(k, v)| format!("\"{}\":{}", k, stringify_json_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        _ => "null".to_string(),
    }
}

/// Create Object object with static methods
pub fn create_object_constructor() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("keys".to_string(), JsValue::NativeFunction {
        name: "keys".to_string(),
        arity: 1,
        func: |args| {
            if let Some(JsValue::Object(obj)) = args.first() {
                let keys: Vec<JsValue> = obj.borrow().keys().map(|k| JsValue::String(k.clone())).collect();
                JsValue::Array(Rc::new(RefCell::new(keys)))
            } else {
                JsValue::Array(Rc::new(RefCell::new(vec![])))
            }
        },
    });

    methods.insert("values".to_string(), JsValue::NativeFunction {
        name: "values".to_string(),
        arity: 1,
        func: |args| {
            if let Some(JsValue::Object(obj)) = args.first() {
                let values: Vec<JsValue> = obj.borrow().values().cloned().collect();
                JsValue::Array(Rc::new(RefCell::new(values)))
            } else {
                JsValue::Array(Rc::new(RefCell::new(vec![])))
            }
        },
    });

    methods.insert("entries".to_string(), JsValue::NativeFunction {
        name: "entries".to_string(),
        arity: 1,
        func: |args| {
            if let Some(JsValue::Object(obj)) = args.first() {
                let entries: Vec<JsValue> = obj.borrow().iter()
                    .map(|(k, v)| JsValue::Array(Rc::new(RefCell::new(vec![JsValue::String(k.clone()), v.clone()]))))
                    .collect();
                JsValue::Array(Rc::new(RefCell::new(entries)))
            } else {
                JsValue::Array(Rc::new(RefCell::new(vec![])))
            }
        },
    });

    methods.insert("assign".to_string(), JsValue::NativeFunction {
        name: "assign".to_string(),
        arity: 2,
        func: |args| {
            if let Some(JsValue::Object(target)) = args.first().cloned() {
                for source in args.iter().skip(1) {
                    if let JsValue::Object(src) = source {
                        for (k, v) in src.borrow().iter() {
                            target.borrow_mut().insert(k.clone(), v.clone());
                        }
                    }
                }
                JsValue::Object(target)
            } else {
                JsValue::Undefined
            }
        },
    });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create Array constructor with static methods
pub fn create_array_constructor() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("isArray".to_string(), JsValue::NativeFunction {
        name: "isArray".to_string(),
        arity: 1,
        func: |args| JsValue::Boolean(matches!(args.first(), Some(JsValue::Array(_)))),
    });

    methods.insert("from".to_string(), JsValue::NativeFunction {
        name: "from".to_string(),
        arity: 1,
        func: |args| {
            match args.first() {
                Some(JsValue::Array(arr)) => JsValue::Array(Rc::new(RefCell::new(arr.borrow().clone()))),
                Some(JsValue::String(s)) => {
                    let chars: Vec<JsValue> = s.chars().map(|c| JsValue::String(c.to_string())).collect();
                    JsValue::Array(Rc::new(RefCell::new(chars)))
                }
                _ => JsValue::Array(Rc::new(RefCell::new(vec![]))),
            }
        },
    });

    methods.insert("of".to_string(), JsValue::NativeFunction {
        name: "of".to_string(),
        arity: 0,
        func: |args| JsValue::Array(Rc::new(RefCell::new(args.to_vec()))),
    });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create String constructor
pub fn create_string_constructor() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("fromCharCode".to_string(), JsValue::NativeFunction {
        name: "fromCharCode".to_string(),
        arity: 0,
        func: |args| {
            let s: String = args.iter()
                .filter_map(|v| {
                    let code = v.to_number() as u32;
                    char::from_u32(code)
                })
                .collect();
            JsValue::String(s)
        },
    });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create Number constructor
pub fn create_number_constructor() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("isNaN".to_string(), JsValue::NativeFunction {
        name: "isNaN".to_string(),
        arity: 1,
        func: |args| {
            let n = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
            JsValue::Boolean(n.is_nan())
        },
    });

    methods.insert("isFinite".to_string(), JsValue::NativeFunction {
        name: "isFinite".to_string(),
        arity: 1,
        func: |args| {
            let n = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
            JsValue::Boolean(n.is_finite())
        },
    });

    methods.insert("isInteger".to_string(), JsValue::NativeFunction {
        name: "isInteger".to_string(),
        arity: 1,
        func: |args| {
            let n = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
            JsValue::Boolean(n.fract() == 0.0 && n.is_finite())
        },
    });

    methods.insert("MAX_VALUE".to_string(), JsValue::Number(f64::MAX));
    methods.insert("MIN_VALUE".to_string(), JsValue::Number(f64::MIN_POSITIVE));
    methods.insert("NaN".to_string(), JsValue::Number(f64::NAN));
    methods.insert("POSITIVE_INFINITY".to_string(), JsValue::Number(f64::INFINITY));
    methods.insert("NEGATIVE_INFINITY".to_string(), JsValue::Number(f64::NEG_INFINITY));

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create localStorage object
pub fn create_local_storage() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("getItem".to_string(), JsValue::NativeFunction {
        name: "getItem".to_string(),
        arity: 1,
        func: |args| {
            let key = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            if let Ok(storage) = LOCAL_STORAGE.lock() {
                storage.get(&key).map(|s| JsValue::String(s.clone())).unwrap_or(JsValue::Null)
            } else {
                JsValue::Null
            }
        },
    });

    methods.insert("setItem".to_string(), JsValue::NativeFunction {
        name: "setItem".to_string(),
        arity: 2,
        func: |args| {
            let key = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            let value = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();
            if let Ok(mut storage) = LOCAL_STORAGE.lock() {
                storage.insert(key, value);
            }
            JsValue::Undefined
        },
    });

    methods.insert("removeItem".to_string(), JsValue::NativeFunction {
        name: "removeItem".to_string(),
        arity: 1,
        func: |args| {
            let key = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            if let Ok(mut storage) = LOCAL_STORAGE.lock() {
                storage.remove(&key);
            }
            JsValue::Undefined
        },
    });

    methods.insert("clear".to_string(), JsValue::NativeFunction {
        name: "clear".to_string(),
        arity: 0,
        func: |_| {
            if let Ok(mut storage) = LOCAL_STORAGE.lock() {
                storage.clear();
            }
            JsValue::Undefined
        },
    });

    methods.insert("key".to_string(), JsValue::NativeFunction {
        name: "key".to_string(),
        arity: 1,
        func: |args| {
            let index = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            if let Ok(storage) = LOCAL_STORAGE.lock() {
                storage.keys().nth(index).map(|k| JsValue::String(k.clone())).unwrap_or(JsValue::Null)
            } else {
                JsValue::Null
            }
        },
    });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create sessionStorage object
pub fn create_session_storage() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("getItem".to_string(), JsValue::NativeFunction {
        name: "getItem".to_string(),
        arity: 1,
        func: |args| {
            let key = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            if let Ok(storage) = SESSION_STORAGE.lock() {
                storage.get(&key).map(|s| JsValue::String(s.clone())).unwrap_or(JsValue::Null)
            } else {
                JsValue::Null
            }
        },
    });

    methods.insert("setItem".to_string(), JsValue::NativeFunction {
        name: "setItem".to_string(),
        arity: 2,
        func: |args| {
            let key = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            let value = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();
            if let Ok(mut storage) = SESSION_STORAGE.lock() {
                storage.insert(key, value);
            }
            JsValue::Undefined
        },
    });

    methods.insert("removeItem".to_string(), JsValue::NativeFunction {
        name: "removeItem".to_string(),
        arity: 1,
        func: |args| {
            let key = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            if let Ok(mut storage) = SESSION_STORAGE.lock() {
                storage.remove(&key);
            }
            JsValue::Undefined
        },
    });

    methods.insert("clear".to_string(), JsValue::NativeFunction {
        name: "clear".to_string(),
        arity: 0,
        func: |_| {
            if let Ok(mut storage) = SESSION_STORAGE.lock() {
                storage.clear();
            }
            JsValue::Undefined
        },
    });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create Date constructor
pub fn create_date_constructor() -> JsValue {
    JsValue::NativeFunction {
        name: "Date".to_string(),
        arity: 0,
        func: |_| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64;

            let mut obj = HashMap::new();
            obj.insert("timestamp".to_string(), JsValue::Number(now));

            obj.insert("getTime".to_string(), JsValue::NativeFunction {
                name: "getTime".to_string(),
                arity: 0,
                func: |_| {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as f64;
                    JsValue::Number(now)
                },
            });

            obj.insert("toISOString".to_string(), JsValue::NativeFunction {
                name: "toISOString".to_string(),
                arity: 0,
                func: |_| JsValue::String("2024-01-01T00:00:00.000Z".to_string()),
            });

            JsValue::Object(Rc::new(RefCell::new(obj)))
        },
    }
}

/// Create Promise constructor
pub fn create_promise_constructor() -> JsValue {
    let mut methods = HashMap::new();

    methods.insert("resolve".to_string(), JsValue::NativeFunction {
        name: "resolve".to_string(),
        arity: 1,
        func: |args| {
            let value = args.first().cloned().unwrap_or(JsValue::Undefined);
            let mut obj = HashMap::new();
            obj.insert("[[PromiseState]]".to_string(), JsValue::String("fulfilled".to_string()));
            obj.insert("[[PromiseResult]]".to_string(), value);
            obj.insert("then".to_string(), JsValue::NativeFunction {
                name: "then".to_string(),
                arity: 1,
                func: |args| args.first().cloned().unwrap_or(JsValue::Undefined),
            });
            JsValue::Object(Rc::new(RefCell::new(obj)))
        },
    });

    methods.insert("reject".to_string(), JsValue::NativeFunction {
        name: "reject".to_string(),
        arity: 1,
        func: |args| {
            let value = args.first().cloned().unwrap_or(JsValue::Undefined);
            let mut obj = HashMap::new();
            obj.insert("[[PromiseState]]".to_string(), JsValue::String("rejected".to_string()));
            obj.insert("[[PromiseResult]]".to_string(), value);
            JsValue::Object(Rc::new(RefCell::new(obj)))
        },
    });

    methods.insert("all".to_string(), JsValue::NativeFunction {
        name: "all".to_string(),
        arity: 1,
        func: |args| {
            let arr = match args.first() {
                Some(JsValue::Array(a)) => a.borrow().clone(),
                _ => vec![],
            };
            let mut result = HashMap::new();
            result.insert("[[PromiseState]]".to_string(), JsValue::String("fulfilled".to_string()));
            result.insert("[[PromiseResult]]".to_string(), JsValue::Array(Rc::new(RefCell::new(arr))));
            JsValue::Object(Rc::new(RefCell::new(result)))
        },
    });

    JsValue::Object(Rc::new(RefCell::new(methods)))
}

/// Create fetch function
pub fn create_fetch() -> JsValue {
    JsValue::NativeFunction {
        name: "fetch".to_string(),
        arity: 1,
        func: |args| {
            let url = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[Fetch] {}", url);

            let mut response = HashMap::new();
            response.insert("ok".to_string(), JsValue::Boolean(true));
            response.insert("status".to_string(), JsValue::Number(200.0));
            response.insert("url".to_string(), JsValue::String(url));

            response.insert("json".to_string(), JsValue::NativeFunction {
                name: "json".to_string(),
                arity: 0,
                func: |_| {
                    let mut obj = HashMap::new();
                    obj.insert("data".to_string(), JsValue::String("mock data".to_string()));
                    JsValue::Object(Rc::new(RefCell::new(obj)))
                },
            });

            response.insert("text".to_string(), JsValue::NativeFunction {
                name: "text".to_string(),
                arity: 0,
                func: |_| JsValue::String("mock response".to_string()),
            });

            // Return as a "promise-like" object
            let mut promise = HashMap::new();
            promise.insert("[[PromiseState]]".to_string(), JsValue::String("fulfilled".to_string()));
            promise.insert("[[PromiseResult]]".to_string(), JsValue::Object(Rc::new(RefCell::new(response))));
            promise.insert("then".to_string(), JsValue::NativeFunction {
                name: "then".to_string(),
                arity: 1,
                func: |args| args.first().cloned().unwrap_or(JsValue::Undefined),
            });

            JsValue::Object(Rc::new(RefCell::new(promise)))
        },
    }
}

/// Create alert function
pub fn create_alert() -> JsValue {
    JsValue::NativeFunction {
        name: "alert".to_string(),
        arity: 1,
        func: |args| {
            let msg = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[ALERT] {}", msg);
            JsValue::Undefined
        },
    }
}

/// Create confirm function
pub fn create_confirm() -> JsValue {
    JsValue::NativeFunction {
        name: "confirm".to_string(),
        arity: 1,
        func: |args| {
            let msg = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[CONFIRM] {}", msg);
            JsValue::Boolean(true)
        },
    }
}

/// Create prompt function
pub fn create_prompt() -> JsValue {
    JsValue::NativeFunction {
        name: "prompt".to_string(),
        arity: 2,
        func: |args| {
            let msg = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            let default = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();
            println!("[PROMPT] {} (default: {})", msg, default);
            JsValue::String(default)
        },
    }
}

/// Create parseInt function
pub fn create_parse_int() -> JsValue {
    JsValue::NativeFunction {
        name: "parseInt".to_string(),
        arity: 2,
        func: |args| {
            let s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            let radix = args.get(1).map(|v| v.to_number() as i32).unwrap_or(10);
            match i64::from_str_radix(s.trim(), radix as u32) {
                Ok(n) => JsValue::Number(n as f64),
                Err(_) => JsValue::Number(f64::NAN),
            }
        },
    }
}

/// Create parseFloat function
pub fn create_parse_float() -> JsValue {
    JsValue::NativeFunction {
        name: "parseFloat".to_string(),
        arity: 1,
        func: |args| {
            let s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            match s.trim().parse::<f64>() {
                Ok(n) => JsValue::Number(n),
                Err(_) => JsValue::Number(f64::NAN),
            }
        },
    }
}

/// Create isNaN function
pub fn create_is_nan() -> JsValue {
    JsValue::NativeFunction {
        name: "isNaN".to_string(),
        arity: 1,
        func: |args| {
            let n = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
            JsValue::Boolean(n.is_nan())
        },
    }
}

/// Create isFinite function
pub fn create_is_finite() -> JsValue {
    JsValue::NativeFunction {
        name: "isFinite".to_string(),
        arity: 1,
        func: |args| {
            let n = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
            JsValue::Boolean(n.is_finite())
        },
    }
}

/// Create encodeURIComponent function
pub fn create_encode_uri_component() -> JsValue {
    JsValue::NativeFunction {
        name: "encodeURIComponent".to_string(),
        arity: 1,
        func: |args| {
            let s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            // Simple percent encoding
            let encoded: String = s.chars().map(|c| {
                if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
                    c.to_string()
                } else {
                    format!("%{:02X}", c as u32)
                }
            }).collect();
            JsValue::String(encoded)
        },
    }
}

/// Create decodeURIComponent function
pub fn create_decode_uri_component() -> JsValue {
    JsValue::NativeFunction {
        name: "decodeURIComponent".to_string(),
        arity: 1,
        func: |args| {
            let s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            // Simple percent decoding
            let mut result = String::new();
            let mut chars = s.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '%' {
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(code) = u8::from_str_radix(&hex, 16) {
                        result.push(code as char);
                    }
                } else {
                    result.push(c);
                }
            }
            JsValue::String(result)
        },
    }
}

/// Create setTimeout function (simulated)
pub fn create_set_timeout() -> JsValue {
    JsValue::NativeFunction {
        name: "setTimeout".to_string(),
        arity: 2,
        func: |args| {
            let _callback = args.first();
            let delay = args.get(1).map(|v| v.to_number()).unwrap_or(0.0);
            println!("[setTimeout] Registered callback with delay {}ms", delay);
            JsValue::Number(1.0) // Return timer ID
        },
    }
}

/// Create setInterval function (simulated)
pub fn create_set_interval() -> JsValue {
    JsValue::NativeFunction {
        name: "setInterval".to_string(),
        arity: 2,
        func: |args| {
            let _callback = args.first();
            let delay = args.get(1).map(|v| v.to_number()).unwrap_or(0.0);
            println!("[setInterval] Registered callback with interval {}ms", delay);
            JsValue::Number(1.0) // Return timer ID
        },
    }
}

/// Create clearTimeout function
pub fn create_clear_timeout() -> JsValue {
    JsValue::NativeFunction {
        name: "clearTimeout".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    }
}

/// Create clearInterval function
pub fn create_clear_interval() -> JsValue {
    JsValue::NativeFunction {
        name: "clearInterval".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    }
}

/// Create requestAnimationFrame function (simulated)
pub fn create_request_animation_frame() -> JsValue {
    JsValue::NativeFunction {
        name: "requestAnimationFrame".to_string(),
        arity: 1,
        func: |_| {
            JsValue::Number(1.0) // Return frame ID
        },
    }
}

/// Create cancelAnimationFrame function
pub fn create_cancel_animation_frame() -> JsValue {
    JsValue::NativeFunction {
        name: "cancelAnimationFrame".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    }
}
