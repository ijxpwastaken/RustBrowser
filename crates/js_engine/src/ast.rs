//! JavaScript Abstract Syntax Tree
//! 
//! Defines the AST nodes for the JavaScript parser.

use std::fmt;

/// A JavaScript program (list of statements)
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Statement {
    /// Variable declaration: var/let/const name = value
    VariableDeclaration {
        kind: VarKind,
        name: String,
        initializer: Option<Box<Expression>>,
    },
    
    /// Expression statement
    Expression(Expression),
    
    /// Block statement: { statements }
    Block(Vec<Statement>),
    
    /// If statement
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    
    /// While loop
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    
    /// Do-While loop
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
    },
    
    /// For loop
    For {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Box<Statement>,
    },
    
    /// For-in loop
    ForIn {
        var_kind: VarKind,
        var_name: String,
        iterable: Expression,
        body: Box<Statement>,
    },
    
    /// For-of loop
    ForOf {
        var_kind: VarKind,
        var_name: String,
        iterable: Expression,
        body: Box<Statement>,
    },
    
    /// Function declaration
    FunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    
    /// Async function declaration
    AsyncFunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    
    /// Generator function declaration
    GeneratorFunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    
    /// Return statement
    Return(Option<Expression>),
    
    /// Break statement
    Break,
    
    /// Continue statement
    Continue,
    
    /// Empty statement (;)
    Empty,
    
    /// Try-catch-finally
    Try {
        try_block: Box<Statement>,
        catch_block: Option<(Option<String>, Box<Statement>)>,
        finally_block: Option<Box<Statement>>,
    },
    
    /// Throw statement
    Throw(Expression),
    
    /// Switch statement
    Switch {
        discriminant: Expression,
        cases: Vec<(Option<Expression>, Vec<Statement>)>,
        default_case: Option<Vec<Statement>>,
    },
    
    /// Class declaration
    Class {
        name: String,
        extends: Option<String>,
        body: Vec<ClassMember>,
    },
    
    /// Labeled statement
    Labeled {
        label: String,
        body: Box<Statement>,
    },
    
    /// With statement (deprecated but supported)
    With {
        object: Expression,
        body: Box<Statement>,
    },
    
    /// Debugger statement
    Debugger,
}

/// Class member
#[derive(Debug, Clone)]
pub struct ClassMember {
    pub name: String,
    pub is_static: bool,
    pub kind: ClassMemberKind,
}

/// Kind of class member
#[derive(Debug, Clone)]
pub enum ClassMemberKind {
    Method { params: Vec<String>, body: Vec<Statement> },
    Property { value: Option<Expression> },
    Constructor { params: Vec<String>, body: Vec<Statement> },
    Getter { body: Vec<Statement> },
    Setter { param: String, body: Vec<Statement> },
}

/// Variable declaration kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expression {
    /// Number literal
    Number(f64),
    
    /// BigInt literal
    BigInt(String),
    
    /// String literal
    String(String),
    
    /// Template literal with interpolations
    TemplateLiteral {
        quasis: Vec<String>,
        expressions: Vec<Expression>,
    },
    
    /// Tagged template literal
    TaggedTemplate {
        tag: Box<Expression>,
        quasis: Vec<String>,
        expressions: Vec<Expression>,
    },
    
    /// Boolean literal
    Boolean(bool),
    
    /// Null literal
    Null,
    
    /// Undefined literal
    Undefined,
    
    /// Regular expression literal
    RegExp { pattern: String, flags: String },
    
    /// Identifier (variable reference)
    Identifier(String),
    
    /// This keyword
    This,
    
    /// Super keyword
    Super,
    
    /// Binary operation: left op right
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    
    /// Unary operation: op expr
    Unary {
        operator: UnaryOp,
        operand: Box<Expression>,
    },
    
    /// Update expression: ++x or x++
    Update {
        operator: String,
        prefix: bool,
        operand: Box<Expression>,
    },
    
    /// Assignment: target = value
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    
    /// Compound assignment: target op= value
    CompoundAssignment {
        target: Box<Expression>,
        operator: CompoundAssignOp,
        value: Box<Expression>,
    },
    
    /// Function call: callee(args)
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    
    /// Optional call: callee?.(args)
    OptionalCall {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    
    /// New expression: new Constructor(args)
    New(Box<Expression>),
    
    /// New with arguments: new Constructor(args)
    NewWithArgs {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    
    /// Member access: object.property or object[property]
    Member {
        object: Box<Expression>,
        property: Box<Expression>,
        computed: bool,  // true for object[property]
    },
    
    /// Optional member access: object?.property
    OptionalMember {
        object: Box<Expression>,
        property: Box<Expression>,
        computed: bool,
    },
    
    /// Array literal: [a, b, c]
    Array(Vec<Expression>),
    
    /// Array with holes: [a, , c]
    ArrayWithHoles(Vec<Option<Expression>>),
    
    /// Object literal: { key: value }
    Object(Vec<(String, Expression)>),
    
    /// Object with computed keys
    ObjectWithComputed(Vec<ObjectProperty>),
    
    /// Function expression
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    
    /// Async function expression
    AsyncFunction {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    
    /// Generator function expression
    GeneratorFunction {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    
    /// Arrow function: (params) => body
    Arrow {
        params: Vec<String>,
        body: Box<Statement>,
    },
    
    /// Async arrow function
    AsyncArrow {
        params: Vec<String>,
        body: Box<Statement>,
    },
    
    /// Ternary: condition ? then : else
    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    
    /// Sequence (comma) expression
    Sequence(Vec<Expression>),
    
    /// typeof expression
    Typeof(Box<Expression>),
    
    /// delete expression
    Delete(Box<Expression>),
    
    /// void expression
    Void(Box<Expression>),
    
    /// Spread operator: ...expr
    Spread(Box<Expression>),
    
    /// Await expression
    Await(Box<Expression>),
    
    /// Yield expression
    Yield(Option<Box<Expression>>),
    
    /// Yield* (delegate)
    YieldDelegate(Box<Expression>),
    
    /// Class expression
    ClassExpression {
        name: Option<String>,
        extends: Option<Box<Expression>>,
        body: Vec<ClassMember>,
    },
    
    /// Destructuring assignment
    DestructuringAssignment {
        pattern: DestructuringPattern,
        value: Box<Expression>,
    },
    
    /// Import expression (dynamic import)
    Import(Box<Expression>),
    
    /// Meta property: new.target, import.meta
    MetaProperty { meta: String, property: String },
}

/// Object property (with computed key support)
#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub key: ObjectKey,
    pub value: Expression,
    pub shorthand: bool,
    pub method: bool,
}

/// Object key types
#[derive(Debug, Clone)]
pub enum ObjectKey {
    Identifier(String),
    String(String),
    Number(f64),
    Computed(Expression),
}

/// Destructuring patterns
#[derive(Debug, Clone)]
pub enum DestructuringPattern {
    Array(Vec<Option<DestructuringElement>>),
    Object(Vec<DestructuringProperty>),
}

/// Array destructuring element
#[derive(Debug, Clone)]
pub struct DestructuringElement {
    pub target: DestructuringTarget,
    pub default_value: Option<Expression>,
}

/// Object destructuring property
#[derive(Debug, Clone)]
pub struct DestructuringProperty {
    pub key: String,
    pub value: DestructuringTarget,
    pub default_value: Option<Expression>,
    pub shorthand: bool,
}

/// Destructuring target
#[derive(Debug, Clone)]
pub enum DestructuringTarget {
    Identifier(String),
    Pattern(Box<DestructuringPattern>),
    Rest(String),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Number(n) => write!(f, "{}", n),
            Expression::BigInt(n) => write!(f, "{}n", n),
            Expression::String(s) => write!(f, "\"{}\"", s),
            Expression::Boolean(b) => write!(f, "{}", b),
            Expression::Null => write!(f, "null"),
            Expression::Undefined => write!(f, "undefined"),
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::This => write!(f, "this"),
            Expression::Super => write!(f, "super"),
            Expression::RegExp { pattern, flags } => write!(f, "/{}/{}", pattern, flags),
            _ => write!(f, "[expression]"),
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,            // +
    Subtract,       // -
    Multiply,       // *
    Divide,         // /
    Modulo,         // %
    Exponent,       // **
    Equal,          // ==
    StrictEqual,    // ===
    NotEqual,       // !=
    StrictNotEqual, // !==
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    And,            // &&
    Or,             // ||
    BitAnd,         // &
    BitOr,          // |
    BitXor,         // ^
    LeftShift,      // <<
    RightShift,     // >>
    UnsignedRightShift, // >>>
    NullishCoalesce, // ??
    Instanceof,     // instanceof
    In,             // in
}

/// Compound assignment operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompoundAssignOp {
    Add,            // +=
    Subtract,       // -=
    Multiply,       // *=
    Divide,         // /=
    Modulo,         // %=
    Exponent,       // **=
    BitAnd,         // &=
    BitOr,          // |=
    BitXor,         // ^=
    LeftShift,      // <<=
    RightShift,     // >>=
    UnsignedRightShift, // >>>=
    And,            // &&=
    Or,             // ||=
    NullishCoalesce, // ??=
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Negate,     // -
    Not,        // !
    Typeof,     // typeof (legacy)
    BitNot,     // ~
    Plus,       // +
}
