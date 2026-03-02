//! Virtual machine types and context

use crate::dynamic::CfmlValue;
use indexmap::IndexMap;

pub type CfmlResult = Result<CfmlValue, CfmlError>;

#[derive(Debug, Clone)]
pub struct CfmlError {
    pub message: String,
    pub error_type: CfmlErrorType,
    pub stack_trace: Vec<StackFrame>,
}

#[derive(Debug, Clone)]
pub enum CfmlErrorType {
    Runtime,
    Compile,
    Expression,
    Template,
    Application,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function: String,
    pub template: String,
    pub line: usize,
}

impl CfmlError {
    pub fn new(message: String, error_type: CfmlErrorType) -> Self {
        Self {
            message,
            error_type,
            stack_trace: Vec::new(),
        }
    }

    pub fn runtime(message: String) -> Self {
        Self::new(message, CfmlErrorType::Runtime)
    }
}

impl std::fmt::Display for CfmlErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CfmlErrorType::Runtime => write!(f, "Runtime"),
            CfmlErrorType::Compile => write!(f, "Compile"),
            CfmlErrorType::Expression => write!(f, "Expression"),
            CfmlErrorType::Template => write!(f, "Template"),
            CfmlErrorType::Application => write!(f, "Application"),
            CfmlErrorType::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Display for CfmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Error: {}", self.error_type, self.message)?;
        if !self.stack_trace.is_empty() {
            write!(f, "\n\nStack trace (most recent call first):")?;
            for (i, frame) in self.stack_trace.iter().enumerate() {
                let template = if frame.template.is_empty() { "<inline>" } else { &frame.template };
                let func = if frame.function == "__main__" { "(main)" } else { &frame.function };
                write!(f, "\n  {}: {} ({}:{})", i + 1, func, template, frame.line)?;
            }
        }
        Ok(())
    }
}

pub struct CfmlContext {
    pub scopes: Vec<IndexMap<String, CfmlValue>>,
    pub this: Option<CfmlValue>,
    pub super_scope: Option<CfmlValue>,
    pub variables: IndexMap<String, CfmlValue>,
    pub local_vars: IndexMap<String, CfmlValue>,
    pub output_buffer: String,
    pub output_enabled: bool,
}

impl CfmlContext {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            this: None,
            super_scope: None,
            variables: IndexMap::new(),
            local_vars: IndexMap::new(),
            output_buffer: String::new(),
            output_enabled: true,
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(IndexMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_var(&self, name: &str) -> Option<CfmlValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        self.local_vars
            .get(name)
            .cloned()
            .or_else(|| self.variables.get(name).cloned())
    }

    pub fn set_var(&mut self, name: String, value: CfmlValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        } else {
            self.variables.insert(name, value);
        }
    }

    pub fn write_output(&mut self, value: &str) {
        if self.output_enabled {
            self.output_buffer.push_str(value);
        }
    }
}

impl Default for CfmlContext {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CfmlFrame {
    pub name: String,
    pub ip: usize,
    pub stack: Vec<CfmlValue>,
    pub locals: IndexMap<String, CfmlValue>,
}

impl CfmlFrame {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ip: 0,
            stack: Vec::new(),
            locals: IndexMap::new(),
        }
    }
}
