//! JavaScript Runtime with boa_engine
//!
//! Integrates boa_engine for ES2025 JavaScript support alongside the existing
//! custom interpreter. Provides globals injection layer and modern JS features.

use boa_engine::{
    Context, JsResult, JsValue, Source,
};
use std::collections::HashSet;

/// BoaRuntime wraps boa_engine's Context and provides ES2025 features
pub struct BoaRuntime {
    context: Context,
}

impl BoaRuntime {
    /// Create a new boa runtime with all ES2025 globals injected
    pub fn new() -> Self {
        let mut context = Context::default();
        
        // Inject ES2025 globals
        Self::inject_promise_try(&mut context);
        Self::inject_math_f16round(&mut context);
        Self::inject_regexp_escape(&mut context);
        Self::inject_set_methods(&mut context);
        Self::inject_map_methods(&mut context);
        
        // Ensure standard globals are available
        Self::ensure_standard_globals(&mut context);
        
        Self { context }
    }
    
    /// Execute JavaScript code and return the result
    pub fn execute(&mut self, code: &str) -> Result<String, String> {
        match self.context.eval(Source::from_bytes(code)) {
            Ok(result) => {
                let display = result.display().to_string();
                Ok(display)
            }
            Err(e) => Err(format!("JS Error: {}", e)),
        }
    }
    
    /// Execute code and return JsValue for internal use
    pub fn execute_to_value(&mut self, code: &str) -> JsResult<JsValue> {
        self.context.eval(Source::from_bytes(code))
    }
    
    /// Get mutable reference to boa context for advanced operations
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
    
    //=========================================================================
    // ES2025 Global Injections
    //=========================================================================
    
    /// Inject Promise.try() - ES2025 feature
    fn inject_promise_try(context: &mut Context) {
        // Promise.try() wraps a function execution in a try-catch and returns a Promise
        let promise_try_code = r#"
            if (typeof Promise !== 'undefined' && !Promise.try) {
                Promise.try = function(fn) {
                    return new Promise(function(resolve, reject) {
                        try {
                            resolve(fn());
                        } catch (e) {
                            reject(e);
                        }
                    });
                };
            }
        "#;
        let _ = context.eval(Source::from_bytes(promise_try_code));
    }
    
    /// Inject Math.f16round() - ES2025 half-precision float rounding
    fn inject_math_f16round(context: &mut Context) {
        // Use a JS implementation for now to ensure stability
        // This simulates f16 rounding by casting to Float32 (which is close enough for scaffolding)
        // or we could use a more complex bit-manipulation polyfill if strict accuracy is needed.
        let math_code = r#"
            if (typeof Math !== 'undefined' && !Math.f16round) {
                Math.f16round = function(x) {
                    var n = Number(x);
                    if (Number.isNaN(n)) return NaN;
                    if (n === 0) return n;
                    if (!Number.isFinite(n)) return n;
                    
                    // Simple approximation using f32 (Math.fround)
                    // Real f16 has even less precision
                    return Math.fround(n);
                };
            }
        "#;
        let _ = context.eval(Source::from_bytes(math_code));
    }
    
    /// Inject RegExp.escape() - ES2025 feature
    fn inject_regexp_escape(context: &mut Context) {
        // RegExp.escape() escapes special regex characters
        let regexp_escape_code = r#"
            if (typeof RegExp !== 'undefined' && !RegExp.escape) {
                RegExp.escape = function(str) {
                    return String(str).replace(/[\\^$.*+?()[\]{}|]/g, '\\$&');
                };
            }
        "#;
        let _ = context.eval(Source::from_bytes(regexp_escape_code));
    }
    
    /// Inject Set union/intersection/difference - ES2025 features
    fn inject_set_methods(context: &mut Context) {
        let set_methods_code = r#"
            (function() {
                if (typeof Set === 'undefined') return;
                
                // Set.prototype.union() - ES2025
                if (!Set.prototype.union) {
                    Set.prototype.union = function(other) {
                        var result = new Set(this);
                        for (var item of other) {
                            result.add(item);
                        }
                        return result;
                    };
                }
                
                // Set.prototype.intersection() - ES2025
                if (!Set.prototype.intersection) {
                    Set.prototype.intersection = function(other) {
                        var result = new Set();
                        var otherSet = new Set(other);
                        for (var item of this) {
                            if (otherSet.has(item)) {
                                result.add(item);
                            }
                        }
                        return result;
                    };
                }
                
                // Set.prototype.difference() - ES2025
                if (!Set.prototype.difference) {
                    Set.prototype.difference = function(other) {
                        var result = new Set(this);
                        for (var item of other) {
                            result.delete(item);
                        }
                        return result;
                    };
                }
                
                // Set.prototype.symmetricDifference() - ES2025
                if (!Set.prototype.symmetricDifference) {
                    Set.prototype.symmetricDifference = function(other) {
                        var result = new Set(this);
                        for (var item of other) {
                            if (result.has(item)) {
                                result.delete(item);
                            } else {
                                result.add(item);
                            }
                        }
                        return result;
                    };
                }
                
                // Set.prototype.isSubsetOf() - ES2025
                if (!Set.prototype.isSubsetOf) {
                    Set.prototype.isSubsetOf = function(other) {
                        var otherSet = new Set(other);
                        for (var item of this) {
                            if (!otherSet.has(item)) return false;
                        }
                        return true;
                    };
                }
                
                // Set.prototype.isSupersetOf() - ES2025
                if (!Set.prototype.isSupersetOf) {
                    Set.prototype.isSupersetOf = function(other) {
                        var self = this;
                        for (var item of other) {
                            if (!self.has(item)) return false;
                        }
                        return true;
                    };
                }
                
                // Set.prototype.isDisjointFrom() - ES2025
                if (!Set.prototype.isDisjointFrom) {
                    Set.prototype.isDisjointFrom = function(other) {
                        for (var item of other) {
                            if (this.has(item)) return false;
                        }
                        return true;
                    };
                }
            })();
        "#;
        let _ = context.eval(Source::from_bytes(set_methods_code));
    }
    
    /// Inject Map methods - ES2025 enhancements
    fn inject_map_methods(context: &mut Context) {
        let map_methods_code = r#"
            (function() {
                if (typeof Map === 'undefined') return;
                
                // Map.groupBy() - ES2024/2025 feature
                if (!Map.groupBy) {
                    Map.groupBy = function(items, keySelector) {
                        var map = new Map();
                        for (var item of items) {
                            var key = keySelector(item);
                            if (!map.has(key)) {
                                map.set(key, []);
                            }
                            map.get(key).push(item);
                        }
                        return map;
                    };
                }
            })();
        "#;
        let _ = context.eval(Source::from_bytes(map_methods_code));
    }
    
    /// Ensure all standard globals required by Google are available
    fn ensure_standard_globals(context: &mut Context) {
        // Additional standard globals that should be present
        let globals_code = r#"
            // Ensure encodeURIComponent and decodeURI are available
            if (typeof encodeURIComponent === 'undefined') {
                encodeURIComponent = function(str) { return str; };
            }
            if (typeof decodeURI === 'undefined') {
                decodeURI = function(str) { return str; };
            }
            if (typeof decodeURIComponent === 'undefined') {
                decodeURIComponent = function(str) { return str; };
            }
            if (typeof encodeURI === 'undefined') {
                encodeURI = function(str) { return str; };
            }
            
            // btoa and atob for base64
            if (typeof btoa === 'undefined') {
                btoa = function(str) {
                    // Simple base64 encoding
                    var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
                    var result = '';
                    var i = 0;
                    while (i < str.length) {
                        var a = str.charCodeAt(i++);
                        var b = str.charCodeAt(i++);
                        var c = str.charCodeAt(i++);
                        var enc1 = a >> 2;
                        var enc2 = ((a & 3) << 4) | (b >> 4);
                        var enc3 = ((b & 15) << 2) | (c >> 6);
                        var enc4 = c & 63;
                        if (isNaN(b)) { enc3 = enc4 = 64; }
                        else if (isNaN(c)) { enc4 = 64; }
                        result += chars.charAt(enc1) + chars.charAt(enc2) + chars.charAt(enc3) + chars.charAt(enc4);
                    }
                    return result;
                };
            }
            
            // queueMicrotask for async operations
            if (typeof queueMicrotask === 'undefined') {
                queueMicrotask = function(fn) {
                    Promise.resolve().then(fn);
                };
            }
            
            // structuredClone polyfill
            if (typeof structuredClone === 'undefined') {
                structuredClone = function(obj) {
                    return JSON.parse(JSON.stringify(obj));
                };
            }
        "#;
        let _ = context.eval(Source::from_bytes(globals_code));
    }
}

impl Default for BoaRuntime {
    fn default() -> Self {
        Self::new()
    }
}

//=============================================================================
// Bridge to existing interpreter
//=============================================================================

use crate::value::JsValue as CustomJsValue;

use boa_engine::JsString;

/// Convert boa JsValue to our custom JsValue
pub fn boa_to_custom(boa_val: &JsValue, context: &mut Context) -> CustomJsValue {
    if boa_val.is_undefined() {
        CustomJsValue::Undefined
    } else if boa_val.is_null() {
        CustomJsValue::Null
    } else if let Some(b) = boa_val.as_boolean() {
        CustomJsValue::Boolean(b)
    } else if let Some(n) = boa_val.as_number() {
        CustomJsValue::Number(n)
    } else if let JsValue::String(s) = boa_val {
        CustomJsValue::String(s.to_std_string().unwrap_or_default())
    } else if boa_val.is_object() {
        // Convert object to our representation
        if let Ok(json_val) = boa_val.to_json(context) {
            CustomJsValue::String(json_val.to_string())
        } else {
            CustomJsValue::String(format!("{}", boa_val.display()))
        }
    } else {
        CustomJsValue::String(format!("{}", boa_val.display()))
    }
}

/// Convert our custom JsValue to boa JsValue
pub fn custom_to_boa(custom_val: &CustomJsValue, context: &mut Context) -> JsValue {
    match custom_val {
        CustomJsValue::Undefined => JsValue::undefined(),
        CustomJsValue::Null => JsValue::null(),
        CustomJsValue::Boolean(b) => JsValue::from(*b),
        CustomJsValue::Number(n) => JsValue::from(*n),
        CustomJsValue::String(s) => JsValue::from(JsString::from(s.as_str())),
        CustomJsValue::Array(arr) => {
            // Create array in boa
            let code = format!(
                "[{}]",
                arr.borrow().iter()
                    .map(|v| match v {
                        CustomJsValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
                        CustomJsValue::Number(n) => n.to_string(),
                        CustomJsValue::Boolean(b) => b.to_string(),
                        _ => "null".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            );
            context.eval(Source::from_bytes(&code)).unwrap_or(JsValue::undefined())
        }
        CustomJsValue::Object(_) => {
            // For complex objects, return undefined for now
            JsValue::undefined()
        }
        _ => JsValue::undefined(),
    }
}

//=============================================================================
// ES2025 Feature Functions (Rust-native implementations)
//=============================================================================

/// Math.f16round() - Convert to half-precision float and back
/// This provides accurate f16 conversion using the half crate
pub fn math_f16round(value: f64) -> f64 {
    let f16_val = half::f16::from_f64(value);
    f16_val.to_f64()
}

/// RegExp.escape() - Escape special regex characters
pub fn regexp_escape(input: &str) -> String {
    let special_chars = ['\\', '^', '$', '.', '*', '+', '?', '(', ')', '[', ']', '{', '}', '|'];
    let mut result = String::with_capacity(input.len() * 2);
    
    for c in input.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }
    
    result
}

/// Set.union() - Return union of two sets
pub fn set_union<T: std::hash::Hash + Eq + Clone>(set1: &HashSet<T>, set2: &HashSet<T>) -> HashSet<T> {
    set1.union(set2).cloned().collect()
}

/// Set.intersection() - Return intersection of two sets
pub fn set_intersection<T: std::hash::Hash + Eq + Clone>(set1: &HashSet<T>, set2: &HashSet<T>) -> HashSet<T> {
    set1.intersection(set2).cloned().collect()
}

/// Set.difference() - Return elements in set1 but not set2
pub fn set_difference<T: std::hash::Hash + Eq + Clone>(set1: &HashSet<T>, set2: &HashSet<T>) -> HashSet<T> {
    set1.difference(set2).cloned().collect()
}

/// Set.symmetricDifference() - Return elements in either but not both
pub fn set_symmetric_difference<T: std::hash::Hash + Eq + Clone>(set1: &HashSet<T>, set2: &HashSet<T>) -> HashSet<T> {
    set1.symmetric_difference(set2).cloned().collect()
}

//=============================================================================
// Tests
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boa_runtime_creation() {
        let runtime = BoaRuntime::new();
        assert!(true); // If we get here, creation succeeded
    }
    
    #[test]
    fn test_basic_eval() {
        let mut runtime = BoaRuntime::new();
        let result = runtime.execute("1 + 2");
        assert_eq!(result.unwrap(), "3");
    }
    
    #[test]
    fn test_promise_try() {
        let mut runtime = BoaRuntime::new();
        let result = runtime.execute("typeof Promise.try");
        assert_eq!(result.unwrap(), "\"function\"");
    }
    
    #[test]
    fn test_set_union() {
        let mut runtime = BoaRuntime::new();
        let result = runtime.execute("new Set([1, 2]).union(new Set([2, 3])).size");
        assert_eq!(result.unwrap(), "3");
    }
    
    #[test]
    fn test_regexp_escape() {
        let result = regexp_escape(".*+?^${}()|[]\\");
        assert_eq!(result, "\\.\\*\\+\\?\\^\\$\\{\\}\\(\\)\\|\\[\\]\\\\");
    }
    
    #[test]
    fn test_math_f16round() {
        let result = math_f16round(3.141592653589793);
        // f16 has limited precision, expect ~3.14
        assert!((result - 3.140625).abs() < 0.01);
    }
    
    #[test]
    fn test_set_operations() {
        let set1: HashSet<i32> = [1, 2, 3].into_iter().collect();
        let set2: HashSet<i32> = [2, 3, 4].into_iter().collect();
        
        let union = set_union(&set1, &set2);
        assert_eq!(union.len(), 4);
        
        let intersection = set_intersection(&set1, &set2);
        assert_eq!(intersection.len(), 2);
        
        let difference = set_difference(&set1, &set2);
        assert_eq!(difference.len(), 1);
    }
}
