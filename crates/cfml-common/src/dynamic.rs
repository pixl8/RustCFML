//! Dynamic value types for CFML runtime

use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum CfmlValue {
    Null,
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    Array(Vec<CfmlValue>),
    Struct(IndexMap<String, CfmlValue>),
    Closure(Box<CfmlClosure>),
    Component(Box<CfmlComponent>),
    Function(CfmlFunction),
    Query(CfmlQuery),
    Binary(Vec<u8>),
}

impl CfmlValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            CfmlValue::Null => "Null",
            CfmlValue::Bool(_) => "Boolean",
            CfmlValue::Int(_) => "Integer",
            CfmlValue::Double(_) => "Double",
            CfmlValue::String(_) => "String",
            CfmlValue::Array(_) => "Array",
            CfmlValue::Struct(_) => "Struct",
            CfmlValue::Closure(_) => "Closure",
            CfmlValue::Component(_) => "Component",
            CfmlValue::Function(_) => "Function",
            CfmlValue::Query(_) => "Query",
            CfmlValue::Binary(_) => "Binary",
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            CfmlValue::Null => false,
            CfmlValue::Bool(b) => *b,
            CfmlValue::Int(i) => *i != 0,
            CfmlValue::Double(d) => *d != 0.0,
            CfmlValue::String(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return false;
                }
                match trimmed.to_lowercase().as_str() {
                    "false" | "no" | "0" => false,
                    _ => true,
                }
            }
            CfmlValue::Array(a) => !a.is_empty(),
            CfmlValue::Struct(s) => !s.is_empty(),
            CfmlValue::Closure(_) => true,
            CfmlValue::Component(_) => true,
            CfmlValue::Function(_) => true,
            CfmlValue::Query(q) => !q.rows.is_empty(),
            CfmlValue::Binary(b) => !b.is_empty(),
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            CfmlValue::Null => String::new(),
            CfmlValue::Bool(b) => b.to_string(),
            CfmlValue::Int(i) => i.to_string(),
            CfmlValue::Double(d) => d.to_string(),
            CfmlValue::String(s) => s.clone(),
            CfmlValue::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.as_string()).collect();
                format!("[{}]", items.join(", "))
            }
            CfmlValue::Struct(s) => {
                let items: Vec<String> = s
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.as_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            CfmlValue::Closure(_) => "<Closure>".to_string(),
            CfmlValue::Component(_) => "<Component>".to_string(),
            CfmlValue::Function(f) => f.name.clone(),
            CfmlValue::Query(_) => "<Query>".to_string(),
            CfmlValue::Binary(_) => "<Binary>".to_string(),
        }
    }

    pub fn get(&self, key: &str) -> Option<CfmlValue> {
        match self {
            CfmlValue::Struct(s) => s.get(key).cloned(),
            CfmlValue::Array(a) => {
                if let Ok(idx) = key.parse::<usize>() {
                    a.get(idx).cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn set(&mut self, key: String, value: CfmlValue) {
        match self {
            CfmlValue::Struct(s) => {
                s.insert(key, value);
            }
            CfmlValue::Array(a) => {
                if let Ok(idx) = key.parse::<usize>() {
                    if idx < a.len() {
                        a[idx] = value;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn as_array(&self) -> Option<&Vec<CfmlValue>> {
        match self {
            CfmlValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_struct(&self) -> Option<&IndexMap<String, CfmlValue>> {
        match self {
            CfmlValue::Struct(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<CfmlValue>> {
        match self {
            CfmlValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_struct_mut(&mut self) -> Option<&mut IndexMap<String, CfmlValue>> {
        match self {
            CfmlValue::Struct(s) => Some(s),
            _ => None,
        }
    }

    pub fn eq(&self, other: &CfmlValue) -> bool {
        match (self, other) {
            (CfmlValue::Null, CfmlValue::Null) => true,
            (CfmlValue::Bool(a), CfmlValue::Bool(b)) => a == b,
            (CfmlValue::Int(a), CfmlValue::Int(b)) => a == b,
            (CfmlValue::Double(a), CfmlValue::Double(b)) => a == b,
            (CfmlValue::String(a), CfmlValue::String(b)) => a.to_lowercase() == b.to_lowercase(),
            (CfmlValue::Int(a), CfmlValue::Double(b)) => *a as f64 == *b,
            (CfmlValue::Double(a), CfmlValue::Int(b)) => *a == *b as f64,
            (CfmlValue::Array(a), CfmlValue::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x.eq(y))
            }
            (CfmlValue::Struct(a), CfmlValue::Struct(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter()
                    .all(|(k, v)| b.get(k).map(|bv| v.eq(bv)).unwrap_or(false))
            }
            _ => false,
        }
    }
}

impl Default for CfmlValue {
    fn default() -> Self {
        CfmlValue::Null
    }
}

impl fmt::Display for CfmlValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

#[derive(Debug, Clone)]
pub struct CfmlClosure {
    pub params: Vec<String>,
    pub body: Box<CfmlClosureBody>,
    pub captured_vars: IndexMap<String, CfmlValue>,
}

#[derive(Debug, Clone)]
pub enum CfmlClosureBody {
    Expression(Box<CfmlValue>),
    Statements(Vec<CfmlStatement>),
}

#[derive(Debug, Clone)]
pub enum CfmlStatement {
    Expression(CfmlValue),
    Return(Option<CfmlValue>),
    Assignment(String, CfmlValue),
}

#[derive(Debug, Clone)]
pub struct CfmlComponent {
    pub name: String,
    pub properties: IndexMap<String, CfmlValue>,
    pub methods: HashMap<String, CfmlFunction>,
    pub extends: Option<String>,
    pub implements: Vec<String>,
}

impl CfmlComponent {
    pub fn new(name: String) -> Self {
        Self {
            name,
            properties: IndexMap::new(),
            methods: HashMap::new(),
            extends: None,
            implements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CfmlFunction {
    pub name: String,
    pub params: Vec<CfmlParam>,
    pub body: CfmlClosureBody,
    pub return_type: Option<String>,
    pub access: CfmlAccess,
    /// Captured scope for closures — shared mutable environment so multiple
    /// invocations (and sibling closures) see each other's mutations.
    pub captured_scope: Option<Arc<RwLock<IndexMap<String, CfmlValue>>>>,
}

#[derive(Debug, Clone)]
pub struct CfmlParam {
    pub name: String,
    pub param_type: Option<String>,
    pub default: Option<CfmlValue>,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CfmlAccess {
    Public,
    Private,
    Package,
    Remote,
}

#[derive(Debug, Clone)]
pub struct CfmlQuery {
    pub columns: Vec<String>,
    pub rows: Vec<IndexMap<String, CfmlValue>>,
    pub sql: Option<String>,
}

impl CfmlQuery {
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            sql: None,
        }
    }
}
