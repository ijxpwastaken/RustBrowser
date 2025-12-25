//! JavaScript Interpreter
//! 
//! Executes the AST and produces results.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::{JsError, JsResult};
use crate::ast::*;
use crate::value::JsValue;
use crate::builtins;
use crate::security::{SecurityContext, ApiAccessControl};

/// Control flow signals
enum ControlFlow {
    None,
    Return(JsValue),
    Break,
    Continue,
}

/// Execution context kind (for proper this binding)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionContextKind {
    Global,
    Function,
    Arrow,
    Method,
    Constructor,
    Eval,
}

/// Execution context for tracking call frames
#[derive(Clone)]
pub struct ExecutionContext {
    /// Kind of execution context
    pub kind: ExecutionContextKind,
    /// The 'this' value for this context
    pub this_value: JsValue,
    /// Is this an arrow function (inherits this)
    pub is_arrow: bool,
    /// Function name for stack traces
    pub function_name: Option<String>,
}

impl ExecutionContext {
    pub fn global() -> Self {
        ExecutionContext {
            kind: ExecutionContextKind::Global,
            this_value: JsValue::Undefined, // Would be window/global in browser
            is_arrow: false,
            function_name: None,
        }
    }
    
    pub fn function(this_value: JsValue, function_name: Option<String>) -> Self {
        ExecutionContext {
            kind: ExecutionContextKind::Function,
            this_value,
            is_arrow: false,
            function_name,
        }
    }
    
    pub fn arrow(outer_this: JsValue, function_name: Option<String>) -> Self {
        ExecutionContext {
            kind: ExecutionContextKind::Arrow,
            this_value: outer_this, // Arrow functions inherit this
            is_arrow: true,
            function_name,
        }
    }
    
    pub fn method(receiver: JsValue, method_name: Option<String>) -> Self {
        ExecutionContext {
            kind: ExecutionContextKind::Method,
            this_value: receiver,
            is_arrow: false,
            function_name: method_name,
        }
    }
    
    pub fn constructor(new_object: JsValue, constructor_name: Option<String>) -> Self {
        ExecutionContext {
            kind: ExecutionContextKind::Constructor,
            this_value: new_object,
            is_arrow: false,
            function_name: constructor_name,
        }
    }
}

/// JavaScript Interpreter
pub struct Interpreter {
    /// Global scope
    global: Rc<RefCell<HashMap<String, JsValue>>>,
    /// Scope stack
    scopes: Vec<Rc<RefCell<HashMap<String, JsValue>>>>,
    /// Execution context stack (for proper this binding)
    context_stack: Vec<ExecutionContext>,
    /// Security context
    pub security: SecurityContext,
    /// Console output buffer (for testing)
    pub console_output: Vec<String>,
    /// Current page origin (for CSP checks)
    pub page_origin: Option<String>,
    /// Strict mode flag
    pub strict_mode: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(HashMap::new()));
        
        // Add built-ins
        {
            let mut g = global.borrow_mut();
            g.insert("console".to_string(), builtins::create_console());
            g.insert("Math".to_string(), builtins::create_math());
            g.insert("alert".to_string(), builtins::create_alert());
            g.insert("parseInt".to_string(), builtins::create_parse_int());
            g.insert("parseFloat".to_string(), builtins::create_parse_float());
            g.insert("undefined".to_string(), JsValue::Undefined);
            g.insert("NaN".to_string(), JsValue::Number(f64::NAN));
            g.insert("Infinity".to_string(), JsValue::Number(f64::INFINITY));
        }
        
        // Create global execution context
        let global_context = ExecutionContext::global();
        
        Interpreter {
            global: global.clone(),
            scopes: vec![global],
            context_stack: vec![global_context],
            security: SecurityContext::default(),
            console_output: Vec::new(),
            page_origin: None,
            strict_mode: false,
        }
    }
    
    /// Create interpreter with security context for a specific origin
    pub fn with_origin(origin: &str) -> Self {
        let mut interp = Self::new();
        interp.page_origin = Some(origin.to_string());
        interp.security = crate::security::create_security_context(origin);
        interp
    }
    
    /// Get current 'this' value
    pub fn get_this(&self) -> JsValue {
        // Walk up context stack to find appropriate this
        for ctx in self.context_stack.iter().rev() {
            // Arrow functions inherit this from enclosing context
            if !ctx.is_arrow {
                return ctx.this_value.clone();
            }
        }
        // Default to undefined in strict mode, global otherwise
        if self.strict_mode {
            JsValue::Undefined
        } else {
            // Would return window/global in browser
            JsValue::Undefined
        }
    }
    
    /// Push a new execution context
    pub fn push_context(&mut self, ctx: ExecutionContext) {
        self.context_stack.push(ctx);
    }
    
    /// Pop execution context
    pub fn pop_context(&mut self) {
        if self.context_stack.len() > 1 {
            self.context_stack.pop();
        }
    }
    
    /// Check if current origin can access database APIs
    pub fn can_access_database(&self) -> bool {
        self.security.can_access_database()
    }
    
    /// Check if current origin can access storage
    pub fn can_access_storage(&self) -> bool {
        self.security.can_access_storage()
    }
    
    /// Check if eval is allowed
    pub fn can_eval(&self) -> bool {
        self.security.can_execute_eval()
    }

    /// Execute a program
    pub fn execute(&mut self, program: &Program) -> JsResult<JsValue> {
        let mut result = JsValue::Undefined;
        
        for statement in &program.statements {
            match self.execute_statement(statement)? {
                ControlFlow::Return(v) => return Ok(v),
                ControlFlow::None => {}
                _ => {}
            }
            
            // Keep last expression result
            if let Statement::Expression(expr) = statement {
                result = self.evaluate(expr)?;
            }
        }
        
        Ok(result)
    }

    fn execute_statement(&mut self, stmt: &Statement) -> JsResult<ControlFlow> {
        match stmt {
            Statement::VariableDeclaration { kind: _, name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => JsValue::Undefined,
                };
                self.define(name.clone(), value);
                Ok(ControlFlow::None)
            }
            
            Statement::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(ControlFlow::None)
            }
            
            Statement::Block(statements) => {
                self.push_scope();
                for stmt in statements {
                    let flow = self.execute_statement(stmt)?;
                    if !matches!(flow, ControlFlow::None) {
                        self.pop_scope();
                        return Ok(flow);
                    }
                }
                self.pop_scope();
                Ok(ControlFlow::None)
            }
            
            Statement::If { condition, then_branch, else_branch } => {
                let cond_value = self.evaluate(condition)?;
                if cond_value.is_truthy() {
                    self.execute_statement(then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.execute_statement(else_stmt)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            
            Statement::While { condition, body } => {
                loop {
                    let cond_value = self.evaluate(condition)?;
                    if !cond_value.is_truthy() {
                        break;
                    }
                    
                    match self.execute_statement(body)? {
                        ControlFlow::Break => break,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Continue | ControlFlow::None => continue,
                    }
                }
                Ok(ControlFlow::None)
            }
            
            Statement::For { init, condition, update, body } => {
                self.push_scope();
                
                if let Some(init_stmt) = init {
                    self.execute_statement(init_stmt)?;
                }
                
                loop {
                    if let Some(cond) = condition {
                        let cond_value = self.evaluate(cond)?;
                        if !cond_value.is_truthy() {
                            break;
                        }
                    }
                    
                    match self.execute_statement(body)? {
                        ControlFlow::Break => break,
                        ControlFlow::Return(v) => {
                            self.pop_scope();
                            return Ok(ControlFlow::Return(v));
                        }
                        ControlFlow::Continue | ControlFlow::None => {}
                    }
                    
                    if let Some(upd) = update {
                        self.evaluate(upd)?;
                    }
                }
                
                self.pop_scope();
                Ok(ControlFlow::None)
            }
            
            Statement::FunctionDeclaration { name, params, body } => {
                let func = JsValue::Function {
                    name: Some(name.clone()),
                    params: params.clone(),
                    body: body.clone(),
                };
                self.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            
            Statement::Return(expr) => {
                let value = match expr {
                    Some(e) => self.evaluate(e)?,
                    None => JsValue::Undefined,
                };
                Ok(ControlFlow::Return(value))
            }
            
            Statement::Break => Ok(ControlFlow::Break),
            Statement::Continue => Ok(ControlFlow::Continue),
            Statement::Empty => Ok(ControlFlow::None),
            
            Statement::DoWhile { body, condition } => {
                loop {
                    match self.execute_statement(body)? {
                        ControlFlow::Break => break,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Continue | ControlFlow::None => {}
                    }
                    let cond_value = self.evaluate(condition)?;
                    if !cond_value.is_truthy() {
                        break;
                    }
                }
                Ok(ControlFlow::None)
            }
            
            Statement::Try { try_block, catch_block, finally_block } => {
                let result = self.execute_statement(try_block);
                
                let result = match result {
                    Err(e) => {
                        if let Some((param, catch_stmt)) = catch_block {
                            self.push_scope();
                            if let Some(name) = param {
                                self.define(name.clone(), JsValue::String(format!("{:?}", e)));
                            }
                            let res = self.execute_statement(catch_stmt);
                            self.pop_scope();
                            res
                        } else {
                            Err(e)
                        }
                    }
                    ok => ok,
                };
                
                if let Some(finally_stmt) = finally_block {
                    self.execute_statement(finally_stmt)?;
                }
                
                result
            }
            
            Statement::Throw(expr) => {
                let value = self.evaluate(expr)?;
                Err(JsError::RuntimeError(value.to_js_string()))
            }
            
            Statement::Switch { discriminant, cases, default_case } => {
                let disc_val = self.evaluate(discriminant)?;
                let mut matched = false;
                let mut fell_through = false;
                
                for (test, consequent) in cases {
                    if let Some(test_expr) = test {
                        let test_val = self.evaluate(test_expr)?;
                        if !matched && !fell_through {
                            if disc_val == test_val {
                                matched = true;
                            }
                        }
                    }
                    
                    if matched || fell_through {
                        for stmt in consequent {
                            match self.execute_statement(stmt)? {
                                ControlFlow::Break => return Ok(ControlFlow::None),
                                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                _ => {}
                            }
                        }
                        fell_through = true;
                    }
                }
                
                if !matched && !fell_through {
                    if let Some(default_stmts) = default_case {
                        for stmt in default_stmts {
                            match self.execute_statement(stmt)? {
                                ControlFlow::Break => return Ok(ControlFlow::None),
                                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                _ => {}
                            }
                        }
                    }
                }
                
                Ok(ControlFlow::None)
            }
            
            Statement::Class { name, extends: _, body: _ } => {
                // Create a simple constructor function for the class
                let func = JsValue::Function {
                    name: Some(name.clone()),
                    params: vec![],
                    body: vec![],
                };
                self.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            
            // For-in loop
            Statement::ForIn { var_kind: _, var_name, iterable, body } => {
                let iter_val = self.evaluate(iterable)?;
                
                // Get keys if object
                if let JsValue::Object(obj) = iter_val {
                    self.push_scope();
                    for key in obj.borrow().keys() {
                        self.define(var_name.clone(), JsValue::String(key.clone()));
                        match self.execute_statement(body)? {
                            ControlFlow::Break => break,
                            ControlFlow::Return(v) => {
                                self.pop_scope();
                                return Ok(ControlFlow::Return(v));
                            }
                            _ => {}
                        }
                    }
                    self.pop_scope();
                } else if let JsValue::Array(arr) = iter_val {
                    self.push_scope();
                    for (i, _) in arr.borrow().iter().enumerate() {
                        self.define(var_name.clone(), JsValue::Number(i as f64));
                        match self.execute_statement(body)? {
                            ControlFlow::Break => break,
                            ControlFlow::Return(v) => {
                                self.pop_scope();
                                return Ok(ControlFlow::Return(v));
                            }
                            _ => {}
                        }
                    }
                    self.pop_scope();
                }
                Ok(ControlFlow::None)
            }
            
            // For-of loop
            Statement::ForOf { var_kind: _, var_name, iterable, body } => {
                let iter_val = self.evaluate(iterable)?;
                
                if let JsValue::Array(arr) = iter_val {
                    self.push_scope();
                    for item in arr.borrow().iter() {
                        self.define(var_name.clone(), item.clone());
                        match self.execute_statement(body)? {
                            ControlFlow::Break => break,
                            ControlFlow::Return(v) => {
                                self.pop_scope();
                                return Ok(ControlFlow::Return(v));
                            }
                            _ => {}
                        }
                    }
                    self.pop_scope();
                } else if let JsValue::String(s) = iter_val {
                    self.push_scope();
                    for c in s.chars() {
                        self.define(var_name.clone(), JsValue::String(c.to_string()));
                        match self.execute_statement(body)? {
                            ControlFlow::Break => break,
                            ControlFlow::Return(v) => {
                                self.pop_scope();
                                return Ok(ControlFlow::Return(v));
                            }
                            _ => {}
                        }
                    }
                    self.pop_scope();
                }
                Ok(ControlFlow::None)
            }
            
            // Async function declaration (treat like regular function for now)
            Statement::AsyncFunctionDeclaration { name, params, body } => {
                let func = JsValue::Function {
                    name: Some(name.clone()),
                    params: params.clone(),
                    body: body.clone(),
                };
                self.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            
            // Generator function declaration (treat like regular function for now)
            Statement::GeneratorFunctionDeclaration { name, params, body } => {
                let func = JsValue::Function {
                    name: Some(name.clone()),
                    params: params.clone(),
                    body: body.clone(),
                };
                self.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            
            // Labeled statement
            Statement::Labeled { label: _, body } => {
                self.execute_statement(body)
            }
            
            // With statement
            Statement::With { object: _, body } => {
                // Simplified: just execute the body
                self.execute_statement(body)
            }
            
            // Debugger statement
            Statement::Debugger => {
                println!("[JS] debugger statement hit");
                Ok(ControlFlow::None)
            }
        }
    }

    fn evaluate(&mut self, expr: &Expression) -> JsResult<JsValue> {
        match expr {
            Expression::Number(n) => Ok(JsValue::Number(*n)),
            Expression::String(s) => Ok(JsValue::String(s.clone())),
            Expression::Boolean(b) => Ok(JsValue::Boolean(*b)),
            Expression::Null => Ok(JsValue::Null),
            Expression::Undefined => Ok(JsValue::Undefined),
            Expression::This => Ok(self.get_this()),
            
            Expression::Identifier(name) => {
                self.get(name).ok_or_else(|| JsError::ReferenceError(name.clone()))
            }
            
            Expression::Binary { left, operator, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.binary_op(&left_val, operator, &right_val)
            }
            
            Expression::Unary { operator, operand } => {
                let val = self.evaluate(operand)?;
                self.unary_op(operator, &val)
            }
            
            Expression::Assignment { target, value } => {
                let val = self.evaluate(value)?;
                
                match target.as_ref() {
                    Expression::Identifier(name) => {
                        self.assign(name, val.clone())?;
                    }
                    Expression::Member { object, property, computed } => {
                        let obj = self.evaluate(object)?;
                        let prop = if *computed {
                            self.evaluate(property)?.to_js_string()
                        } else {
                            if let Expression::String(s) = property.as_ref() {
                                s.clone()
                            } else {
                                return Err(JsError::TypeError("Invalid property access".into()));
                            }
                        };
                        
                        if let JsValue::Object(map) = obj {
                            map.borrow_mut().insert(prop, val.clone());
                        }
                    }
                    _ => return Err(JsError::SyntaxError("Invalid assignment target".into())),
                }
                
                Ok(val)
            }
            
            Expression::Call { callee, arguments } => {
                let func = self.evaluate(callee)?;
                let args: Vec<JsValue> = arguments.iter()
                    .map(|a| self.evaluate(a))
                    .collect::<JsResult<_>>()?;
                
                self.call_function(func, args)
            }
            
            Expression::Member { object, property, computed } => {
                let obj = self.evaluate(object)?;
                let prop = if *computed {
                    self.evaluate(property)?.to_js_string()
                } else {
                    if let Expression::String(s) = property.as_ref() {
                        s.clone()
                    } else {
                        return Err(JsError::TypeError("Invalid property access".into()));
                    }
                };
                
                self.get_property(&obj, &prop)
            }
            
            Expression::Array(elements) => {
                let values: Vec<JsValue> = elements.iter()
                    .map(|e| self.evaluate(e))
                    .collect::<JsResult<_>>()?;
                Ok(JsValue::Array(Rc::new(RefCell::new(values))))
            }
            
            Expression::Object(properties) => {
                let mut map = HashMap::new();
                for (key, value) in properties {
                    map.insert(key.clone(), self.evaluate(value)?);
                }
                Ok(JsValue::Object(Rc::new(RefCell::new(map))))
            }
            
            Expression::Function { name, params, body } => {
                Ok(JsValue::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                })
            }
            
            Expression::Arrow { params, body } => {
                // Arrow now has a Statement body
                let body_stmts = match body.as_ref() {
                    Statement::Block(stmts) => stmts.clone(),
                    Statement::Return(expr) => vec![Statement::Return(expr.clone())],
                    other => vec![other.clone()],
                };
                Ok(JsValue::Function {
                    name: None,
                    params: params.clone(),
                    body: body_stmts,
                })
            }
            
            Expression::Ternary { condition, then_expr, else_expr } => {
                let cond = self.evaluate(condition)?;
                if cond.is_truthy() {
                    self.evaluate(then_expr)
                } else {
                    self.evaluate(else_expr)
                }
            }
            
            Expression::Super => Ok(JsValue::Undefined),
            
            Expression::New(callee) => {
                let _func = self.evaluate(callee)?;
                // For new, create an empty object (simplified)
                let obj = JsValue::Object(Rc::new(RefCell::new(HashMap::new())));
                Ok(obj)
            }
            
            Expression::Update { operator, prefix, operand } => {
                let current = self.evaluate(operand)?;
                let current_num = current.to_number();
                let new_val = if operator == "++" {
                    JsValue::Number(current_num + 1.0)
                } else {
                    JsValue::Number(current_num - 1.0)
                };
                
                // Update the variable
                if let Expression::Identifier(name) = operand.as_ref() {
                    self.assign(name, new_val.clone())?;
                }
                
                if *prefix {
                    Ok(new_val)
                } else {
                    Ok(JsValue::Number(current_num))
                }
            }
            
            Expression::Typeof(operand) => {
                let val = self.evaluate(operand)?;
                Ok(JsValue::String(val.type_of().to_string()))
            }
            
            Expression::Delete(_operand) => {
                // Simplified - just return true
                Ok(JsValue::Boolean(true))
            }
            
            Expression::Void(operand) => {
                self.evaluate(operand)?;
                Ok(JsValue::Undefined)
            }
            
            Expression::Spread(operand) => {
                // Just evaluate the operand for now
                self.evaluate(operand)
            }
            
            Expression::AsyncFunction { name, params, body } => {
                Ok(JsValue::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                })
            }
            
            Expression::Await(operand) => {
                // Just evaluate the promise-like value
                self.evaluate(operand)
            }
            
            Expression::Yield(operand) => {
                if let Some(expr) = operand {
                    self.evaluate(expr)
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            
            // BigInt literal
            Expression::BigInt(s) => {
                let n = s.parse::<i128>().unwrap_or(0);
                Ok(JsValue::BigInt(n))
            }
            
            // Template literal
            Expression::TemplateLiteral { quasis, expressions } => {
                let mut result = String::new();
                for (i, quasi) in quasis.iter().enumerate() {
                    result.push_str(quasi);
                    if let Some(expr) = expressions.get(i) {
                        let val = self.evaluate(expr)?;
                        result.push_str(&val.to_js_string());
                    }
                }
                Ok(JsValue::String(result))
            }
            
            // Tagged template (simplified)
            Expression::TaggedTemplate { tag, quasis, expressions } => {
                let tag_fn = self.evaluate(tag)?;
                let mut args = Vec::new();
                
                // First arg: template strings array
                let strings: Vec<JsValue> = quasis.iter()
                    .map(|s| JsValue::String(s.clone()))
                    .collect();
                args.push(JsValue::Array(Rc::new(RefCell::new(strings))));
                
                // Rest: expression values
                for expr in expressions {
                    args.push(self.evaluate(expr)?);
                }
                
                self.call_function(tag_fn, args)
            }
            
            // RegExp literal
            Expression::RegExp { pattern, flags } => {
                Ok(JsValue::RegExp { 
                    pattern: pattern.clone(), 
                    flags: flags.clone() 
                })
            }
            
            // Generator function expression
            Expression::GeneratorFunction { name, params, body } => {
                Ok(JsValue::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                })
            }
            
            // Async arrow function
            Expression::AsyncArrow { params, body } => {
                let body_stmts = match body.as_ref() {
                    Statement::Block(stmts) => stmts.clone(),
                    Statement::Return(expr) => vec![Statement::Return(expr.clone())],
                    other => vec![other.clone()],
                };
                Ok(JsValue::Function {
                    name: None,
                    params: params.clone(),
                    body: body_stmts,
                })
            }
            
            // Sequence expression (comma operator)
            Expression::Sequence(exprs) => {
                let mut result = JsValue::Undefined;
                for expr in exprs {
                    result = self.evaluate(expr)?;
                }
                Ok(result)
            }
            
            // Compound assignment
            Expression::CompoundAssignment { target, operator, value } => {
                let left = self.evaluate(target)?;
                let right = self.evaluate(value)?;
                
                let result = match operator {
                    CompoundAssignOp::Add => {
                        if matches!(left, JsValue::String(_)) || matches!(right, JsValue::String(_)) {
                            JsValue::String(format!("{}{}", left.to_js_string(), right.to_js_string()))
                        } else {
                            JsValue::Number(left.to_number() + right.to_number())
                        }
                    }
                    CompoundAssignOp::Subtract => JsValue::Number(left.to_number() - right.to_number()),
                    CompoundAssignOp::Multiply => JsValue::Number(left.to_number() * right.to_number()),
                    CompoundAssignOp::Divide => JsValue::Number(left.to_number() / right.to_number()),
                    CompoundAssignOp::Modulo => JsValue::Number(left.to_number() % right.to_number()),
                    CompoundAssignOp::Exponent => JsValue::Number(left.to_number().powf(right.to_number())),
                    CompoundAssignOp::BitAnd => JsValue::Number((left.to_number() as i64 & right.to_number() as i64) as f64),
                    CompoundAssignOp::BitOr => JsValue::Number((left.to_number() as i64 | right.to_number() as i64) as f64),
                    CompoundAssignOp::BitXor => JsValue::Number((left.to_number() as i64 ^ right.to_number() as i64) as f64),
                    CompoundAssignOp::LeftShift => JsValue::Number(((left.to_number() as i64) << (right.to_number() as i64)) as f64),
                    CompoundAssignOp::RightShift => JsValue::Number(((left.to_number() as i64) >> (right.to_number() as i64)) as f64),
                    CompoundAssignOp::UnsignedRightShift => JsValue::Number(((left.to_number() as u64) >> (right.to_number() as u64)) as f64),
                    CompoundAssignOp::And => if !left.is_truthy() { left } else { right },
                    CompoundAssignOp::Or => if left.is_truthy() { left } else { right },
                    CompoundAssignOp::NullishCoalesce => if left.is_nullish() { right } else { left },
                };
                
                // Assign result back to target
                if let Expression::Identifier(name) = target.as_ref() {
                    self.assign(name, result.clone())?;
                }
                Ok(result)
            }
            
            // Optional call
            Expression::OptionalCall { callee, arguments } => {
                let func = self.evaluate(callee)?;
                if func.is_nullish() {
                    return Ok(JsValue::Undefined);
                }
                let args: Vec<JsValue> = arguments.iter()
                    .map(|a| self.evaluate(a))
                    .collect::<JsResult<_>>()?;
                self.call_function(func, args)
            }
            
            // New with arguments
            Expression::NewWithArgs { callee, arguments } => {
                let _func = self.evaluate(callee)?;
                let _args: Vec<JsValue> = arguments.iter()
                    .map(|a| self.evaluate(a))
                    .collect::<JsResult<_>>()?;
                // Simplified: return empty object
                Ok(JsValue::Object(Rc::new(RefCell::new(HashMap::new()))))
            }
            
            // Optional member access
            Expression::OptionalMember { object, property, computed } => {
                let obj = self.evaluate(object)?;
                if obj.is_nullish() {
                    return Ok(JsValue::Undefined);
                }
                let prop = if *computed {
                    self.evaluate(property)?.to_js_string()
                } else {
                    if let Expression::String(s) = property.as_ref() {
                        s.clone()
                    } else {
                        return Err(JsError::TypeError("Invalid property access".into()));
                    }
                };
                self.get_property(&obj, &prop)
            }
            
            // Array with holes
            Expression::ArrayWithHoles(elements) => {
                let values: Vec<JsValue> = elements.iter()
                    .map(|e| match e {
                        Some(expr) => self.evaluate(expr).unwrap_or(JsValue::Undefined),
                        None => JsValue::Undefined,
                    })
                    .collect();
                Ok(JsValue::Array(Rc::new(RefCell::new(values))))
            }
            
            // Object with computed keys
            Expression::ObjectWithComputed(properties) => {
                let mut map = HashMap::new();
                for prop in properties {
                    let key = match &prop.key {
                        ObjectKey::Identifier(s) => s.clone(),
                        ObjectKey::String(s) => s.clone(),
                        ObjectKey::Number(n) => n.to_string(),
                        ObjectKey::Computed(expr) => self.evaluate(expr)?.to_js_string(),
                    };
                    map.insert(key, self.evaluate(&prop.value)?);
                }
                Ok(JsValue::Object(Rc::new(RefCell::new(map))))
            }
            
            // Class expression
            Expression::ClassExpression { name, extends: _, body: _ } => {
                Ok(JsValue::Function {
                    name: name.clone(),
                    params: vec![],
                    body: vec![],
                })
            }
            
            // Destructuring assignment (simplified)
            Expression::DestructuringAssignment { pattern: _, value } => {
                self.evaluate(value)
            }
            
            // Dynamic import
            Expression::Import(source) => {
                let _src = self.evaluate(source)?;
                // Return a promise-like object
                Ok(JsValue::Promise {
                    state: crate::value::PromiseState::Fulfilled,
                    value: Box::new(JsValue::Object(Rc::new(RefCell::new(HashMap::new())))),
                })
            }
            
            // Meta property
            Expression::MetaProperty { meta, property } => {
                if meta == "import" && property == "meta" {
                    // Return import.meta object
                    let mut meta_obj = HashMap::new();
                    meta_obj.insert("url".to_string(), JsValue::String("file://".to_string()));
                    Ok(JsValue::Object(Rc::new(RefCell::new(meta_obj))))
                } else if meta == "new" && property == "target" {
                    Ok(JsValue::Undefined)
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            
            // Yield delegate
            Expression::YieldDelegate(operand) => {
                self.evaluate(operand)
            }
        }
    }

    fn binary_op(&self, left: &JsValue, op: &BinaryOp, right: &JsValue) -> JsResult<JsValue> {
        match op {
            BinaryOp::Add => {
                // String concatenation
                if matches!(left, JsValue::String(_)) || matches!(right, JsValue::String(_)) {
                    Ok(JsValue::String(format!("{}{}", left.to_js_string(), right.to_js_string())))
                } else {
                    Ok(JsValue::Number(left.to_number() + right.to_number()))
                }
            }
            BinaryOp::Subtract => Ok(JsValue::Number(left.to_number() - right.to_number())),
            BinaryOp::Multiply => Ok(JsValue::Number(left.to_number() * right.to_number())),
            BinaryOp::Divide => Ok(JsValue::Number(left.to_number() / right.to_number())),
            BinaryOp::Modulo => Ok(JsValue::Number(left.to_number() % right.to_number())),
            BinaryOp::Exponent => Ok(JsValue::Number(left.to_number().powf(right.to_number()))),
            BinaryOp::Equal => Ok(JsValue::Boolean(self.loose_equals(left, right))),
            BinaryOp::StrictEqual => Ok(JsValue::Boolean(left == right)),
            BinaryOp::NotEqual => Ok(JsValue::Boolean(!self.loose_equals(left, right))),
            BinaryOp::StrictNotEqual => Ok(JsValue::Boolean(left != right)),
            BinaryOp::Less => Ok(JsValue::Boolean(left.to_number() < right.to_number())),
            BinaryOp::Greater => Ok(JsValue::Boolean(left.to_number() > right.to_number())),
            BinaryOp::LessEqual => Ok(JsValue::Boolean(left.to_number() <= right.to_number())),
            BinaryOp::GreaterEqual => Ok(JsValue::Boolean(left.to_number() >= right.to_number())),
            BinaryOp::And => {
                if !left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            BinaryOp::Or => {
                if left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            BinaryOp::BitAnd => Ok(JsValue::Number((left.to_number() as i64 & right.to_number() as i64) as f64)),
            BinaryOp::BitOr => Ok(JsValue::Number((left.to_number() as i64 | right.to_number() as i64) as f64)),
            BinaryOp::BitXor => Ok(JsValue::Number((left.to_number() as i64 ^ right.to_number() as i64) as f64)),
            BinaryOp::LeftShift => Ok(JsValue::Number(((left.to_number() as i64) << (right.to_number() as i64)) as f64)),
            BinaryOp::RightShift => Ok(JsValue::Number(((left.to_number() as i64) >> (right.to_number() as i64)) as f64)),
            BinaryOp::UnsignedRightShift => Ok(JsValue::Number(((left.to_number() as u64) >> (right.to_number() as u64)) as f64)),
            BinaryOp::NullishCoalesce => {
                if matches!(left, JsValue::Null | JsValue::Undefined) {
                    Ok(right.clone())
                } else {
                    Ok(left.clone())
                }
            }
            BinaryOp::Instanceof => Ok(JsValue::Boolean(false)), // Simplified
            BinaryOp::In => Ok(JsValue::Boolean(false)), // Simplified
        }
    }

    fn unary_op(&self, op: &UnaryOp, val: &JsValue) -> JsResult<JsValue> {
        match op {
            UnaryOp::Negate => Ok(JsValue::Number(-val.to_number())),
            UnaryOp::Not => Ok(JsValue::Boolean(!val.is_truthy())),
            UnaryOp::Typeof => Ok(JsValue::String(val.type_of().to_string())),
            UnaryOp::BitNot => Ok(JsValue::Number(!(val.to_number() as i64) as f64)),
            UnaryOp::Plus => Ok(JsValue::Number(val.to_number())),
        }
    }

    fn loose_equals(&self, left: &JsValue, right: &JsValue) -> bool {
        match (left, right) {
            (JsValue::Undefined, JsValue::Null) | (JsValue::Null, JsValue::Undefined) => true,
            (JsValue::Number(a), JsValue::Number(b)) => a == b,
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            _ => left == right,
        }
    }

    fn call_function(&mut self, func: JsValue, args: Vec<JsValue>) -> JsResult<JsValue> {
        match func {
            JsValue::NativeFunction { func, .. } => {
                Ok(func(&args))
            }
            JsValue::Function { params, body, .. } => {
                self.push_scope();
                
                // Bind parameters
                for (i, param) in params.iter().enumerate() {
                    let value = args.get(i).cloned().unwrap_or(JsValue::Undefined);
                    self.define(param.clone(), value);
                }
                
                // Execute body
                let mut result = JsValue::Undefined;
                for stmt in &body {
                    match self.execute_statement(stmt)? {
                        ControlFlow::Return(v) => {
                            result = v;
                            break;
                        }
                        _ => {}
                    }
                }
                
                self.pop_scope();
                Ok(result)
            }
            _ => Err(JsError::TypeError(format!("{} is not a function", func.type_of()))),
        }
    }

    fn get_property(&self, obj: &JsValue, prop: &str) -> JsResult<JsValue> {
        match obj {
            JsValue::Object(map) => {
                Ok(map.borrow().get(prop).cloned().unwrap_or(JsValue::Undefined))
            }
            JsValue::Array(arr) => {
                if prop == "length" {
                    Ok(JsValue::Number(arr.borrow().len() as f64))
                } else if let Ok(idx) = prop.parse::<usize>() {
                    Ok(arr.borrow().get(idx).cloned().unwrap_or(JsValue::Undefined))
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            JsValue::String(s) => {
                if prop == "length" {
                    Ok(JsValue::Number(s.len() as f64))
                } else if let Ok(idx) = prop.parse::<usize>() {
                    Ok(s.chars().nth(idx)
                        .map(|c| JsValue::String(c.to_string()))
                        .unwrap_or(JsValue::Undefined))
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            _ => Ok(JsValue::Undefined),
        }
    }

    // Scope management
    
    fn push_scope(&mut self) {
        self.scopes.push(Rc::new(RefCell::new(HashMap::new())));
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define(&mut self, name: String, value: JsValue) {
        if let Some(scope) = self.scopes.last() {
            scope.borrow_mut().insert(name, value);
        }
    }

    fn get(&self, name: &str) -> Option<JsValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.borrow().get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    fn assign(&mut self, name: &str, value: JsValue) -> JsResult<()> {
        for scope in self.scopes.iter().rev() {
            if scope.borrow().contains_key(name) {
                scope.borrow_mut().insert(name.to_string(), value);
                return Ok(());
            }
        }
        // Create in global scope if not found
        self.global.borrow_mut().insert(name.to_string(), value);
        Ok(())
    }
    
    /// Define a variable in the global scope (for DOM objects)
    pub fn define_global(&mut self, name: &str, value: JsValue) {
        self.global.borrow_mut().insert(name.to_string(), value);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
