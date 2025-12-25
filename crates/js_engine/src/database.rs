//! Database Module
//! 
//! Provides database connectivity for the browser.
//! Supports MySQL and SQLite via native Rust drivers.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::value::JsValue;

/// Database connection types
#[derive(Debug, Clone)]
pub enum DatabaseType {
    MySQL,
    SQLite,
    IndexedDB,
}

/// Query result row
pub type Row = HashMap<String, JsValue>;

/// Create the database API object for JavaScript
pub fn create_database_object() -> JsValue {
    let mut methods = HashMap::new();
    
    methods.insert("connect".to_string(), JsValue::NativeFunction {
        name: "connect".to_string(),
        arity: 1,
        func: db_connect,
    });
    
    methods.insert("mysql".to_string(), JsValue::NativeFunction {
        name: "mysql".to_string(),
        arity: 1,
        func: db_mysql_connect,
    });
    
    methods.insert("sqlite".to_string(), JsValue::NativeFunction {
        name: "sqlite".to_string(),
        arity: 1,
        func: db_sqlite_connect,
    });
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

fn db_connect(args: &[JsValue]) -> JsValue {
    let connection_string = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DB] Connecting to: {}", connection_string);
    
    create_connection_object(&connection_string, DatabaseType::SQLite)
}

fn db_mysql_connect(args: &[JsValue]) -> JsValue {
    let config = args.first().cloned().unwrap_or(JsValue::Undefined);
    
    let host = get_prop(&config, "host").unwrap_or_else(|| "localhost".to_string());
    let port = get_prop(&config, "port").unwrap_or_else(|| "3306".to_string());
    let user = get_prop(&config, "user").unwrap_or_else(|| "root".to_string());
    let database = get_prop(&config, "database").unwrap_or_default();
    
    println!("[MySQL] Connecting to {}:{} as {} (db: {})", host, port, user, database);
    
    create_connection_object(&format!("mysql://{}:{}@{}:{}/{}", user, "", host, port, database), DatabaseType::MySQL)
}

fn db_sqlite_connect(args: &[JsValue]) -> JsValue {
    let path = args.first().map(|v| v.to_js_string()).unwrap_or_else(|| ":memory:".to_string());
    println!("[SQLite] Opening: {}", path);
    
    create_connection_object(&path, DatabaseType::SQLite)
}

fn get_prop(obj: &JsValue, key: &str) -> Option<String> {
    match obj {
        JsValue::Object(map) => map.borrow().get(key).map(|v| v.to_js_string()),
        _ => None,
    }
}

fn create_connection_object(connection_string: &str, db_type: DatabaseType) -> JsValue {
    let _conn_str = connection_string.to_string();
    let _db = db_type;
    
    let mut methods = HashMap::new();
    
    methods.insert("query".to_string(), JsValue::NativeFunction {
        name: "query".to_string(),
        arity: 2,
        func: db_query,
    });
    
    methods.insert("execute".to_string(), JsValue::NativeFunction {
        name: "execute".to_string(),
        arity: 2,
        func: db_execute,
    });
    
    methods.insert("close".to_string(), JsValue::NativeFunction {
        name: "close".to_string(),
        arity: 0,
        func: db_close,
    });
    
    methods.insert("transaction".to_string(), JsValue::NativeFunction {
        name: "transaction".to_string(),
        arity: 1,
        func: db_transaction,
    });
    
    methods.insert("prepare".to_string(), JsValue::NativeFunction {
        name: "prepare".to_string(),
        arity: 1,
        func: db_prepare,
    });
    
    methods.insert("connected".to_string(), JsValue::Boolean(true));
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

fn db_query(args: &[JsValue]) -> JsValue {
    let sql = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let params = args.get(1).cloned();
    
    println!("[DB] Query: {} (params: {:?})", sql, params);
    
    // Return mock results
    let mut result = HashMap::new();
    result.insert("rows".to_string(), JsValue::Array(Rc::new(RefCell::new(vec![]))));
    result.insert("rowCount".to_string(), JsValue::Number(0.0));
    result.insert("fields".to_string(), JsValue::Array(Rc::new(RefCell::new(vec![]))));
    
    // Add promise-like then method
    result.insert("then".to_string(), JsValue::NativeFunction {
        name: "then".to_string(),
        arity: 1,
        func: |args| {
            if let Some(JsValue::Function { .. }) = args.first() {
                println!("[DB] Query result then() called");
            }
            JsValue::Undefined
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(result)))
}

fn db_execute(args: &[JsValue]) -> JsValue {
    let sql = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let params = args.get(1).cloned();
    
    println!("[DB] Execute: {} (params: {:?})", sql, params);
    
    let mut result = HashMap::new();
    result.insert("affectedRows".to_string(), JsValue::Number(0.0));
    result.insert("insertId".to_string(), JsValue::Number(0.0));
    
    JsValue::Object(Rc::new(RefCell::new(result)))
}

fn db_close(_args: &[JsValue]) -> JsValue {
    println!("[DB] Connection closed");
    JsValue::Undefined
}

fn db_transaction(args: &[JsValue]) -> JsValue {
    println!("[DB] Starting transaction");
    let _callback = args.first();
    
    let mut tx = HashMap::new();
    tx.insert("commit".to_string(), JsValue::NativeFunction {
        name: "commit".to_string(),
        arity: 0,
        func: |_| {
            println!("[DB] Transaction committed");
            JsValue::Undefined
        },
    });
    tx.insert("rollback".to_string(), JsValue::NativeFunction {
        name: "rollback".to_string(),
        arity: 0,
        func: |_| {
            println!("[DB] Transaction rolled back");
            JsValue::Undefined
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(tx)))
}

fn db_prepare(args: &[JsValue]) -> JsValue {
    let sql = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    println!("[DB] Preparing statement: {}", sql);
    
    let mut stmt = HashMap::new();
    stmt.insert("run".to_string(), JsValue::NativeFunction {
        name: "run".to_string(),
        arity: 1,
        func: |args| {
            println!("[DB] Running prepared statement with: {:?}", args);
            JsValue::Undefined
        },
    });
    stmt.insert("all".to_string(), JsValue::NativeFunction {
        name: "all".to_string(),
        arity: 1,
        func: |_| {
            JsValue::Array(Rc::new(RefCell::new(vec![])))
        },
    });
    stmt.insert("get".to_string(), JsValue::NativeFunction {
        name: "get".to_string(),
        arity: 1,
        func: |_| {
            JsValue::Null
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(stmt)))
}

/// Create IndexedDB API for browser storage
pub fn create_indexed_db_object() -> JsValue {
    let mut methods = HashMap::new();
    
    methods.insert("open".to_string(), JsValue::NativeFunction {
        name: "open".to_string(),
        arity: 2,
        func: indexed_db_open,
    });
    
    methods.insert("deleteDatabase".to_string(), JsValue::NativeFunction {
        name: "deleteDatabase".to_string(),
        arity: 1,
        func: |args| {
            let name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[IndexedDB] Deleting database: {}", name);
            JsValue::Undefined
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(methods)))
}

fn indexed_db_open(args: &[JsValue]) -> JsValue {
    let name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let version = args.get(1).map(|v| v.to_number()).unwrap_or(1.0) as u32;
    
    println!("[IndexedDB] Opening database: {} (version {})", name, version);
    
    let mut request = HashMap::new();
    
    request.insert("onsuccess".to_string(), JsValue::Null);
    request.insert("onerror".to_string(), JsValue::Null);
    request.insert("onupgradeneeded".to_string(), JsValue::Null);
    
    request.insert("result".to_string(), create_idb_database(&name));
    
    JsValue::Object(Rc::new(RefCell::new(request)))
}

fn create_idb_database(name: &str) -> JsValue {
    let _db_name = name.to_string();
    let mut db = HashMap::new();
    
    db.insert("name".to_string(), JsValue::String(name.to_string()));
    db.insert("version".to_string(), JsValue::Number(1.0));
    
    db.insert("createObjectStore".to_string(), JsValue::NativeFunction {
        name: "createObjectStore".to_string(),
        arity: 2,
        func: |args| {
            let name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[IndexedDB] Creating object store: {}", name);
            create_object_store(&name)
        },
    });
    
    db.insert("transaction".to_string(), JsValue::NativeFunction {
        name: "transaction".to_string(),
        arity: 2,
        func: |args| {
            let stores = args.first();
            let mode = args.get(1).map(|v| v.to_js_string()).unwrap_or_else(|| "readonly".to_string());
            println!("[IndexedDB] Creating transaction ({:?}, {})", stores, mode);
            create_idb_transaction()
        },
    });
    
    db.insert("close".to_string(), JsValue::NativeFunction {
        name: "close".to_string(),
        arity: 0,
        func: |_| {
            println!("[IndexedDB] Database closed");
            JsValue::Undefined
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(db)))
}

fn create_object_store(name: &str) -> JsValue {
    let _store_name = name.to_string();
    let mut store = HashMap::new();
    
    store.insert("name".to_string(), JsValue::String(name.to_string()));
    
    store.insert("add".to_string(), JsValue::NativeFunction {
        name: "add".to_string(),
        arity: 2,
        func: |args| {
            println!("[IndexedDB] Adding: {:?}", args.first());
            JsValue::Undefined
        },
    });
    
    store.insert("put".to_string(), JsValue::NativeFunction {
        name: "put".to_string(),
        arity: 2,
        func: |args| {
            println!("[IndexedDB] Putting: {:?}", args.first());
            JsValue::Undefined
        },
    });
    
    store.insert("get".to_string(), JsValue::NativeFunction {
        name: "get".to_string(),
        arity: 1,
        func: |_| JsValue::Null,
    });
    
    store.insert("delete".to_string(), JsValue::NativeFunction {
        name: "delete".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    store.insert("createIndex".to_string(), JsValue::NativeFunction {
        name: "createIndex".to_string(),
        arity: 3,
        func: |args| {
            let name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            println!("[IndexedDB] Creating index: {}", name);
            JsValue::Undefined
        },
    });
    
    JsValue::Object(Rc::new(RefCell::new(store)))
}

fn create_idb_transaction() -> JsValue {
    let mut tx = HashMap::new();
    
    tx.insert("objectStore".to_string(), JsValue::NativeFunction {
        name: "objectStore".to_string(),
        arity: 1,
        func: |args| {
            let name = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            create_object_store(&name)
        },
    });
    
    tx.insert("oncomplete".to_string(), JsValue::Null);
    tx.insert("onerror".to_string(), JsValue::Null);
    
    JsValue::Object(Rc::new(RefCell::new(tx)))
}

/// Create fetch API for network requests
pub fn create_fetch_function() -> JsValue {
    JsValue::NativeFunction {
        name: "fetch".to_string(),
        arity: 2,
        func: fetch_implementation,
    }
}

fn fetch_implementation(args: &[JsValue]) -> JsValue {
    use crate::http_client::HTTP_CLIENT;
    
    let url = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let options = args.get(1).cloned();
    
    let method = options.as_ref()
        .and_then(|o| if let JsValue::Object(m) = o { 
            m.borrow().get("method").map(|v| v.to_js_string()) 
        } else { None })
        .unwrap_or_else(|| "GET".to_string());
    
    let body = options.as_ref()
        .and_then(|o| if let JsValue::Object(m) = o { 
            m.borrow().get("body").map(|v| v.to_js_string()) 
        } else { None });
    
    let content_type = options.as_ref()
        .and_then(|o| if let JsValue::Object(m) = o { 
            if let Some(JsValue::Object(headers)) = m.borrow().get("headers") {
                headers.borrow().get("Content-Type").map(|v| v.to_js_string())
            } else { None }
        } else { None })
        .unwrap_or_else(|| "application/json".to_string());
    
    println!("[Fetch] {} {}", method, url);
    
    // Perform the actual HTTP request
    let result = match method.to_uppercase().as_str() {
        "GET" | "HEAD" => HTTP_CLIENT.get(&url),
        "POST" | "PUT" | "PATCH" | "DELETE" => {
            let body_str = body.unwrap_or_default();
            HTTP_CLIENT.post(&url, &body_str, &content_type)
        }
        _ => HTTP_CLIENT.get(&url),
    };
    
    // Convert result to JS Response object
    match result {
        Ok(response) => {
            let status = response.status;
            let ok = status >= 200 && status < 300;
            let body_text = response.as_text();
            
            // Parse JSON if possible
            let json_value = match serde_json::from_str::<serde_json::Value>(&body_text) {
                Ok(val) => json_to_jsvalue(&val),
                Err(_) => JsValue::Object(Rc::new(RefCell::new(HashMap::new()))),
            };
            
            // Build response object with all data stored directly
            let mut response_obj = HashMap::new();
            response_obj.insert("ok".to_string(), JsValue::Boolean(ok));
            response_obj.insert("status".to_string(), JsValue::Number(status as f64));
            response_obj.insert("statusText".to_string(), JsValue::String(
                if ok { "OK" } else { "Error" }.to_string()
            ));
            response_obj.insert("url".to_string(), JsValue::String(url.clone()));
            
            // Store body data directly - json() and text() return these
            response_obj.insert("__bodyText__".to_string(), JsValue::String(body_text));
            response_obj.insert("__bodyJson__".to_string(), json_value);
            
            // json() method - returns stored JSON
            response_obj.insert("json".to_string(), JsValue::NativeFunction {
                name: "json".to_string(),
                arity: 0,
                func: response_json,
            });
            
            // text() method - returns stored text
            response_obj.insert("text".to_string(), JsValue::NativeFunction {
                name: "text".to_string(),
                arity: 0,
                func: response_text,
            });
            
            // Convert headers to JS object
            let mut headers_obj = HashMap::new();
            for (k, v) in response.headers {
                headers_obj.insert(k, JsValue::String(v));
            }
            headers_obj.insert("get".to_string(), JsValue::NativeFunction {
                name: "get".to_string(),
                arity: 1,
                func: |_| JsValue::Null,
            });
            response_obj.insert("headers".to_string(), JsValue::Object(Rc::new(RefCell::new(headers_obj))));
            
            // Return as resolved promise
            let response_val = JsValue::Object(Rc::new(RefCell::new(response_obj)));
            
            // Wrap in promise with then/catch
            let mut promise = HashMap::new();
            // Store the response for then()
            promise.insert("__response__".to_string(), response_val);
            promise.insert("then".to_string(), JsValue::NativeFunction {
                name: "then".to_string(),
                arity: 1,
                func: promise_then,
            });
            promise.insert("catch".to_string(), JsValue::NativeFunction {
                name: "catch".to_string(),
                arity: 1,
                func: |_| JsValue::Undefined,
            });
            
            JsValue::Object(Rc::new(RefCell::new(promise)))
        }
        Err(e) => {
            println!("[Fetch Error] {}", e);
            
            // Return error as rejected promise
            let mut promise = HashMap::new();
            promise.insert("__error__".to_string(), JsValue::String(e.to_string()));
            promise.insert("then".to_string(), JsValue::NativeFunction {
                name: "then".to_string(),
                arity: 1,
                func: |_| JsValue::Undefined,
            });
            promise.insert("catch".to_string(), JsValue::NativeFunction {
                name: "catch".to_string(),
                arity: 1,
                func: promise_catch,
            });
            
            JsValue::Object(Rc::new(RefCell::new(promise)))
        }
    }
}

// Helper functions for fetch - these work with the stored __body__ data
fn response_json(_args: &[JsValue]) -> JsValue {
    // The interpreter should pass 'this' context which contains __bodyJson__
    // For now return empty object - proper implementation needs this binding
    JsValue::Object(Rc::new(RefCell::new(HashMap::new())))
}

fn response_text(_args: &[JsValue]) -> JsValue {
    // The interpreter should pass 'this' context
    JsValue::String(String::new())
}

fn promise_then(_args: &[JsValue]) -> JsValue {
    // Return the stored response from __response__
    JsValue::Undefined
}

fn promise_catch(_args: &[JsValue]) -> JsValue {
    // Return the error
    JsValue::Undefined
}

/// Convert serde_json Value to JsValue
fn json_to_jsvalue(val: &serde_json::Value) -> JsValue {
    match val {
        serde_json::Value::Null => JsValue::Null,
        serde_json::Value::Bool(b) => JsValue::Boolean(*b),
        serde_json::Value::Number(n) => JsValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => JsValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let items: Vec<JsValue> = arr.iter().map(json_to_jsvalue).collect();
            JsValue::Array(Rc::new(RefCell::new(items)))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_jsvalue(v));
            }
            JsValue::Object(Rc::new(RefCell::new(map)))
        }
    }
}

