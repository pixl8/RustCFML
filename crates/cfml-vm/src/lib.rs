//! CFML Virtual Machine - Bytecode execution engine

use cfml_codegen::{BytecodeFunction, BytecodeOp, BytecodeProgram};
use cfml_common::dynamic::CfmlValue;
use cfml_common::vm::{CfmlError, CfmlErrorType, CfmlResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type BuiltinFunction = fn(Vec<CfmlValue>) -> CfmlResult;

/// Persistent application state, keyed by app name.
pub struct ApplicationState {
    pub name: String,
    pub variables: HashMap<String, CfmlValue>,
    pub started: bool,
    pub config: HashMap<String, CfmlValue>,
}

/// Server-level state, persists across requests in --serve mode.
#[derive(Clone)]
pub struct ServerState {
    pub applications: Arc<Mutex<HashMap<String, ApplicationState>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            applications: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub struct CfmlVirtualMachine {
    pub program: BytecodeProgram,
    pub globals: HashMap<String, CfmlValue>,
    pub builtins: HashMap<String, BuiltinFunction>,
    pub output_buffer: String,
    /// User-defined functions (name -> function index in program.functions)
    pub user_functions: HashMap<String, usize>,
    /// Source file path (for include resolution)
    pub source_file: Option<String>,
    /// Call stack for tracking execution
    #[allow(dead_code)]
    call_stack: Vec<CallFrame>,
    /// Try-catch handler stack
    try_stack: Vec<TryHandler>,
    /// Current exception (if any)
    #[allow(dead_code)]
    current_exception: Option<CfmlValue>,
    /// After a component method executes, holds the modified `this` for write-back
    /// to the caller's object variable. Set by execute_function_with_args.
    method_this_writeback: Option<CfmlValue>,
    /// Request scope — lives for the duration of one request
    pub request_scope: HashMap<String, CfmlValue>,
    /// Application scope — shared across requests (Arc<Mutex> for thread safety)
    pub application_scope: Option<Arc<Mutex<HashMap<String, CfmlValue>>>>,
    /// Server-level state — persists across requests in --serve mode
    pub server_state: Option<ServerState>,
    /// HTTP response headers set by cfheader
    pub response_headers: Vec<(String, String)>,
    /// HTTP response status code set by cfheader
    pub response_status: Option<(u16, String)>,
    /// Content type set by cfcontent
    pub response_content_type: Option<String>,
    /// Body override set by cfcontent (variable/file)
    pub response_body: Option<CfmlValue>,
    /// Redirect URL set by cflocation
    pub redirect_url: Option<String>,
    /// HTTP request data for getHTTPRequestData()
    pub http_request_data: Option<CfmlValue>,
    /// Stack of saved output buffers for cfsavecontent
    pub saved_output_buffers: Vec<String>,
    /// Base template path (original .cfm being served)
    pub base_template_path: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CallFrame {
    function_name: String,
    return_ip: usize,
    return_func_idx: usize,
    locals: HashMap<String, CfmlValue>,
    stack_base: usize,
}

#[derive(Debug, Clone)]
struct TryHandler {
    catch_ip: usize,
    stack_depth: usize,
}

impl CfmlVirtualMachine {
    pub fn new(program: BytecodeProgram) -> Self {
        Self {
            program,
            globals: HashMap::new(),
            builtins: HashMap::new(),
            output_buffer: String::new(),
            user_functions: HashMap::new(),
            source_file: None,
            call_stack: Vec::new(),
            try_stack: Vec::new(),
            current_exception: None,
            method_this_writeback: None,
            request_scope: HashMap::new(),
            application_scope: None,
            server_state: None,
            response_headers: Vec::new(),
            response_status: None,
            response_content_type: None,
            response_body: None,
            redirect_url: None,
            http_request_data: None,
            saved_output_buffers: Vec::new(),
            base_template_path: None,
        }
    }

    pub fn execute(&mut self) -> CfmlResult {
        let main_idx = self
            .program
            .functions
            .iter()
            .position(|f| f.name == "__main__")
            .ok_or_else(|| CfmlError::runtime("No main function found".to_string()))?;

        self.execute_function_by_index(main_idx, Vec::new())
    }

    fn execute_function_by_index(
        &mut self,
        func_idx: usize,
        args: Vec<CfmlValue>,
    ) -> CfmlResult {
        let func = self.program.functions[func_idx].clone();
        self.execute_function_with_args(&func, args, None)
    }

    fn execute_function_with_args(
        &mut self,
        func: &BytecodeFunction,
        args: Vec<CfmlValue>,
        parent_scope: Option<&HashMap<String, CfmlValue>>,
    ) -> CfmlResult {
        let mut locals: HashMap<String, CfmlValue> = HashMap::new();
        let mut stack: Vec<CfmlValue> = Vec::new();
        let mut ip = 0;

        // Copy parent scope variables (closures and nested functions see parent vars)
        if let Some(parent) = parent_scope {
            for (k, v) in parent {
                locals.insert(k.clone(), v.clone());
            }
        }

        // Build CFML arguments scope
        let mut arguments_map = HashMap::new();
        for (i, param_name) in func.params.iter().enumerate() {
            let value = args.get(i).cloned().unwrap_or(CfmlValue::Null);
            locals.insert(param_name.clone(), value.clone());
            arguments_map.insert(param_name.clone(), value.clone());
            // Also make args accessible by position (1-based)
            arguments_map.insert((i + 1).to_string(), value);
        }
        // Also add any extra positional args beyond declared params
        for i in func.params.len()..args.len() {
            arguments_map.insert((i + 1).to_string(), args[i].clone());
        }
        locals.insert("arguments".to_string(), CfmlValue::Struct(arguments_map));

        loop {
            if ip >= func.instructions.len() {
                break;
            }

            let op = func.instructions[ip].clone();
            ip += 1;

            match op {
                BytecodeOp::Null => stack.push(CfmlValue::Null),
                BytecodeOp::True => stack.push(CfmlValue::Bool(true)),
                BytecodeOp::False => stack.push(CfmlValue::Bool(false)),
                BytecodeOp::Integer(n) => stack.push(CfmlValue::Int(n)),
                BytecodeOp::Double(d) => stack.push(CfmlValue::Double(d)),
                BytecodeOp::String(s) => stack.push(CfmlValue::String(s)),

                BytecodeOp::LoadLocal(name) => {
                    // Handle CFML scope references
                    let name_lower = name.to_lowercase();
                    let val = if name_lower == "local" || name_lower == "variables" {
                        // Return a struct representing the local/variables scope
                        CfmlValue::Struct(locals.clone())
                    } else if name_lower == "request" {
                        CfmlValue::Struct(self.request_scope.clone())
                    } else if name_lower == "application" {
                        if let Some(ref app_scope) = self.application_scope {
                            if let Ok(scope) = app_scope.lock() {
                                CfmlValue::Struct(scope.clone())
                            } else {
                                CfmlValue::Struct(HashMap::new())
                            }
                        } else {
                            CfmlValue::Struct(HashMap::new())
                        }
                    } else if name_lower == "server" {
                        let mut info = HashMap::new();
                        info.insert("coldfusion".to_string(), CfmlValue::Struct({
                            let mut cf = HashMap::new();
                            cf.insert("productname".to_string(), CfmlValue::String("RustCFML".to_string()));
                            cf.insert("productversion".to_string(), CfmlValue::String(env!("CARGO_PKG_VERSION").to_string()));
                            cf
                        }));
                        info.insert("os".to_string(), CfmlValue::Struct({
                            let mut os = HashMap::new();
                            os.insert("name".to_string(), CfmlValue::String(std::env::consts::OS.to_string()));
                            os.insert("arch".to_string(), CfmlValue::String(std::env::consts::ARCH.to_string()));
                            os
                        }));
                        CfmlValue::Struct(info)
                    } else if let Some(val) = locals.get(&name) {
                        val.clone()
                    } else if let Some(val) = self.globals.get(&name) {
                        val.clone()
                    } else {
                        // Case-insensitive local lookup
                        locals
                            .iter()
                            .find(|(k, _)| k.to_lowercase() == name_lower)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(CfmlValue::Null)
                    };
                    stack.push(val);
                }
                BytecodeOp::StoreLocal(name) => {
                    if let Some(val) = stack.pop() {
                        let name_lower = name.to_lowercase();
                        if name_lower == "request" {
                            if let CfmlValue::Struct(s) = &val {
                                self.request_scope = s.clone();
                            }
                        } else if name_lower == "application" {
                            if let CfmlValue::Struct(s) = &val {
                                if let Some(ref app_scope) = self.application_scope {
                                    if let Ok(mut scope) = app_scope.lock() {
                                        *scope = s.clone();
                                    }
                                }
                            }
                        } else {
                            locals.insert(name, val);
                        }
                    }
                }
                BytecodeOp::LoadGlobal(name) => {
                    let name_lower = name.to_lowercase();
                    // 1. Check locals (exact, then CI)
                    if let Some(val) = locals.get(&name) {
                        stack.push(val.clone());
                    } else if let Some(val) = locals.iter()
                        .find(|(k, _)| k.to_lowercase() == name_lower)
                        .map(|(_, v)| v.clone()) {
                        stack.push(val);
                    // 2. Check globals (exact, then CI)
                    } else if let Some(val) = self.globals.get(&name) {
                        stack.push(val.clone());
                    } else if let Some(val) = self.globals.iter()
                        .find(|(k, _)| k.to_lowercase() == name_lower)
                        .map(|(_, v)| v.clone()) {
                        stack.push(val);
                    // 3. Check builtins/user_functions (exact, then CI)
                    } else if self.builtins.contains_key(&name) || self.user_functions.contains_key(&name) {
                        stack.push(CfmlValue::Function(cfml_common::dynamic::CfmlFunction {
                            name: name.clone(),
                            params: Vec::new(),
                            body: cfml_common::dynamic::CfmlClosureBody::Expression(
                                Box::new(CfmlValue::Null),
                            ),
                            return_type: None,
                            access: cfml_common::dynamic::CfmlAccess::Public,
                        }));
                    } else if self.builtins.keys().any(|k| k.to_lowercase() == name_lower)
                           || self.user_functions.keys().any(|k| k.to_lowercase() == name_lower) {
                        let canonical = self.builtins.keys()
                            .find(|k| k.to_lowercase() == name_lower)
                            .or_else(|| self.user_functions.keys()
                                .find(|k| k.to_lowercase() == name_lower))
                            .cloned()
                            .unwrap_or(name.clone());
                        stack.push(CfmlValue::Function(cfml_common::dynamic::CfmlFunction {
                            name: canonical,
                            params: Vec::new(),
                            body: cfml_common::dynamic::CfmlClosureBody::Expression(
                                Box::new(CfmlValue::Null),
                            ),
                            return_type: None,
                            access: cfml_common::dynamic::CfmlAccess::Public,
                        }));
                    } else {
                        stack.push(CfmlValue::Null);
                    }
                }
                BytecodeOp::StoreGlobal(name) => {
                    if let Some(val) = stack.pop() {
                        self.globals.insert(name, val);
                    }
                }

                BytecodeOp::Pop => {
                    stack.pop();
                }
                BytecodeOp::Dup => {
                    if let Some(val) = stack.last() {
                        stack.push(val.clone());
                    }
                }
                BytecodeOp::Swap => {
                    let len = stack.len();
                    if len >= 2 {
                        stack.swap(len - 1, len - 2);
                    }
                }

                // Arithmetic
                BytecodeOp::Add => {
                    binary_op(&mut stack, |a, b| match (&a, &b) {
                        (CfmlValue::Int(i), CfmlValue::Int(j)) => CfmlValue::Int(i + j),
                        (CfmlValue::Double(x), CfmlValue::Double(y)) => CfmlValue::Double(x + y),
                        (CfmlValue::Int(i), CfmlValue::Double(d)) => CfmlValue::Double(*i as f64 + d),
                        (CfmlValue::Double(d), CfmlValue::Int(i)) => CfmlValue::Double(d + *i as f64),
                        (CfmlValue::String(s), CfmlValue::String(t)) => {
                            CfmlValue::String(format!("{}{}", s, t))
                        }
                        // CFML: try numeric coercion
                        _ => {
                            let a_num = to_number(&a);
                            let b_num = to_number(&b);
                            match (a_num, b_num) {
                                (Some(x), Some(y)) => CfmlValue::Double(x + y),
                                _ => CfmlValue::String(format!("{}{}", a.as_string(), b.as_string())),
                            }
                        }
                    });
                }
                BytecodeOp::Sub => {
                    binary_op(&mut stack, |a, b| numeric_op(&a, &b, |x, y| x - y));
                }
                BytecodeOp::Mul => {
                    binary_op(&mut stack, |a, b| numeric_op(&a, &b, |x, y| x * y));
                }
                BytecodeOp::Div => {
                    binary_op(&mut stack, |a, b| {
                        let x = to_number(&a).unwrap_or(0.0);
                        let y = to_number(&b).unwrap_or(1.0);
                        if y == 0.0 {
                            CfmlValue::Double(f64::INFINITY)
                        } else {
                            CfmlValue::Double(x / y)
                        }
                    });
                }
                BytecodeOp::Mod => {
                    binary_op(&mut stack, |a, b| {
                        match (&a, &b) {
                            (CfmlValue::Int(i), CfmlValue::Int(j)) if *j != 0 => CfmlValue::Int(i % j),
                            _ => {
                                let x = to_number(&a).unwrap_or(0.0);
                                let y = to_number(&b).unwrap_or(1.0);
                                CfmlValue::Double(x % y)
                            }
                        }
                    });
                }
                BytecodeOp::Pow => {
                    binary_op(&mut stack, |a, b| {
                        let x = to_number(&a).unwrap_or(0.0);
                        let y = to_number(&b).unwrap_or(0.0);
                        CfmlValue::Double(x.powf(y))
                    });
                }
                BytecodeOp::IntDiv => {
                    binary_op(&mut stack, |a, b| {
                        let x = to_number(&a).unwrap_or(0.0) as i64;
                        let y = to_number(&b).unwrap_or(1.0) as i64;
                        if y == 0 {
                            CfmlValue::Int(0)
                        } else {
                            CfmlValue::Int(x / y)
                        }
                    });
                }
                BytecodeOp::Negate => {
                    if let Some(val) = stack.pop() {
                        match val {
                            CfmlValue::Int(i) => stack.push(CfmlValue::Int(-i)),
                            CfmlValue::Double(d) => stack.push(CfmlValue::Double(-d)),
                            _ => {
                                if let Some(n) = to_number(&val) {
                                    stack.push(CfmlValue::Double(-n));
                                } else {
                                    stack.push(CfmlValue::Int(0));
                                }
                            }
                        }
                    }
                }

                // String concatenation
                BytecodeOp::Concat => {
                    binary_op(&mut stack, |a, b| {
                        CfmlValue::String(format!("{}{}", a.as_string(), b.as_string()))
                    });
                }

                // Comparison - proper value comparison
                BytecodeOp::Eq => {
                    compare_op(&mut stack, |a, b| cfml_equal(a, b));
                }
                BytecodeOp::Neq => {
                    compare_op(&mut stack, |a, b| !cfml_equal(a, b));
                }
                BytecodeOp::Lt => {
                    compare_op(&mut stack, |a, b| cfml_compare(a, b) < 0);
                }
                BytecodeOp::Lte => {
                    compare_op(&mut stack, |a, b| cfml_compare(a, b) <= 0);
                }
                BytecodeOp::Gt => {
                    compare_op(&mut stack, |a, b| cfml_compare(a, b) > 0);
                }
                BytecodeOp::Gte => {
                    compare_op(&mut stack, |a, b| cfml_compare(a, b) >= 0);
                }

                // CFML-specific operators
                BytecodeOp::Contains => {
                    compare_op(&mut stack, |a, b| {
                        let haystack = a.as_string().to_lowercase();
                        let needle = b.as_string().to_lowercase();
                        haystack.contains(&needle)
                    });
                }
                BytecodeOp::DoesNotContain => {
                    compare_op(&mut stack, |a, b| {
                        let haystack = a.as_string().to_lowercase();
                        let needle = b.as_string().to_lowercase();
                        !haystack.contains(&needle)
                    });
                }

                // Logical
                BytecodeOp::And => {
                    binary_op(&mut stack, |a, b| {
                        CfmlValue::Bool(a.is_true() && b.is_true())
                    });
                }
                BytecodeOp::Or => {
                    binary_op(&mut stack, |a, b| {
                        CfmlValue::Bool(a.is_true() || b.is_true())
                    });
                }
                BytecodeOp::Not => {
                    if let Some(a) = stack.pop() {
                        stack.push(CfmlValue::Bool(!a.is_true()));
                    }
                }
                BytecodeOp::Xor => {
                    binary_op(&mut stack, |a, b| {
                        CfmlValue::Bool(a.is_true() ^ b.is_true())
                    });
                }
                BytecodeOp::Eqv => {
                    binary_op(&mut stack, |a, b| {
                        CfmlValue::Bool(a.is_true() == b.is_true())
                    });
                }
                BytecodeOp::Imp => {
                    binary_op(&mut stack, |a, b| {
                        CfmlValue::Bool(!a.is_true() || b.is_true())
                    });
                }

                // Control flow
                BytecodeOp::Jump(target) => {
                    ip = target;
                }
                BytecodeOp::JumpIfFalse(target) => {
                    if let Some(cond) = stack.pop() {
                        if !cond.is_true() {
                            ip = target;
                        }
                    }
                }
                BytecodeOp::JumpIfTrue(target) => {
                    if let Some(cond) = stack.pop() {
                        if cond.is_true() {
                            ip = target;
                        }
                    }
                }

                BytecodeOp::Call(arg_count) => {
                    let args: Vec<CfmlValue> =
                        (0..arg_count).filter_map(|_| stack.pop()).collect::<Vec<_>>().into_iter().rev().collect();

                    if let Some(func_ref) = stack.pop() {
                        let result = self.call_function(&func_ref, args, &locals)?;
                        stack.push(result);
                    } else {
                        stack.push(CfmlValue::Null);
                    }
                }

                BytecodeOp::Return => {
                    // Save modified 'this' for component method write-back
                    if let Some(this_val) = locals.get("this") {
                        self.method_this_writeback = Some(this_val.clone());
                    }
                    return Ok(stack.pop().unwrap_or(CfmlValue::Null));
                }

                // Collections
                BytecodeOp::BuildArray(count) => {
                    let mut elements = Vec::new();
                    for _ in 0..count {
                        if let Some(val) = stack.pop() {
                            elements.push(val);
                        }
                    }
                    elements.reverse();
                    stack.push(CfmlValue::Array(elements));
                }
                BytecodeOp::BuildStruct(count) => {
                    let mut pairs = Vec::new();
                    for _ in 0..count {
                        let value = stack.pop().unwrap_or(CfmlValue::Null);
                        let key = stack.pop().unwrap_or(CfmlValue::String(String::new()));
                        pairs.push((key.as_string(), value));
                    }
                    let mut map = HashMap::new();
                    for (k, v) in pairs.into_iter().rev() {
                        map.insert(k, v);
                    }
                    stack.push(CfmlValue::Struct(map));
                }
                BytecodeOp::GetIndex => {
                    let index = stack.pop().unwrap_or(CfmlValue::Null);
                    let collection = stack.pop().unwrap_or(CfmlValue::Null);
                    match &collection {
                        CfmlValue::Array(arr) => {
                            let idx = match &index {
                                CfmlValue::Int(i) => *i as usize,
                                CfmlValue::Double(d) => *d as usize,
                                CfmlValue::String(s) => s.parse::<usize>().unwrap_or(0),
                                _ => 0,
                            };
                            // CFML arrays are 1-based
                            let idx = if idx > 0 { idx - 1 } else { 0 };
                            stack.push(arr.get(idx).cloned().unwrap_or(CfmlValue::Null));
                        }
                        CfmlValue::Struct(s) => {
                            let key = index.as_string();
                            stack.push(
                                s.get(&key)
                                    .or_else(|| s.get(&key.to_uppercase()))
                                    .or_else(|| s.get(&key.to_lowercase()))
                                    .cloned()
                                    .unwrap_or(CfmlValue::Null),
                            );
                        }
                        _ => stack.push(CfmlValue::Null),
                    }
                }
                BytecodeOp::SetIndex => {
                    let index = stack.pop().unwrap_or(CfmlValue::Null);
                    let mut collection = stack.pop().unwrap_or(CfmlValue::Null);
                    let value = stack.pop().unwrap_or(CfmlValue::Null);
                    match &mut collection {
                        CfmlValue::Array(arr) => {
                            let idx = match &index {
                                CfmlValue::Int(i) => (*i as usize).saturating_sub(1), // 1-based
                                _ => 0,
                            };
                            if idx < arr.len() {
                                arr[idx] = value;
                            }
                        }
                        CfmlValue::Struct(s) => {
                            s.insert(index.as_string(), value);
                        }
                        _ => {}
                    }
                    stack.push(collection);
                }

                BytecodeOp::GetProperty(name) => {
                    if let Some(obj) = stack.pop() {
                        match &obj {
                            CfmlValue::Struct(s) => {
                                let val = s
                                    .get(&name)
                                    .or_else(|| s.get(&name.to_uppercase()))
                                    .or_else(|| s.get(&name.to_lowercase()))
                                    .cloned()
                                    .unwrap_or(CfmlValue::Null);
                                stack.push(val);
                            }
                            CfmlValue::Array(arr) => {
                                // Array member functions
                                match name.to_lowercase().as_str() {
                                    "len" | "length" => {
                                        stack.push(CfmlValue::Int(arr.len() as i64));
                                    }
                                    _ => stack.push(CfmlValue::Null),
                                }
                            }
                            CfmlValue::String(s) => {
                                // String member functions
                                match name.to_lowercase().as_str() {
                                    "len" | "length" => {
                                        stack.push(CfmlValue::Int(s.len() as i64));
                                    }
                                    _ => stack.push(CfmlValue::Null),
                                }
                            }
                            CfmlValue::Query(q) => {
                                match name.to_lowercase().as_str() {
                                    "recordcount" => {
                                        stack.push(CfmlValue::Int(q.rows.len() as i64));
                                    }
                                    "columnlist" => {
                                        stack.push(CfmlValue::String(q.columns.join(",")));
                                    }
                                    _ => {
                                        // Column access: q.columnName returns array of values
                                        let col_lower = name.to_lowercase();
                                        let is_col = q.columns.iter().any(|c| c.to_lowercase() == col_lower);
                                        if is_col {
                                            let col_data: Vec<CfmlValue> = q.rows.iter()
                                                .map(|row| {
                                                    row.iter()
                                                        .find(|(k, _)| k.to_lowercase() == col_lower)
                                                        .map(|(_, v)| v.clone())
                                                        .unwrap_or(CfmlValue::Null)
                                                })
                                                .collect();
                                            stack.push(CfmlValue::Array(col_data));
                                        } else {
                                            stack.push(CfmlValue::Null);
                                        }
                                    }
                                }
                            }
                            _ => {
                                stack.push(obj.get(&name).unwrap_or(CfmlValue::Null));
                            }
                        }
                    } else {
                        stack.push(CfmlValue::Null);
                    }
                }
                BytecodeOp::SetProperty(name) => {
                    if let Some(value) = stack.pop() {
                        if let Some(mut obj) = stack.pop() {
                            obj.set(name, value);
                            stack.push(obj);
                        }
                    }
                }

                BytecodeOp::NewObject(arg_count) => {
                    // Pop constructor arguments first
                    let ctor_args: Vec<CfmlValue> = (0..arg_count)
                        .filter_map(|_| stack.pop())
                        .collect::<Vec<_>>().into_iter().rev().collect();

                    if let Some(class_ref) = stack.pop() {
                        // Resolve the component template
                        let template = if let CfmlValue::Struct(s) = &class_ref {
                            CfmlValue::Struct(s.clone())
                        } else {
                            let class_name = match &class_ref {
                                CfmlValue::Function(f) => f.name.clone(),
                                CfmlValue::String(s) => s.clone(),
                                _ => class_ref.as_string(),
                            };
                            self.resolve_component_template(&class_name, &locals)
                                .unwrap_or(CfmlValue::Struct(HashMap::new()))
                        };

                        // Resolve inheritance chain
                        let instance = self.resolve_inheritance(template, &locals);

                        // Call init() constructor if present
                        let final_instance = if let CfmlValue::Struct(ref s) = instance {
                            let has_init = s.get("init")
                                .or_else(|| s.get("INIT"))
                                .or_else(|| s.get("Init"))
                                .cloned();
                            if let Some(ref init_func) = has_init {
                                if matches!(init_func, CfmlValue::Function(_)) {
                                    let mut init_locals = locals.clone();
                                    init_locals.insert("this".to_string(), instance.clone());
                                    if let Ok(result) = self.call_function(init_func, ctor_args, &init_locals) {
                                        // After init, check method_this_writeback for modified this
                                        if let Some(modified_this) = self.method_this_writeback.take() {
                                            modified_this
                                        } else if let CfmlValue::Struct(_) = &result {
                                            result
                                        } else {
                                            instance
                                        }
                                    } else {
                                        instance
                                    }
                                } else {
                                    instance
                                }
                            } else {
                                instance
                            }
                        } else {
                            instance
                        };

                        stack.push(final_instance);
                    } else {
                        stack.push(CfmlValue::Null);
                    }
                }

                BytecodeOp::DefineFunction(func_idx) => {
                    let func_name = self.program.functions[func_idx].name.clone();
                    self.user_functions.insert(func_name.clone(), func_idx);
                    // Push function reference — encode func_idx in body for super dispatch
                    stack.push(CfmlValue::Function(cfml_common::dynamic::CfmlFunction {
                        name: func_name,
                        params: self.program.functions[func_idx].params.iter().map(|name| {
                            cfml_common::dynamic::CfmlParam {
                                name: name.clone(),
                                param_type: None,
                                default: None,
                                required: false,
                            }
                        }).collect(),
                        body: cfml_common::dynamic::CfmlClosureBody::Expression(
                            Box::new(CfmlValue::Int(func_idx as i64)),
                        ),
                        return_type: None,
                        access: cfml_common::dynamic::CfmlAccess::Public,
                    }));
                }

                BytecodeOp::Increment(name) => {
                    if let Some(val) = locals.get(&name) {
                        let new_val = match val {
                            CfmlValue::Int(i) => CfmlValue::Int(i + 1),
                            CfmlValue::Double(d) => CfmlValue::Double(d + 1.0),
                            _ => CfmlValue::Int(1),
                        };
                        locals.insert(name, new_val);
                    }
                }
                BytecodeOp::Decrement(name) => {
                    if let Some(val) = locals.get(&name) {
                        let new_val = match val {
                            CfmlValue::Int(i) => CfmlValue::Int(i - 1),
                            CfmlValue::Double(d) => CfmlValue::Double(d - 1.0),
                            _ => CfmlValue::Int(-1),
                        };
                        locals.insert(name, new_val);
                    }
                }

                // Exception handling
                BytecodeOp::TryStart(catch_ip) => {
                    self.try_stack.push(TryHandler {
                        catch_ip,
                        stack_depth: stack.len(),
                    });
                }
                BytecodeOp::TryEnd => {
                    self.try_stack.pop();
                }
                BytecodeOp::Throw => {
                    let error_val = stack.pop().unwrap_or(CfmlValue::String(
                        "Unknown error".to_string(),
                    ));
                    if let Some(handler) = self.try_stack.pop() {
                        // Unwind stack
                        while stack.len() > handler.stack_depth {
                            stack.pop();
                        }
                        stack.push(error_val);
                        ip = handler.catch_ip;
                    } else {
                        return Err(CfmlError::runtime(error_val.as_string()));
                    }
                }

                BytecodeOp::CallMethod(method_name, arg_count, write_back) => {
                    // Pop explicit args
                    let mut extra_args: Vec<CfmlValue> =
                        (0..arg_count).filter_map(|_| stack.pop()).collect::<Vec<_>>().into_iter().rev().collect();
                    // Pop the object (receiver)
                    let object = stack.pop().unwrap_or(CfmlValue::Null);

                    // Clear method_this_writeback before the call
                    self.method_this_writeback = None;

                    // Detect super calls: object is a __super struct (no __name key,
                    // but contains Function values). For super.method(), bind `this`
                    // to the actual child instance from the caller's locals.
                    let result = if let CfmlValue::Struct(ref s) = object {
                        if !s.contains_key("__name") && !s.is_empty()
                            && s.values().any(|v| matches!(v, CfmlValue::Function(_)))
                        {
                            // Super dispatch — find the parent's function by stored index
                            let prop = object.get(&method_name).unwrap_or(CfmlValue::Null);
                            if let CfmlValue::Function(ref f) = &prop {
                                // Extract stored bytecode index from function body
                                let func_idx = if let cfml_common::dynamic::CfmlClosureBody::Expression(ref body) = f.body {
                                    if let CfmlValue::Int(idx) = body.as_ref() {
                                        Some(*idx as usize)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                let args: Vec<CfmlValue> = extra_args.drain(..).collect();
                                let mut method_locals = HashMap::new();
                                // Use the actual child 'this' from caller's locals
                                if let Some(real_this) = locals.get("this") {
                                    method_locals.insert("this".to_string(), real_this.clone());
                                } else {
                                    method_locals.insert("this".to_string(), object.clone());
                                }
                                // Execute directly by index to avoid name collision
                                if let Some(idx) = func_idx {
                                    if idx < self.program.functions.len() {
                                        let parent_func = self.program.functions[idx].clone();
                                        self.execute_function_with_args(&parent_func, args, Some(&method_locals))?
                                    } else {
                                        self.call_function(&prop, args, &method_locals)?
                                    }
                                } else {
                                    self.call_function(&prop, args, &method_locals)?
                                }
                            } else {
                                self.call_member_function(&object, &method_name, &mut extra_args)?
                            }
                        } else {
                            self.call_member_function(&object, &method_name, &mut extra_args)?
                        }
                    } else {
                        self.call_member_function(&object, &method_name, &mut extra_args)?
                    };

                    // Write-back: emulate CFML pass-by-reference semantics for mutating methods.
                    // The compiler encodes where to write back based on the AST:
                    //   Some((var, Some(prop))) — e.g. this.items.append(x) → write result to var.prop
                    //   Some((var, None))       — e.g. arr.append(x) → write result to var
                    if let Some((var_name, prop_name)) = &write_back {
                        match prop_name {
                            Some(prop) => {
                                // Property access write-back: var.prop.method(args)
                                if Self::is_mutating_method(&method_name) {
                                    let parent = locals.get(var_name).cloned()
                                        .or_else(|| self.globals.get(var_name).cloned());
                                    if let Some(mut parent_obj) = parent {
                                        parent_obj.set(prop.clone(), result.clone());
                                        if locals.contains_key(var_name) {
                                            locals.insert(var_name.clone(), parent_obj);
                                        } else if self.globals.contains_key(var_name) {
                                            self.globals.insert(var_name.clone(), parent_obj);
                                        }
                                    }
                                }
                            }
                            None => {
                                // Direct variable write-back: var.method(args)
                                if Self::is_mutating_method(&method_name) {
                                    if locals.contains_key(var_name) {
                                        locals.insert(var_name.clone(), result.clone());
                                    } else if self.globals.contains_key(var_name) {
                                        self.globals.insert(var_name.clone(), result.clone());
                                    }
                                }
                            }
                        }
                    }

                    // Propagate component method `this` modifications back to caller.
                    // When a component method modifies `this` internally, the modified
                    // `this` is saved by execute_function_with_args. Write it back.
                    if let Some(modified_this) = self.method_this_writeback.take() {
                        if let Some((var_name, None)) = &write_back {
                            if locals.contains_key(var_name) {
                                locals.insert(var_name.clone(), modified_this);
                            } else if self.globals.contains_key(var_name) {
                                self.globals.insert(var_name.clone(), modified_this);
                            }
                        }
                    }

                    stack.push(result);
                }

                BytecodeOp::GetKeys => {
                    // For for-in: convert struct to array of keys, leave arrays unchanged
                    if let Some(val) = stack.pop() {
                        match val {
                            CfmlValue::Struct(s) => {
                                let keys: Vec<CfmlValue> = s.keys()
                                    .map(|k| CfmlValue::String(k.clone()))
                                    .collect();
                                stack.push(CfmlValue::Array(keys));
                            }
                            CfmlValue::String(s) => {
                                // Iterating over a string: convert to array of chars
                                let chars: Vec<CfmlValue> = s.chars()
                                    .map(|c| CfmlValue::String(c.to_string()))
                                    .collect();
                                stack.push(CfmlValue::Array(chars));
                            }
                            other => stack.push(other), // arrays pass through
                        }
                    }
                }

                BytecodeOp::IsNull => {
                    if let Some(val) = stack.pop() {
                        stack.push(CfmlValue::Bool(matches!(val, CfmlValue::Null)));
                    } else {
                        stack.push(CfmlValue::Bool(true));
                    }
                }

                BytecodeOp::JumpIfNotNull(target) => {
                    // Peek at the top of stack - if not null, jump (leave value on stack)
                    // If null, continue (leave null on stack)
                    if let Some(val) = stack.last() {
                        if !matches!(val, CfmlValue::Null) {
                            ip = target;
                        }
                    }
                }

                BytecodeOp::Include(path) => {
                    // Resolve path relative to source file or CWD
                    let resolved = if let Some(ref source) = self.source_file {
                        let source_dir = std::path::Path::new(source).parent()
                            .unwrap_or_else(|| std::path::Path::new("."));
                        source_dir.join(&path).to_string_lossy().to_string()
                    } else {
                        path.clone()
                    };

                    // Read, parse, compile, and execute the included file
                    match std::fs::read_to_string(&resolved) {
                        Ok(source_code) => {
                            // Check for CFML tags and preprocess
                            let source_code = if cfml_compiler::tag_parser::has_cfml_tags(&source_code) {
                                cfml_compiler::tag_parser::tags_to_script(&source_code)
                            } else {
                                source_code
                            };

                            let mut parser = cfml_compiler::parser::Parser::new(source_code);
                            match parser.parse() {
                                Ok(ast) => {
                                    let compiler = cfml_codegen::compiler::CfmlCompiler::new();
                                    let sub_program = compiler.compile(ast);
                                    // Execute the included program's main function
                                    // Copy current locals into the included program's execution
                                    let old_program = std::mem::replace(&mut self.program, sub_program);
                                    let main_idx = self.program.functions.iter()
                                        .position(|f| f.name == "__main__")
                                        .unwrap_or(0);
                                    let inc_func = self.program.functions[main_idx].clone();
                                    let _ = self.execute_function_with_args(&inc_func, Vec::new(), Some(&locals));
                                    self.program = old_program;
                                }
                                Err(e) => {
                                    return Err(CfmlError::runtime(
                                        format!("Include parse error in '{}': {}", resolved, e.message)
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(CfmlError::runtime(
                                format!("Include error: Cannot read '{}': {}", resolved, e)
                            ));
                        }
                    }
                }

                BytecodeOp::Print => {
                    if let Some(val) = stack.pop() {
                        self.output_buffer.push_str(&val.as_string());
                        self.output_buffer.push('\n');
                    }
                }
                BytecodeOp::Halt => break,
            }
        }

        // Save modified 'this' for component method write-back
        if let Some(this_val) = locals.get("this") {
            self.method_this_writeback = Some(this_val.clone());
        }
        Ok(stack.pop().unwrap_or(CfmlValue::Null))
    }

    fn call_function(
        &mut self,
        func_ref: &CfmlValue,
        args: Vec<CfmlValue>,
        parent_locals: &HashMap<String, CfmlValue>,
    ) -> CfmlResult {
        if let CfmlValue::Function(func) = func_ref {
            // Check builtin functions first (case-insensitive)
            let name_lower = func.name.to_lowercase();

            // writeOutput/writeDump must be handled before the builtin lookup
            // so output goes to output_buffer (not stdout via the builtin fn)
            if name_lower == "writeoutput" {
                for arg in &args {
                    self.output_buffer.push_str(&arg.as_string());
                }
                return Ok(CfmlValue::Null);
            }
            if name_lower == "writedump" || name_lower == "dump" {
                for arg in &args {
                    self.output_buffer.push_str(&format!("{:?}\n", arg));
                }
                return Ok(CfmlValue::Null);
            }

            // Higher-order functions must be handled BEFORE regular builtins
            // because they need VM access to invoke closures
            match name_lower.as_str() {
                "arraymap" | "arrayfilter" | "arrayreduce" | "arrayeach"
                | "arraysome" | "arrayevery"
                | "structeach" | "structmap" | "structfilter"
                | "structreduce" | "structsome" | "structevery"
                | "listeach" | "listmap" | "listfilter" | "listreduce"
                | "createobject"
                | "getcurrenttemplatepath"
                | "getcomponentmetadata"
                | "__cfheader" | "__cfcontent" | "__cflocation"
                | "gethttprequestdata" | "__cfinvoke"
                | "__cfsavecontent_start" | "__cfsavecontent_end" | "invoke"
                | "getbasetemplatepath" | "gettimezone" => {
                    // Will be handled at the end of this function (needs VM access)
                }
                _ => {
                    // Try exact match first, then case-insensitive
                    if let Some(builtin) = self.builtins.get(&func.name) {
                        return builtin(args);
                    }

                    // Case-insensitive builtin lookup
                    let builtin_match = self
                        .builtins
                        .iter()
                        .find(|(k, _)| k.to_lowercase() == name_lower)
                        .map(|(_, v)| *v);

                    if let Some(builtin) = builtin_match {
                        return builtin(args);
                    }
                }
            }

            // If the function has a stored bytecode index, use it directly
            // (avoids name collision when parent/child have same-named methods)
            if let cfml_common::dynamic::CfmlClosureBody::Expression(ref body) = func.body {
                if let CfmlValue::Int(idx) = body.as_ref() {
                    let idx = *idx as usize;
                    if idx < self.program.functions.len() {
                        let user_func = self.program.functions[idx].clone();
                        return self.execute_function_with_args(&user_func, args, Some(parent_locals));
                    }
                }
            }

            // Check user-defined functions by name
            if let Some(&func_idx) = self.user_functions.get(&func.name) {
                let user_func = self.program.functions[func_idx].clone();
                return self.execute_function_with_args(&user_func, args, Some(parent_locals));
            }

            // Case-insensitive user function lookup
            let user_match = self
                .user_functions
                .iter()
                .find(|(k, _)| k.to_lowercase() == name_lower)
                .map(|(_, v)| *v);

            if let Some(func_idx) = user_match {
                let user_func = self.program.functions[func_idx].clone();
                return self.execute_function_with_args(&user_func, args, Some(parent_locals));
            }

            // Higher-order standalone functions (arrayMap, arrayFilter, arrayReduce, etc.)
            match name_lower.as_str() {
                "arraymap" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let mut result = Vec::new();
                            let callback = callback.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                                result.push(mapped);
                            }
                            return Ok(CfmlValue::Array(result));
                        }
                    }
                    return Ok(CfmlValue::Array(Vec::new()));
                }
                "arrayfilter" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let mut result = Vec::new();
                            let callback = callback.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                let keep = self.call_function(&callback, cb_args, parent_locals)?;
                                if keep.is_true() {
                                    result.push(item.clone());
                                }
                            }
                            return Ok(CfmlValue::Array(result));
                        }
                    }
                    return Ok(CfmlValue::Array(Vec::new()));
                }
                "arrayreduce" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let mut acc = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                            let callback = callback.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![acc.clone(), item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                acc = self.call_function(&callback, cb_args, parent_locals)?;
                            }
                            return Ok(acc);
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "arrayeach" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let callback = callback.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                self.call_function(&callback, cb_args, parent_locals)?;
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "structeach" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let callback = callback.clone();
                            for (k, v) in s {
                                let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                self.call_function(&callback, cb_args, parent_locals)?;
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "structmap" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let mut result = HashMap::new();
                            let callback = callback.clone();
                            for (k, v) in s {
                                let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                                result.insert(k.clone(), mapped);
                            }
                            return Ok(CfmlValue::Struct(result));
                        }
                    }
                    return Ok(CfmlValue::Struct(HashMap::new()));
                }
                "structfilter" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let mut result = HashMap::new();
                            let callback = callback.clone();
                            for (k, v) in s {
                                let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                let keep = self.call_function(&callback, cb_args, parent_locals)?;
                                if keep.is_true() {
                                    result.insert(k.clone(), v.clone());
                                }
                            }
                            return Ok(CfmlValue::Struct(result));
                        }
                    }
                    return Ok(CfmlValue::Struct(HashMap::new()));
                }
                "arraysome" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let callback = callback.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if result.is_true() {
                                    return Ok(CfmlValue::Bool(true));
                                }
                            }
                            return Ok(CfmlValue::Bool(false));
                        }
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "arrayevery" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let callback = callback.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if !result.is_true() {
                                    return Ok(CfmlValue::Bool(false));
                                }
                            }
                            return Ok(CfmlValue::Bool(true));
                        }
                    }
                    return Ok(CfmlValue::Bool(true));
                }
                "structreduce" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let mut acc = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                            let callback = callback.clone();
                            for (k, v) in s {
                                let cb_args = vec![acc.clone(), CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                acc = self.call_function(&callback, cb_args, parent_locals)?;
                            }
                            return Ok(acc);
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "structsome" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let callback = callback.clone();
                            for (k, v) in s {
                                let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if result.is_true() {
                                    return Ok(CfmlValue::Bool(true));
                                }
                            }
                            return Ok(CfmlValue::Bool(false));
                        }
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "structevery" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let callback = callback.clone();
                            for (k, v) in s {
                                let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if !result.is_true() {
                                    return Ok(CfmlValue::Bool(false));
                                }
                            }
                            return Ok(CfmlValue::Bool(true));
                        }
                    }
                    return Ok(CfmlValue::Bool(true));
                }
                "listeach" => {
                    if let (Some(list_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let list = list_val.as_string();
                        let delimiter = args.get(2).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string());
                        let callback = callback.clone();
                        let items: Vec<&str> = list.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
                        for (i, item) in items.iter().enumerate() {
                            let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), list_val.clone()];
                            self.call_function(&callback, cb_args, parent_locals)?;
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "listmap" => {
                    if let (Some(list_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let list = list_val.as_string();
                        let delimiter = args.get(2).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string());
                        let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
                        let callback = callback.clone();
                        let items: Vec<&str> = list.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
                        let mut result = Vec::new();
                        for (i, item) in items.iter().enumerate() {
                            let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), list_val.clone()];
                            let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                            result.push(mapped.as_string());
                        }
                        return Ok(CfmlValue::String(result.join(&first_delim)));
                    }
                    return Ok(CfmlValue::String(String::new()));
                }
                "listfilter" => {
                    if let (Some(list_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let list = list_val.as_string();
                        let delimiter = args.get(2).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string());
                        let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
                        let callback = callback.clone();
                        let items: Vec<&str> = list.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
                        let mut result = Vec::new();
                        for (i, item) in items.iter().enumerate() {
                            let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), list_val.clone()];
                            let keep = self.call_function(&callback, cb_args, parent_locals)?;
                            if keep.is_true() {
                                result.push(item.to_string());
                            }
                        }
                        return Ok(CfmlValue::String(result.join(&first_delim)));
                    }
                    return Ok(CfmlValue::String(String::new()));
                }
                "listreduce" => {
                    if let (Some(list_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let list = list_val.as_string();
                        let mut acc = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                        let delimiter = args.get(3).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string());
                        let callback = callback.clone();
                        let items: Vec<&str> = list.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
                        for (i, item) in items.iter().enumerate() {
                            let cb_args = vec![acc.clone(), CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), list_val.clone()];
                            acc = self.call_function(&callback, cb_args, parent_locals)?;
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "getcurrenttemplatepath" => {
                    if let Some(ref source) = self.source_file {
                        let path = std::path::Path::new(source);
                        if let Ok(abs) = std::fs::canonicalize(path) {
                            return Ok(CfmlValue::String(abs.to_string_lossy().to_string()));
                        }
                        return Ok(CfmlValue::String(source.clone()));
                    }
                    // Fallback to CWD
                    if let Ok(cwd) = std::env::current_dir() {
                        return Ok(CfmlValue::String(cwd.to_string_lossy().to_string()));
                    }
                    return Ok(CfmlValue::String(String::new()));
                }
                "getbasetemplatepath" => {
                    if let Some(ref base) = self.base_template_path {
                        let path = std::path::Path::new(base);
                        if let Ok(abs) = std::fs::canonicalize(path) {
                            return Ok(CfmlValue::String(abs.to_string_lossy().to_string()));
                        }
                        return Ok(CfmlValue::String(base.clone()));
                    }
                    // Fall back to source_file
                    if let Some(ref source) = self.source_file {
                        let path = std::path::Path::new(source);
                        if let Ok(abs) = std::fs::canonicalize(path) {
                            return Ok(CfmlValue::String(abs.to_string_lossy().to_string()));
                        }
                        return Ok(CfmlValue::String(source.clone()));
                    }
                    return Ok(CfmlValue::String(String::new()));
                }
                "gettimezone" => {
                    // Return the system timezone name
                    // Try to get IANA timezone from environment variable first
                    if let Ok(tz) = std::env::var("TZ") {
                        if !tz.is_empty() {
                            return Ok(CfmlValue::String(tz));
                        }
                    }
                    // macOS/Linux: read /etc/localtime symlink target
                    #[cfg(unix)]
                    {
                        if let Ok(link) = std::fs::read_link("/etc/localtime") {
                            let link_str = link.to_string_lossy().to_string();
                            // Extract timezone from path like /usr/share/zoneinfo/America/New_York
                            if let Some(pos) = link_str.find("zoneinfo/") {
                                let tz = &link_str[pos + 9..];
                                return Ok(CfmlValue::String(tz.to_string()));
                            }
                        }
                    }
                    // Fallback: return UTC
                    return Ok(CfmlValue::String("UTC".to_string()));
                }
                "getcomponentmetadata" => {
                    if let Some(dot_path) = args.get(0) {
                        let comp_name = dot_path.as_string();
                        if let Some(template) = self.resolve_component_template(&comp_name, parent_locals) {
                            let resolved = self.resolve_inheritance(template, parent_locals);
                            if let CfmlValue::Struct(ref s) = resolved {
                                let mut meta = HashMap::new();
                                // Name
                                meta.insert("name".to_string(), s.get("__name").cloned().unwrap_or(CfmlValue::String(comp_name.clone())));
                                // Extends
                                if let Some(chain) = s.get("__extends_chain") {
                                    if let CfmlValue::Array(arr) = chain {
                                        if let Some(first) = arr.first() {
                                            meta.insert("extends".to_string(), first.clone());
                                        }
                                    }
                                }
                                // Functions array
                                let mut functions = Vec::new();
                                for (k, v) in s {
                                    if let CfmlValue::Function(f) = v {
                                        if !k.starts_with("__") {
                                            let mut func_meta = HashMap::new();
                                            func_meta.insert("name".to_string(), CfmlValue::String(k.clone()));
                                            func_meta.insert("access".to_string(), CfmlValue::String(format!("{:?}", f.access).to_lowercase()));
                                            if let Some(ref rt) = f.return_type {
                                                func_meta.insert("returntype".to_string(), CfmlValue::String(rt.clone()));
                                            }
                                            let params: Vec<CfmlValue> = f.params.iter().map(|p| CfmlValue::String(p.name.clone())).collect();
                                            func_meta.insert("parameters".to_string(), CfmlValue::Array(params));
                                            functions.push(CfmlValue::Struct(func_meta));
                                        }
                                    }
                                }
                                meta.insert("functions".to_string(), CfmlValue::Array(functions));
                                // Component metadata
                                if let Some(md) = s.get("__metadata") {
                                    meta.insert("metadata".to_string(), md.clone());
                                }
                                // Properties
                                if let Some(props) = s.get("__properties") {
                                    meta.insert("properties".to_string(), props.clone());
                                }
                                return Ok(CfmlValue::Struct(meta));
                            }
                            return Ok(resolved);
                        }
                    }
                    return Ok(CfmlValue::Struct(HashMap::new()));
                }
                "createobject" => {
                    if args.len() >= 2 {
                        let obj_type = args[0].as_string().to_lowercase();
                        if obj_type == "component" {
                            let comp_name = args[1].as_string();
                            if let Some(template) = self.resolve_component_template(&comp_name, parent_locals) {
                                let instance = self.resolve_inheritance(template, parent_locals);
                                return Ok(instance);
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cfheader" => {
                    if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        if let Some(code_val) = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "statuscode")
                            .map(|(_, v)| v.clone())
                        {
                            let code = match &code_val {
                                CfmlValue::Int(n) => *n as u16,
                                CfmlValue::String(s) => s.parse::<u16>().unwrap_or(200),
                                CfmlValue::Double(d) => *d as u16,
                                _ => 200,
                            };
                            let text = opts.iter()
                                .find(|(k, _)| k.to_lowercase() == "statustext")
                                .map(|(_, v)| v.as_string())
                                .unwrap_or_else(|| "OK".to_string());
                            self.response_status = Some((code, text));
                        } else if let Some(name_val) = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "name")
                            .map(|(_, v)| v.as_string())
                        {
                            let value = opts.iter()
                                .find(|(k, _)| k.to_lowercase() == "value")
                                .map(|(_, v)| v.as_string())
                                .unwrap_or_default();
                            self.response_headers.push((name_val, value));
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cfcontent" => {
                    if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        if let Some(reset_val) = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "reset")
                            .map(|(_, v)| v.clone())
                        {
                            if reset_val.is_true() {
                                self.output_buffer.clear();
                            }
                        }
                        if let Some(ct) = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "type")
                            .map(|(_, v)| v.as_string())
                        {
                            self.response_content_type = Some(ct);
                        }
                        if let Some(var_val) = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "variable")
                            .map(|(_, v)| v.clone())
                        {
                            self.response_body = Some(var_val);
                        }
                        if let Some(file_path) = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "file")
                            .map(|(_, v)| v.as_string())
                        {
                            if let Ok(contents) = std::fs::read_to_string(&file_path) {
                                self.response_body = Some(CfmlValue::String(contents));
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cflocation" => {
                    if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        let url = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "url")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_default();
                        let status_code = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "statuscode")
                            .map(|(_, v)| match v {
                                CfmlValue::Int(n) => *n as u16,
                                CfmlValue::String(s) => s.parse::<u16>().unwrap_or(302),
                                CfmlValue::Double(d) => *d as u16,
                                _ => 302,
                            })
                            .unwrap_or(302);
                        self.redirect_url = Some(url.clone());
                        self.response_headers.push(("Location".to_string(), url));
                        self.response_status = Some((status_code, "Found".to_string()));
                        return Err(CfmlError::new(
                            "__cflocation_redirect".to_string(),
                            CfmlErrorType::Custom("redirect".to_string()),
                        ));
                    }
                    return Ok(CfmlValue::Null);
                }
                "gethttprequestdata" => {
                    if let Some(ref data) = self.http_request_data {
                        return Ok(data.clone());
                    }
                    let mut empty = HashMap::new();
                    empty.insert("headers".to_string(), CfmlValue::Struct(HashMap::new()));
                    empty.insert("content".to_string(), CfmlValue::String(String::new()));
                    empty.insert("method".to_string(), CfmlValue::String(String::new()));
                    empty.insert("protocol".to_string(), CfmlValue::String(String::new()));
                    return Ok(CfmlValue::Struct(empty));
                }
                "__cfinvoke" => {
                    let comp_val = args.get(0).cloned().unwrap_or(CfmlValue::Null);
                    let method_name = args.get(1).map(|v| v.as_string()).unwrap_or_default();
                    let invoke_args = args.get(2).cloned().unwrap_or(CfmlValue::Null);

                    let component = match &comp_val {
                        CfmlValue::Struct(_) => comp_val.clone(),
                        CfmlValue::String(name) => {
                            if let Some(template) = self.resolve_component_template(name, parent_locals) {
                                self.resolve_inheritance(template, parent_locals)
                            } else {
                                return Err(CfmlError::runtime(format!("Component '{}' not found", name)));
                            }
                        }
                        _ => {
                            let name = comp_val.as_string();
                            if let Some(template) = self.resolve_component_template(&name, parent_locals) {
                                self.resolve_inheritance(template, parent_locals)
                            } else {
                                return Err(CfmlError::runtime(format!("Component '{}' not found", name)));
                            }
                        }
                    };

                    let method_lower = method_name.to_lowercase();
                    if let CfmlValue::Struct(ref comp_struct) = component {
                        let method_func = comp_struct.iter()
                            .find(|(k, _)| k.to_lowercase() == method_lower)
                            .map(|(_, v)| v.clone());

                        if let Some(func @ CfmlValue::Function(_)) = method_func {
                            let call_args = if let CfmlValue::Struct(ref arg_map) = invoke_args {
                                if arg_map.is_empty() {
                                    Vec::new()
                                } else if let CfmlValue::Function(ref f) = func {
                                    // Get param names from bytecode function if CfmlFunction.params is empty
                                    let param_names: Vec<String> = if f.params.is_empty() {
                                        // Look up actual bytecode function params via func_idx
                                        if let cfml_common::dynamic::CfmlClosureBody::Expression(ref body) = f.body {
                                            if let CfmlValue::Int(idx) = body.as_ref() {
                                                let idx = *idx as usize;
                                                if idx < self.program.functions.len() {
                                                    self.program.functions[idx].params.clone()
                                                } else {
                                                    Vec::new()
                                                }
                                            } else {
                                                Vec::new()
                                            }
                                        } else {
                                            Vec::new()
                                        }
                                    } else {
                                        f.params.iter().map(|p| p.name.clone()).collect()
                                    };

                                    let mut positional = Vec::new();
                                    for param_name in &param_names {
                                        let param_lower = param_name.to_lowercase();
                                        let val = arg_map.iter()
                                            .find(|(k, _)| k.to_lowercase() == param_lower)
                                            .map(|(_, v)| v.clone())
                                            .unwrap_or(CfmlValue::Null);
                                        positional.push(val);
                                    }
                                    positional
                                } else {
                                    Vec::new()
                                }
                            } else if matches!(invoke_args, CfmlValue::Null) {
                                Vec::new()
                            } else {
                                vec![invoke_args]
                            };

                            let mut method_locals = parent_locals.clone();
                            method_locals.insert("this".to_string(), component.clone());
                            return self.call_function(&func, call_args, &method_locals);
                        } else {
                            return Err(CfmlError::runtime(
                                format!("Method '{}' not found in component", method_name),
                            ));
                        }
                    }
                    return Err(CfmlError::runtime("Invalid component for cfinvoke".to_string()));
                }
                "__cfsavecontent_start" => {
                    self.saved_output_buffers.push(std::mem::take(&mut self.output_buffer));
                    return Ok(CfmlValue::Null);
                }
                "__cfsavecontent_end" => {
                    let captured = std::mem::take(&mut self.output_buffer);
                    self.output_buffer = self.saved_output_buffers.pop().unwrap_or_default();
                    return Ok(CfmlValue::String(captured));
                }
                "invoke" => {
                    // Same as __cfinvoke: invoke(component, "methodName", argStruct)
                    let comp_val = args.get(0).cloned().unwrap_or(CfmlValue::Null);
                    let method_name = args.get(1).map(|v| v.as_string()).unwrap_or_default();
                    let invoke_args = args.get(2).cloned().unwrap_or(CfmlValue::Null);

                    let component = match &comp_val {
                        CfmlValue::Struct(_) => comp_val.clone(),
                        CfmlValue::String(name) => {
                            if let Some(template) = self.resolve_component_template(name, parent_locals) {
                                self.resolve_inheritance(template, parent_locals)
                            } else {
                                return Err(CfmlError::runtime(format!("Component '{}' not found", name)));
                            }
                        }
                        _ => {
                            let name = comp_val.as_string();
                            if let Some(template) = self.resolve_component_template(&name, parent_locals) {
                                self.resolve_inheritance(template, parent_locals)
                            } else {
                                return Err(CfmlError::runtime(format!("Component '{}' not found", name)));
                            }
                        }
                    };

                    let method_lower = method_name.to_lowercase();
                    if let CfmlValue::Struct(ref comp_struct) = component {
                        let method_func = comp_struct.iter()
                            .find(|(k, _)| k.to_lowercase() == method_lower)
                            .map(|(_, v)| v.clone());

                        if let Some(func @ CfmlValue::Function(_)) = method_func {
                            let call_args = if let CfmlValue::Struct(ref arg_map) = invoke_args {
                                if arg_map.is_empty() {
                                    Vec::new()
                                } else if let CfmlValue::Function(ref f) = func {
                                    let param_names: Vec<String> = if f.params.is_empty() {
                                        if let cfml_common::dynamic::CfmlClosureBody::Expression(ref body) = f.body {
                                            if let CfmlValue::Int(idx) = body.as_ref() {
                                                let idx = *idx as usize;
                                                if idx < self.program.functions.len() {
                                                    self.program.functions[idx].params.clone()
                                                } else {
                                                    Vec::new()
                                                }
                                            } else {
                                                Vec::new()
                                            }
                                        } else {
                                            Vec::new()
                                        }
                                    } else {
                                        f.params.iter().map(|p| p.name.clone()).collect()
                                    };

                                    let mut positional = Vec::new();
                                    for param_name in &param_names {
                                        let param_lower = param_name.to_lowercase();
                                        let val = arg_map.iter()
                                            .find(|(k, _)| k.to_lowercase() == param_lower)
                                            .map(|(_, v)| v.clone())
                                            .unwrap_or(CfmlValue::Null);
                                        positional.push(val);
                                    }
                                    positional
                                } else {
                                    Vec::new()
                                }
                            } else if matches!(invoke_args, CfmlValue::Null) {
                                Vec::new()
                            } else {
                                vec![invoke_args]
                            };

                            let mut method_locals = parent_locals.clone();
                            method_locals.insert("this".to_string(), component.clone());
                            return self.call_function(&func, call_args, &method_locals);
                        } else {
                            return Err(CfmlError::runtime(
                                format!("Method '{}' not found on component", method_name)
                            ));
                        }
                    } else {
                        return Err(CfmlError::runtime("invoke() first argument must be a component or component name".into()));
                    }
                }
                _ => {}
            }
        }

        Ok(CfmlValue::Null)
    }

    /// Handle member function calls like "hello".ucase(), [1,2,3].len(), etc.
    /// CFML member functions are syntactic sugar for standalone function calls
    /// where the object becomes the first argument.
    /// Returns true if the method name is a mutating array/struct operation.
    /// These methods modify the receiver in-place in CFML (pass-by-reference semantics).
    fn is_mutating_method(method: &str) -> bool {
        let lower = method.to_lowercase();
        // Implicit property setters (setXxx) are mutating
        if lower.starts_with("set") && lower.len() > 3 {
            return true;
        }
        matches!(lower.as_str(),
            // Array mutators
            "append" | "push" | "prepend" | "deleteat" | "insertat" |
            "sort" | "reverse" | "clear" |
            // Struct mutators
            "delete" | "insert" | "update"
        )
    }

    fn call_member_function(
        &mut self,
        object: &CfmlValue,
        method: &str,
        extra_args: &mut Vec<CfmlValue>,
    ) -> CfmlResult {
        let method_lower = method.to_lowercase();

        // Map member function names to standalone builtin names
        // The object becomes the first argument
        let builtin_name = match object {
            CfmlValue::String(_) => match method_lower.as_str() {
                "len" | "length" => Some("len"),
                "ucase" | "touppercase" => Some("ucase"),
                "lcase" | "tolowercase" => Some("lcase"),
                "trim" => Some("trim"),
                "ltrim" => Some("ltrim"),
                "rtrim" => Some("rtrim"),
                "reverse" => Some("reverse"),
                "left" => Some("left"),
                "right" => Some("right"),
                "mid" => Some("mid"),
                "find" | "indexof" => Some("find"),
                "findnocase" => Some("findNoCase"),
                "replace" => Some("replace"),
                "replacenocase" => Some("replaceNoCase"),
                "contains" => {
                    // "hello".contains("ell") => find("ell", "hello") > 0
                    if let Some(needle) = extra_args.first() {
                        let haystack = object.as_string().to_lowercase();
                        let needle_str = needle.as_string().to_lowercase();
                        return Ok(CfmlValue::Bool(haystack.contains(&needle_str)));
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "insert" => Some("insert"),
                "removechars" => Some("removeChars"),
                "repeatstring" | "repeat" => Some("repeatString"),
                "compare" => Some("compare"),
                "comparenocase" => Some("compareNoCase"),
                "asc" => Some("asc"),
                "chr" => Some("chr"),
                "split" => Some("listToArray"),
                "listtoarray" => Some("listToArray"),
                "listlen" => Some("listLen"),
                "listfirst" => Some("listFirst"),
                "listlast" => Some("listLast"),
                "listrest" => Some("listRest"),
                "listgetat" => Some("listGetAt"),
                "listfind" => Some("listFind"),
                "listcontains" => Some("listContains"),
                "listappend" => Some("listAppend"),
                "refind" => Some("reFind"),
                "rereplace" => Some("reReplace"),
                "rematch" => Some("reMatch"),
                "wrap" => Some("wrap"),
                "tojson" | "serializejson" => Some("serializeJSON"),
                "tonumeric" | "val" => Some("val"),
                "toboolean" => Some("toBoolean"),
                _ => None,
            },
            CfmlValue::Array(arr) => match method_lower.as_str() {
                "len" | "length" | "size" => Some("arrayLen"),
                "append" | "push" => Some("arrayAppend"),
                "prepend" => Some("arrayPrepend"),
                "deleteat" => Some("arrayDeleteAt"),
                "insertat" => Some("arrayInsertAt"),
                "contains" => Some("arrayContains"),
                "containsnocase" => Some("arrayContainsNoCase"),
                "find" | "indexof" => Some("arrayFind"),
                "findnocase" => Some("arrayFindNoCase"),
                "sort" => Some("arraySort"),
                "reverse" => Some("arrayReverse"),
                "slice" => Some("arraySlice"),
                "tolist" => Some("arrayToList"),
                "merge" => Some("arrayMerge"),
                "clear" => Some("arrayClear"),
                "min" => Some("arrayMin"),
                "max" => Some("arrayMax"),
                "avg" => Some("arrayAvg"),
                "sum" => Some("arraySum"),
                "map" => {
                    // arr.map(callback) - callback(item, index, array)
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut result = Vec::new();
                        for (i, item) in arr.iter().enumerate() {
                            let cb_args = vec![
                                item.clone(),
                                CfmlValue::Int((i + 1) as i64),
                                object.clone(),
                            ];
                            let mapped = self.call_function(&callback, cb_args, &HashMap::new())?;
                            result.push(mapped);
                        }
                        return Ok(CfmlValue::Array(result));
                    }
                    return Ok(object.clone());
                }
                "filter" => {
                    // arr.filter(callback) - callback(item, index, array)
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut result = Vec::new();
                        for (i, item) in arr.iter().enumerate() {
                            let cb_args = vec![
                                item.clone(),
                                CfmlValue::Int((i + 1) as i64),
                                object.clone(),
                            ];
                            let keep = self.call_function(&callback, cb_args, &HashMap::new())?;
                            if keep.is_true() {
                                result.push(item.clone());
                            }
                        }
                        return Ok(CfmlValue::Array(result));
                    }
                    return Ok(object.clone());
                }
                "reduce" => {
                    // arr.reduce(callback, initialValue) - callback(accumulator, item, index, array)
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut acc = extra_args.get(1).cloned().unwrap_or(CfmlValue::Null);
                        for (i, item) in arr.iter().enumerate() {
                            let cb_args = vec![
                                acc.clone(),
                                item.clone(),
                                CfmlValue::Int((i + 1) as i64),
                                object.clone(),
                            ];
                            acc = self.call_function(&callback, cb_args, &HashMap::new())?;
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "each" => {
                    // arr.each(callback) - callback(item, index, array)
                    if let Some(callback) = extra_args.first().cloned() {
                        for (i, item) in arr.iter().enumerate() {
                            let cb_args = vec![
                                item.clone(),
                                CfmlValue::Int((i + 1) as i64),
                                object.clone(),
                            ];
                            self.call_function(&callback, cb_args, &HashMap::new())?;
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "some" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        for (i, item) in arr.iter().enumerate() {
                            let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), object.clone()];
                            let result = self.call_function(&callback, cb_args, &HashMap::new())?;
                            if result.is_true() { return Ok(CfmlValue::Bool(true)); }
                        }
                        return Ok(CfmlValue::Bool(false));
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "every" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        for (i, item) in arr.iter().enumerate() {
                            let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), object.clone()];
                            let result = self.call_function(&callback, cb_args, &HashMap::new())?;
                            if !result.is_true() { return Ok(CfmlValue::Bool(false)); }
                        }
                        return Ok(CfmlValue::Bool(true));
                    }
                    return Ok(CfmlValue::Bool(true));
                }
                "first" => {
                    return Ok(arr.first().cloned().unwrap_or(CfmlValue::Null));
                }
                "last" => {
                    return Ok(arr.last().cloned().unwrap_or(CfmlValue::Null));
                }
                "isempty" => {
                    return Ok(CfmlValue::Bool(arr.is_empty()));
                }
                "tojson" | "serializejson" => Some("serializeJSON"),
                _ => None,
            },
            CfmlValue::Struct(s) => match method_lower.as_str() {
                "count" | "len" | "size" => Some("structCount"),
                "keyexists" => Some("structKeyExists"),
                "keylist" => Some("structKeyList"),
                "keyarray" => Some("structKeyArray"),
                "delete" => Some("structDelete"),
                "insert" => Some("structInsert"),
                "update" => Some("structUpdate"),
                "find" => Some("structFind"),
                "findkey" => Some("structFindKey"),
                "findvalue" => Some("structFindValue"),
                "clear" => Some("structClear"),
                "copy" => Some("structCopy"),
                "append" => Some("structAppend"),
                "isempty" => Some("structIsEmpty"),
                "sort" => Some("structSort"),
                "each" => {
                    // struct.each(callback) - callback(key, value, struct)
                    if let Some(callback) = extra_args.first().cloned() {
                        for (k, v) in s {
                            let cb_args = vec![
                                CfmlValue::String(k.clone()),
                                v.clone(),
                                object.clone(),
                            ];
                            self.call_function(&callback, cb_args, &HashMap::new())?;
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "map" => {
                    // struct.map(callback) - callback(key, value, struct) returns new value
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut result = HashMap::new();
                        for (k, v) in s {
                            let cb_args = vec![
                                CfmlValue::String(k.clone()),
                                v.clone(),
                                object.clone(),
                            ];
                            let mapped = self.call_function(&callback, cb_args, &HashMap::new())?;
                            result.insert(k.clone(), mapped);
                        }
                        return Ok(CfmlValue::Struct(result));
                    }
                    return Ok(object.clone());
                }
                "filter" => {
                    // struct.filter(callback) - callback(key, value, struct) returns boolean
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut result = HashMap::new();
                        for (k, v) in s {
                            let cb_args = vec![
                                CfmlValue::String(k.clone()),
                                v.clone(),
                                object.clone(),
                            ];
                            let keep = self.call_function(&callback, cb_args, &HashMap::new())?;
                            if keep.is_true() {
                                result.insert(k.clone(), v.clone());
                            }
                        }
                        return Ok(CfmlValue::Struct(result));
                    }
                    return Ok(object.clone());
                }
                "some" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        for (k, v) in s {
                            let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), object.clone()];
                            let result = self.call_function(&callback, cb_args, &HashMap::new())?;
                            if result.is_true() { return Ok(CfmlValue::Bool(true)); }
                        }
                        return Ok(CfmlValue::Bool(false));
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "every" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        for (k, v) in s {
                            let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), object.clone()];
                            let result = self.call_function(&callback, cb_args, &HashMap::new())?;
                            if !result.is_true() { return Ok(CfmlValue::Bool(false)); }
                        }
                        return Ok(CfmlValue::Bool(true));
                    }
                    return Ok(CfmlValue::Bool(true));
                }
                "reduce" => {
                    if extra_args.len() >= 1 {
                        let callback = extra_args[0].clone();
                        let mut acc = if extra_args.len() >= 2 { extra_args[1].clone() } else { CfmlValue::Null };
                        for (k, v) in s {
                            let cb_args = vec![acc.clone(), CfmlValue::String(k.clone()), v.clone(), object.clone()];
                            acc = self.call_function(&callback, cb_args, &HashMap::new())?;
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "tojson" | "serializejson" => Some("serializeJSON"),
                _ => None,
            },
            CfmlValue::Int(_) | CfmlValue::Double(_) => match method_lower.as_str() {
                "tostring" => {
                    return Ok(CfmlValue::String(object.as_string()));
                }
                "abs" => Some("abs"),
                "ceiling" | "ceil" => Some("ceiling"),
                "floor" => Some("floor"),
                "round" => Some("round"),
                _ => None,
            },
            _ => None,
        };

        if let Some(name) = builtin_name {
            // Build args list: object as first arg, then extra args
            let mut args = vec![object.clone()];
            args.append(extra_args);

            // Look up the builtin (case-insensitive)
            let name_lower = name.to_lowercase();
            if let Some(builtin) = self.builtins.get(name) {
                return builtin(args);
            }
            // Case-insensitive fallback
            let builtin_match = self
                .builtins
                .iter()
                .find(|(k, _)| k.to_lowercase() == name_lower)
                .map(|(_, v)| *v);
            if let Some(builtin) = builtin_match {
                return builtin(args);
            }
        }

        // If no builtin match found, try to get property and call it
        // This handles user-defined methods on components
        let prop = if let CfmlValue::Struct(ref s) = object {
            let method_lower = method.to_lowercase();
            s.iter()
                .find(|(k, _)| k.to_lowercase() == method_lower)
                .map(|(_, v)| v.clone())
                .unwrap_or(CfmlValue::Null)
        } else {
            object.get(method).unwrap_or(CfmlValue::Null)
        };
        if let CfmlValue::Function(_) = &prop {
            let func_ref = prop.clone();
            let args: Vec<CfmlValue> = extra_args.drain(..).collect();
            // Bind 'this' to the object for component method calls
            let mut method_locals = HashMap::new();
            method_locals.insert("this".to_string(), object.clone());
            return self.call_function(&func_ref, args, &method_locals);
        }

        // Implicit property accessors (getXxx / setXxx) for components
        if let CfmlValue::Struct(ref s) = object {
            if s.contains_key("__name") || s.iter().any(|(k, _)| k.to_lowercase() == "__properties") {
                let method_lower = method.to_lowercase();
                if method_lower.starts_with("get") && method_lower.len() > 3 {
                    let prop_name = &method[3..];
                    let val = s.iter()
                        .find(|(k, _)| k.to_lowercase() == prop_name.to_lowercase())
                        .map(|(_, v)| v.clone());
                    if let Some(v) = val {
                        return Ok(v);
                    }
                }
                if method_lower.starts_with("set") && method_lower.len() > 3 {
                    let prop_name = &method[3..];
                    if let Some(value) = extra_args.first() {
                        let mut modified = object.clone();
                        if let CfmlValue::Struct(ref mut ms) = modified {
                            let actual_key = ms.keys()
                                .find(|k| k.to_lowercase() == prop_name.to_lowercase())
                                .cloned()
                                .unwrap_or_else(|| prop_name.to_string());
                            ms.insert(actual_key, value.clone());
                        }
                        return Ok(modified);
                    }
                }
            }
        }

        // onMissingMethod fallback for components
        if let CfmlValue::Struct(ref s) = object {
            let missing_handler = s.iter()
                .find(|(k, _)| k.to_lowercase() == "onmissingmethod")
                .map(|(_, v)| v.clone());
            if let Some(handler @ CfmlValue::Function(_)) = missing_handler {
                let args_array: Vec<CfmlValue> = extra_args.drain(..).collect();
                let mut missing_args = HashMap::new();
                for (i, a) in args_array.iter().enumerate() {
                    missing_args.insert((i + 1).to_string(), a.clone());
                }
                let mut method_locals = HashMap::new();
                method_locals.insert("this".to_string(), object.clone());
                return self.call_function(
                    &handler,
                    vec![CfmlValue::String(method.to_string()), CfmlValue::Struct(missing_args)],
                    &method_locals,
                );
            }
        }

        Ok(CfmlValue::Null)
    }

    /// Resolve a component template by name: tries locals, globals (exact + CI),
    /// then loads from a .cfc file on disk.
    fn resolve_component_template(
        &mut self,
        class_name: &str,
        locals: &HashMap<String, CfmlValue>,
    ) -> Option<CfmlValue> {
        // 1. Try locals
        if let Some(val) = locals.get(class_name) {
            if matches!(val, CfmlValue::Struct(_)) {
                return Some(val.clone());
            }
        }
        // 2. Try globals (exact)
        if let Some(val) = self.globals.get(class_name) {
            if matches!(val, CfmlValue::Struct(_)) {
                return Some(val.clone());
            }
        }
        // 3. Case-insensitive lookup in globals
        let lower = class_name.to_lowercase();
        if let Some(val) = self.globals.iter()
            .find(|(k, _)| k.to_lowercase() == lower)
            .map(|(_, v)| v.clone())
        {
            if matches!(val, CfmlValue::Struct(_)) {
                return Some(val);
            }
        }
        // 4. Try loading .cfc file (convert dots to path separators)
        let cfc_path = if let Some(ref source) = self.source_file {
            let source_dir = std::path::Path::new(source).parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let file_name = class_name.replace('.', std::path::MAIN_SEPARATOR_STR);
            source_dir.join(format!("{}.cfc", file_name)).to_string_lossy().to_string()
        } else {
            format!("{}.cfc", class_name.replace('.', std::path::MAIN_SEPARATOR_STR))
        };

        if let Ok(source_code) = std::fs::read_to_string(&cfc_path) {
            let source_code = if cfml_compiler::tag_parser::has_cfml_tags(&source_code) {
                cfml_compiler::tag_parser::tags_to_script(&source_code)
            } else {
                source_code
            };
            if let Ok(ast) = cfml_compiler::parser::Parser::new(source_code).parse() {
                let compiler = cfml_codegen::compiler::CfmlCompiler::new();
                let sub_program = compiler.compile(ast);
                let old_program = std::mem::replace(&mut self.program, sub_program);
                let main_idx = self.program.functions.iter()
                    .position(|f| f.name == "__main__")
                    .unwrap_or(0);
                let cfc_func = self.program.functions[main_idx].clone();
                let _ = self.execute_function_with_args(&cfc_func, Vec::new(), Some(locals));
                // Merge sub-program functions
                let sub_funcs = self.program.functions.clone();
                self.program = old_program;
                let _base_idx = self.program.functions.len();
                for func in sub_funcs {
                    if func.name != "__main__" {
                        self.program.functions.push(func.clone());
                        if let Some(_old_idx) = self.user_functions.get(&func.name).cloned() {
                            self.user_functions.insert(func.name.clone(), self.program.functions.len() - 1);
                        }
                    }
                }
                // Look up the result
                let short_name = class_name.split('.').last().unwrap_or(class_name);
                return self.globals.get(class_name).cloned()
                    .or_else(|| self.globals.get(short_name).cloned())
                    .or_else(|| {
                        let lower = class_name.to_lowercase();
                        self.globals.iter()
                            .find(|(k, _)| k.to_lowercase() == lower)
                            .map(|(_, v)| v.clone())
                    })
                    .or_else(|| self.globals.get("Anonymous").cloned());
            }
        }
        None
    }

    /// Resolve the full inheritance chain for a component template.
    /// If the template has an `__extends` key, load the parent, recursively
    /// resolve its inheritance, then merge child on top of parent.
    fn resolve_inheritance(
        &mut self,
        template: CfmlValue,
        locals: &HashMap<String, CfmlValue>,
    ) -> CfmlValue {
        let s = match &template {
            CfmlValue::Struct(s) => s,
            _ => return template,
        };

        // Check for __extends key
        let extends_name = match s.get("__extends") {
            Some(CfmlValue::String(name)) => name.clone(),
            _ => return template, // No extends, return as-is
        };

        // Prevent circular inheritance
        let mut visited = std::collections::HashSet::new();
        if let Some(CfmlValue::String(name)) = s.get("__name") {
            visited.insert(name.to_lowercase());
        }

        self.resolve_inheritance_chain(template, &extends_name, locals, &mut visited)
    }

    fn resolve_inheritance_chain(
        &mut self,
        child: CfmlValue,
        parent_name: &str,
        locals: &HashMap<String, CfmlValue>,
        visited: &mut std::collections::HashSet<String>,
    ) -> CfmlValue {
        // Check circular
        if visited.contains(&parent_name.to_lowercase()) {
            return child;
        }
        visited.insert(parent_name.to_lowercase());

        // Resolve parent template
        let parent = match self.resolve_component_template(parent_name, locals) {
            Some(p) => p,
            None => return child, // Parent not found, return child as-is
        };

        // Recursively resolve parent's inheritance
        let parent = if let CfmlValue::Struct(ref ps) = parent {
            if let Some(CfmlValue::String(grandparent)) = ps.get("__extends") {
                let gp = grandparent.clone();
                self.resolve_inheritance_chain(parent, &gp, locals, visited)
            } else {
                parent
            }
        } else {
            parent
        };

        // Now merge: start with parent, layer child on top
        let child_map = match child {
            CfmlValue::Struct(s) => s,
            _ => return parent,
        };
        let mut parent_map = match parent {
            CfmlValue::Struct(s) => s,
            _ => return CfmlValue::Struct(child_map),
        };

        // Collect parent methods for __super
        let mut super_methods = HashMap::new();
        for (k, v) in &parent_map {
            if matches!(v, CfmlValue::Function(_)) && !k.starts_with("__") {
                super_methods.insert(k.clone(), v.clone());
            }
        }

        // Layer child on top of parent (child overrides parent)
        for (k, v) in &child_map {
            if k == "__extends" {
                continue; // Don't copy __extends to merged result
            }
            parent_map.insert(k.clone(), v.clone());
        }

        // Add __super struct
        if !super_methods.is_empty() {
            parent_map.insert("__super".to_string(), CfmlValue::Struct(super_methods));
        }

        // Build __extends_chain for isInstanceOf
        let mut chain = Vec::new();
        chain.push(CfmlValue::String(parent_name.to_string()));
        if let Some(CfmlValue::Array(existing)) = parent_map.get("__extends_chain") {
            for item in existing {
                chain.push(item.clone());
            }
        }
        parent_map.insert("__extends_chain".to_string(), CfmlValue::Array(chain));

        CfmlValue::Struct(parent_map)
    }

    /// Walk up the directory tree from source_file to find Application.cfc
    fn find_application_cfc(&self) -> Option<String> {
        let start_dir = if let Some(ref source) = self.source_file {
            std::path::Path::new(source).parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        } else {
            std::env::current_dir().unwrap_or_default()
        };

        let mut dir = start_dir.as_path();
        loop {
            // Check for Application.cfc (case-insensitive)
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.to_lowercase() == "application.cfc" {
                            return Some(entry.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
            match dir.parent() {
                Some(parent) if parent != dir => dir = parent,
                _ => break,
            }
        }
        None
    }

    /// Load and execute Application.cfc, returning the component struct
    fn load_application_cfc(&mut self, path: &str) -> Option<CfmlValue> {
        let source_code = std::fs::read_to_string(path).ok()?;
        let source_code = if cfml_compiler::tag_parser::has_cfml_tags(&source_code) {
            cfml_compiler::tag_parser::tags_to_script(&source_code)
        } else {
            source_code
        };

        let ast = cfml_compiler::parser::Parser::new(source_code).parse().ok()?;
        let compiler = cfml_codegen::compiler::CfmlCompiler::new();
        let sub_program = compiler.compile(ast);

        // Save current program, swap in sub-program
        let old_program = std::mem::replace(&mut self.program, sub_program);
        let main_idx = self.program.functions.iter()
            .position(|f| f.name == "__main__")
            .unwrap_or(0);
        let cfc_func = self.program.functions[main_idx].clone();
        let empty_locals = HashMap::new();
        let _ = self.execute_function_with_args(&cfc_func, Vec::new(), Some(&empty_locals));

        // Merge sub-program functions into main program
        let sub_funcs = self.program.functions.clone();
        self.program = old_program;
        for func in sub_funcs {
            if func.name != "__main__" {
                self.program.functions.push(func.clone());
                self.user_functions.insert(func.name.clone(), self.program.functions.len() - 1);
            }
        }

        // Find the component struct in globals
        self.globals.iter()
            .find(|(k, v)| {
                let k_lower = k.to_lowercase();
                (k_lower == "application" || *k == "Anonymous")
                    && matches!(v, CfmlValue::Struct(_))
                    && if let CfmlValue::Struct(s) = v { s.contains_key("__name") || s.values().any(|v| matches!(v, CfmlValue::Function(_))) } else { false }
            })
            .map(|(_, v)| v.clone())
            .or_else(|| {
                // Look for any struct with component-like structure
                self.globals.iter()
                    .find(|(_, v)| {
                        if let CfmlValue::Struct(s) = v {
                            s.contains_key("__name") || s.values().any(|val| matches!(val, CfmlValue::Function(_)))
                        } else {
                            false
                        }
                    })
                    .map(|(_, v)| v.clone())
            })
    }

    /// Extract application config from a component struct
    fn extract_app_config(template: &CfmlValue) -> (String, HashMap<String, CfmlValue>) {
        let s = match template {
            CfmlValue::Struct(s) => s,
            _ => return ("default".to_string(), HashMap::new()),
        };

        // Case-insensitive lookup for this.name
        let app_name = s.iter()
            .find(|(k, _)| k.to_lowercase() == "name")
            .and_then(|(_, v)| match v {
                CfmlValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "default".to_string());

        let mut config = HashMap::new();
        for (k, v) in s {
            if !k.starts_with("__") && !matches!(v, CfmlValue::Function(_)) {
                config.insert(k.to_lowercase(), v.clone());
            }
        }

        (app_name, config)
    }

    /// Call a lifecycle method on the Application.cfc template
    fn call_lifecycle_method(
        &mut self,
        template: &CfmlValue,
        method: &str,
        args: Vec<CfmlValue>,
    ) -> Result<bool, CfmlError> {
        let s = match template {
            CfmlValue::Struct(s) => s,
            _ => return Ok(false),
        };

        // Case-insensitive lookup for the method
        let method_lower = method.to_lowercase();
        let func_val = s.iter()
            .find(|(k, _)| k.to_lowercase() == method_lower)
            .map(|(_, v)| v.clone());

        match func_val {
            Some(ref func @ CfmlValue::Function(_)) => {
                // Bind `this` to the template
                let mut parent_locals = HashMap::new();
                parent_locals.insert("this".to_string(), template.clone());
                self.call_function(func, args, &parent_locals)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Execute with Application.cfc lifecycle
    pub fn execute_with_lifecycle(&mut self) -> CfmlResult {
        // 1. Find Application.cfc
        let app_cfc_path = self.find_application_cfc();

        let app_cfc_path = match app_cfc_path {
            Some(path) => path,
            None => return self.execute(), // No Application.cfc, just execute directly
        };

        // 2. Load Application.cfc
        let template = match self.load_application_cfc(&app_cfc_path) {
            Some(t) => t,
            None => return self.execute(), // Failed to load, fall through
        };

        // 3. Extract config
        let (app_name, _config) = Self::extract_app_config(&template);

        // 4. Wire up application scope
        if let Some(ref server_state) = self.server_state.clone() {
            let mut apps = server_state.applications.lock().unwrap();
            if !apps.contains_key(&app_name) {
                // New application
                let app_state = ApplicationState {
                    name: app_name.clone(),
                    variables: HashMap::new(),
                    started: false,
                    config: _config.clone(),
                };
                apps.insert(app_name.clone(), app_state);
            }
            let app = apps.get_mut(&app_name).unwrap();
            let scope = Arc::new(Mutex::new(app.variables.clone()));
            self.application_scope = Some(scope.clone());

            // 5. onApplicationStart (if not yet started)
            if !app.started {
                app.started = true;
                drop(apps); // Release lock before calling lifecycle method
                if let Err(e) = self.call_lifecycle_method(&template, "onApplicationStart", vec![]) {
                    let _ = self.call_lifecycle_method(&template, "onError", vec![
                        CfmlValue::String(e.message.clone()),
                        CfmlValue::String("onApplicationStart".to_string()),
                    ]);
                    return Err(e);
                }
            } else {
                drop(apps);
            }
        } else {
            // CLI mode: fresh application scope each time
            let scope = Arc::new(Mutex::new(HashMap::new()));
            self.application_scope = Some(scope);

            // Still call onApplicationStart in CLI mode
            let _ = self.call_lifecycle_method(&template, "onApplicationStart", vec![]);
        }

        // 6. onRequestStart
        let target_page = self.source_file.clone().unwrap_or_default();
        let _ = self.call_lifecycle_method(
            &template,
            "onRequestStart",
            vec![CfmlValue::String(target_page.clone())],
        );

        // 7. Check for onRequest — if exists, call it; else execute normally
        let has_on_request = if let CfmlValue::Struct(ref s) = template {
            s.iter().any(|(k, v)| k.to_lowercase() == "onrequest" && matches!(v, CfmlValue::Function(_)))
        } else {
            false
        };

        let result = if has_on_request {
            match self.call_lifecycle_method(
                &template,
                "onRequest",
                vec![CfmlValue::String(target_page.clone())],
            ) {
                Ok(_) => Ok(CfmlValue::Null),
                Err(e) if e.message == "__cflocation_redirect" => Ok(CfmlValue::Null),
                Err(e) => Err(e),
            }
        } else {
            match self.execute() {
                Ok(v) => Ok(v),
                Err(e) if e.message == "__cflocation_redirect" => Ok(CfmlValue::Null),
                Err(e) => Err(e),
            }
        };

        // 8. onRequestEnd
        let _ = self.call_lifecycle_method(
            &template,
            "onRequestEnd",
            vec![CfmlValue::String(target_page)],
        );

        // 9. Write application scope back to ServerState
        if let Some(ref server_state) = self.server_state.clone() {
            if let Some(ref app_scope) = self.application_scope {
                if let Ok(scope) = app_scope.lock() {
                    if let Ok(mut apps) = server_state.applications.lock() {
                        if let Some(app) = apps.get_mut(&app_name) {
                            app.variables = scope.clone();
                        }
                    }
                }
            }
        }

        // 10. Clear request scope
        self.request_scope.clear();

        result
    }

    pub fn get_output(&self) -> String {
        self.output_buffer.clone()
    }
}

// ---- Helper functions ----

fn binary_op<F>(stack: &mut Vec<CfmlValue>, op: F)
where
    F: FnOnce(CfmlValue, CfmlValue) -> CfmlValue,
{
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(op(a, b));
    }
}

fn compare_op<F>(stack: &mut Vec<CfmlValue>, op: F)
where
    F: FnOnce(&CfmlValue, &CfmlValue) -> bool,
{
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(CfmlValue::Bool(op(&a, &b)));
    }
}

fn to_number(val: &CfmlValue) -> Option<f64> {
    match val {
        CfmlValue::Int(i) => Some(*i as f64),
        CfmlValue::Double(d) => Some(*d),
        CfmlValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        CfmlValue::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn numeric_op<F>(a: &CfmlValue, b: &CfmlValue, op: F) -> CfmlValue
where
    F: FnOnce(f64, f64) -> f64,
{
    match (a, b) {
        (CfmlValue::Int(i), CfmlValue::Int(j)) => {
            // Try integer arithmetic first
            let fi = *i as f64;
            let fj = *j as f64;
            let result = op(fi, fj);
            if result == (result as i64 as f64) && result.abs() < i64::MAX as f64 {
                CfmlValue::Int(result as i64)
            } else {
                CfmlValue::Double(result)
            }
        }
        _ => {
            let x = to_number(a).unwrap_or(0.0);
            let y = to_number(b).unwrap_or(0.0);
            CfmlValue::Double(op(x, y))
        }
    }
}

/// CFML equality comparison (case-insensitive for strings, type-coercing for numbers)
fn cfml_equal(a: &CfmlValue, b: &CfmlValue) -> bool {
    match (a, b) {
        (CfmlValue::Null, CfmlValue::Null) => true,
        (CfmlValue::Null, _) | (_, CfmlValue::Null) => false,
        (CfmlValue::Bool(x), CfmlValue::Bool(y)) => x == y,
        (CfmlValue::Int(x), CfmlValue::Int(y)) => x == y,
        (CfmlValue::Double(x), CfmlValue::Double(y)) => x == y,
        (CfmlValue::Int(x), CfmlValue::Double(y)) => (*x as f64) == *y,
        (CfmlValue::Double(x), CfmlValue::Int(y)) => *x == (*y as f64),
        (CfmlValue::String(x), CfmlValue::String(y)) => x.eq_ignore_ascii_case(y),
        // String-number comparison: try to coerce
        (CfmlValue::String(s), CfmlValue::Int(i)) | (CfmlValue::Int(i), CfmlValue::String(s)) => {
            s.trim().parse::<i64>().map_or(false, |n| n == *i)
        }
        (CfmlValue::String(s), CfmlValue::Double(d))
        | (CfmlValue::Double(d), CfmlValue::String(s)) => {
            s.trim().parse::<f64>().map_or(false, |n| n == *d)
        }
        (CfmlValue::String(s), CfmlValue::Bool(b))
        | (CfmlValue::Bool(b), CfmlValue::String(s)) => {
            match s.to_lowercase().as_str() {
                "true" | "yes" => *b,
                "false" | "no" => !*b,
                _ => false,
            }
        }
        _ => false,
    }
}

/// CFML comparison ordering
fn cfml_compare(a: &CfmlValue, b: &CfmlValue) -> i32 {
    match (a, b) {
        (CfmlValue::Int(x), CfmlValue::Int(y)) => x.cmp(y) as i32,
        (CfmlValue::Double(x), CfmlValue::Double(y)) => {
            x.partial_cmp(y).map_or(0, |o| o as i32)
        }
        (CfmlValue::Int(x), CfmlValue::Double(y)) => {
            (*x as f64).partial_cmp(y).map_or(0, |o| o as i32)
        }
        (CfmlValue::Double(x), CfmlValue::Int(y)) => {
            x.partial_cmp(&(*y as f64)).map_or(0, |o| o as i32)
        }
        (CfmlValue::String(x), CfmlValue::String(y)) => {
            // Try numeric comparison first
            if let (Ok(a), Ok(b)) = (x.parse::<f64>(), y.parse::<f64>()) {
                return a.partial_cmp(&b).map_or(0, |o| o as i32);
            }
            x.to_lowercase().cmp(&y.to_lowercase()) as i32
        }
        _ => {
            let x = to_number(a).unwrap_or(0.0);
            let y = to_number(b).unwrap_or(0.0);
            x.partial_cmp(&y).map_or(0, |o| o as i32)
        }
    }
}
