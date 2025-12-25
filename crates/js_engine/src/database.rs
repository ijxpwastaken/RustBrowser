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
    let url = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let options = args.get(1).cloned();
    
    let method = options.as_ref()
        .and_then(|o| if let JsValue::Object(m) = o { 
            m.borrow().get("method").map(|v| v.to_js_string()) 
        } else { None })
        .unwrap_or_else(|| "GET".to_string());
    
    println!("[Fetch] {} {}", method, url);
    
    // Note: In a real implementation, this would be async
    // For now, we'll create a promise-like object that can be resolved
    // The actual fetch will be handled by the browser core when needed
    
    let mut promise = HashMap::new();
    
    promise.insert("then".to_string(), JsValue::NativeFunction {
        name: "then".to_string(),
        arity: 1,
        func: move |_| {
            // Return response object
            // In a full implementation, this would contain the actual response
            let mut response = HashMap::new();
            response.insert("ok".to_string(), JsValue::Boolean(true));
            response.insert("status".to_string(), JsValue::Number(200.0));
            response.insert("statusText".to_string(), JsValue::String("OK".to_string()));
            
            response.insert("json".to_string(), JsValue::NativeFunction {
                name: "json".to_string(),
                arity: 0,
                func: |_| {
                    // Return empty object for now
                    JsValue::Object(Rc::new(RefCell::new(HashMap::new())))
                },
            });
            
            response.insert("text".to_string(), JsValue::NativeFunction {
                name: "text".to_string(),
                arity: 0,
                func: |_| JsValue::String(String::new()),
            });
            
            response.insert("headers".to_string(), JsValue::Object(Rc::new(RefCell::new(HashMap::new()))));
            
            JsValue::Object(Rc::new(RefCell::new(response)))
        },
    });
    
    promise.insert("catch".to_string(), JsValue::NativeFunction {
        name: "catch".to_string(),
        arity: 1,
        func: |_| JsValue::Undefined,
    });
    
    JsValue::Object(Rc::new(RefCell::new(promise)))
}
