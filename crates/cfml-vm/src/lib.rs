//! CFML Virtual Machine - Bytecode execution engine

use cfml_codegen::{BytecodeFunction, BytecodeOp, BytecodeProgram};
use cfml_common::dynamic::CfmlValue;
use cfml_common::vm::{CfmlError, CfmlErrorType, CfmlResult};
use std::collections::HashMap;
use indexmap::IndexMap;
use std::sync::{Arc, Mutex, RwLock};

pub type BuiltinFunction = fn(Vec<CfmlValue>) -> CfmlResult;

/// Persistent application state, keyed by app name.
pub struct ApplicationState {
    pub name: String,
    pub variables: IndexMap<String, CfmlValue>,
    pub started: bool,
    pub config: IndexMap<String, CfmlValue>,
    /// Bytecode functions added during onApplicationStart (factory, resources, etc.).
    /// Only the delta (functions added after load_application_cfc) is cached.
    pub cached_functions: Vec<cfml_codegen::compiler::BytecodeFunction>,
    /// The program.functions.len() at which cached_functions were originally inserted.
    /// Used to compute index offsets when restoring on subsequent requests.
    pub cached_functions_original_offset: usize,
}

/// A CFML component mapping: virtual prefix → physical directory.
#[derive(Debug, Clone)]
pub struct CfmlMapping {
    pub name: String, // Normalized: leading+trailing "/" e.g. "/taffy/"
    pub path: String, // Absolute filesystem directory
}

/// Session data for a single user session.
#[derive(Clone)]
pub struct SessionData {
    pub variables: IndexMap<String, CfmlValue>,
    pub created: std::time::Instant,
    pub last_accessed: std::time::Instant,
    pub auth_user: Option<String>,
    pub auth_roles: Vec<String>,
    /// Session timeout in seconds (default 1800 = 30 minutes)
    pub timeout_secs: u64,
}

/// Server-level state, persists across requests in --serve mode.
#[derive(Clone)]
pub struct ServerState {
    pub applications: Arc<Mutex<HashMap<String, ApplicationState>>>,
    pub sessions: Arc<Mutex<HashMap<String, SessionData>>>,
    /// Named locks for cflock: name → RwLock (exclusive = write, readonly = read)
    pub named_locks: Arc<Mutex<HashMap<String, Arc<RwLock<()>>>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            applications: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            named_locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// A held lock guard for cflock (keeps the lock alive during the block).
/// The guard fields are never read directly — they are held for their Drop behavior.
#[allow(dead_code)]
enum HeldLock {
    Write(std::sync::RwLockWriteGuard<'static, ()>),
    Read(std::sync::RwLockReadGuard<'static, ()>),
}

pub struct CfmlVirtualMachine {
    pub program: BytecodeProgram,
    pub globals: IndexMap<String, CfmlValue>,
    pub builtins: HashMap<String, BuiltinFunction>,
    pub output_buffer: String,
    /// User-defined functions (name -> function definition)
    pub user_functions: HashMap<String, BytecodeFunction>,
    /// Source file path (for include resolution)
    pub source_file: Option<String>,
    /// Call stack for tracking execution
    call_stack: Vec<CallFrame>,
    /// Try-catch handler stack
    try_stack: Vec<TryHandler>,
    /// Current exception (if any)
    #[allow(dead_code)]
    current_exception: Option<CfmlValue>,
    /// Last thrown exception (for rethrow support)
    last_exception: Option<CfmlValue>,
    /// Current source line being executed (updated by LineInfo op)
    current_line: usize,
    /// Current source column
    current_column: usize,
    /// After a component method executes, holds the modified `this` for write-back
    /// to the caller's object variable. Set by execute_function_with_args.
    method_this_writeback: Option<CfmlValue>,
    /// After a closure executes, holds modified parent-scope variables for write-back
    /// to the caller's locals. Enables closures to mutate parent scope.
    closure_parent_writeback: Option<IndexMap<String, CfmlValue>>,
    /// Request scope — lives for the duration of one request
    pub request_scope: IndexMap<String, CfmlValue>,
    /// Application scope — shared across requests (Arc<Mutex> for thread safety)
    pub application_scope: Option<Arc<Mutex<IndexMap<String, CfmlValue>>>>,
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
    /// Component mappings: virtual prefix → physical directory (sorted longest-first)
    pub mappings: Vec<CfmlMapping>,
    /// Captured locals from most recent execute_function_with_args call
    /// Used to capture component body variables (variables scope) after component loading
    captured_locals: Option<IndexMap<String, CfmlValue>>,
    /// Active transaction connection (held during cftransaction block, type-erased)
    pub transaction_conn: Option<Box<dyn std::any::Any>>,
    /// Datasource URL of the active transaction
    pub transaction_datasource: Option<String>,
    /// Function pointer: begin transaction (datasource) -> conn
    pub txn_begin: Option<fn(&str) -> Result<Box<dyn std::any::Any>, CfmlError>>,
    /// Function pointer: commit transaction (conn)
    pub txn_commit: Option<fn(&mut Box<dyn std::any::Any>) -> Result<(), CfmlError>>,
    /// Function pointer: rollback transaction (conn)
    pub txn_rollback: Option<fn(&mut Box<dyn std::any::Any>) -> Result<(), CfmlError>>,
    /// Function pointer: execute query with transaction conn (conn, sql, params, return_type) -> result
    pub txn_execute: Option<fn(&mut Box<dyn std::any::Any>, &str, &CfmlValue, &str) -> CfmlResult>,
    /// Function pointer: execute query normally (args) -> result
    pub query_execute_fn: Option<fn(Vec<CfmlValue>) -> CfmlResult>,
    /// Session ID for current request
    pub session_id: Option<String>,
    /// Stack of held cflock guards (name, guard)
    held_locks: Vec<(String, HeldLock)>,
    /// Custom tag paths from this.customTagPaths in Application.cfc
    pub custom_tag_paths: Vec<String>,
    /// Stack for nested body-mode custom tags
    custom_tag_stack: Vec<CustomTagState>,
    /// In-memory cache: key -> (value, optional expiry instant)
    pub cache: HashMap<String, (CfmlValue, Option<std::time::Instant>)>,
}

#[derive(Debug, Clone)]
struct CallFrame {
    function_name: String,
    template: String,
    /// Current line within this function (updated by LineInfo)
    line: usize,
    /// Line in the caller where this function was invoked
    caller_line: usize,
}

#[derive(Debug, Clone)]
struct TryHandler {
    catch_ip: usize,
    stack_depth: usize,
}

/// State for a body-mode custom tag execution
#[derive(Debug, Clone)]
struct CustomTagState {
    template_path: String,
    attributes: CfmlValue,
}

impl CfmlVirtualMachine {
    pub fn new(program: BytecodeProgram) -> Self {
        Self {
            program,
            globals: IndexMap::new(),
            builtins: HashMap::new(),
            output_buffer: String::new(),
            user_functions: HashMap::new(),
            source_file: None,
            call_stack: Vec::new(),
            try_stack: Vec::new(),
            current_exception: None,
            last_exception: None,
            current_line: 0,
            current_column: 0,
            method_this_writeback: None,
            closure_parent_writeback: None,
            request_scope: IndexMap::new(),
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
            mappings: Vec::new(),
            captured_locals: None,
            transaction_conn: None,
            transaction_datasource: None,
            txn_begin: None,
            txn_commit: None,
            txn_rollback: None,
            txn_execute: None,
            session_id: None,
            query_execute_fn: None,
            held_locks: Vec::new(),
            custom_tag_paths: Vec::new(),
            custom_tag_stack: Vec::new(),
            cache: HashMap::new(),
        }
    }

    fn build_stack_trace(&self) -> Vec<cfml_common::vm::StackFrame> {
        use cfml_common::vm::StackFrame;
        let mut frames = Vec::new();
        let template = self.source_file.clone().unwrap_or_default();

        if self.call_stack.is_empty() {
            // Error in __main__ — single frame
            frames.push(StackFrame {
                function: "__main__".to_string(),
                template,
                line: self.current_line,
            });
        } else {
            // Innermost frame: the function currently executing, at the current line
            frames.push(StackFrame {
                function: self.call_stack.last().unwrap().function_name.clone(),
                template: template.clone(),
                line: self.current_line,
            });
            // Intermediate frames in reverse (skip the last/current)
            for frame in self.call_stack.iter().rev().skip(1) {
                frames.push(StackFrame {
                    function: frame.function_name.clone(),
                    template: frame.template.clone(),
                    line: frame.line,
                });
            }
            // Root frame: __main__ at the line where the outermost function was called
            frames.push(StackFrame {
                function: "__main__".to_string(),
                template,
                line: self.call_stack.first().unwrap().caller_line,
            });
        }
        frames
    }

    fn build_tag_context(&self) -> CfmlValue {
        let frames = self.build_stack_trace();
        let context: Vec<CfmlValue> = frames.iter().map(|f| {
            let mut entry = IndexMap::new();
            entry.insert("template".to_string(), CfmlValue::String(f.template.clone()));
            entry.insert("line".to_string(), CfmlValue::Int(f.line as i64));
            entry.insert("id".to_string(), CfmlValue::String("CFML".to_string()));
            entry.insert("raw_trace".to_string(), CfmlValue::String(
                format!("at {}({}:{})", f.function, f.template, f.line)
            ));
            entry.insert("column".to_string(), CfmlValue::Int(0));
            CfmlValue::Struct(entry)
        }).collect();
        CfmlValue::Array(context)
    }

    fn wrap_error(&self, mut err: CfmlError) -> CfmlError {
        if err.stack_trace.is_empty() {
            err.stack_trace = self.build_stack_trace();
        }
        err
    }

    pub fn execute(&mut self) -> CfmlResult {
        let main_idx = self
            .program
            .functions
            .iter()
            .position(|f| f.name == "__main__")
            .ok_or_else(|| CfmlError::runtime("No main function found".to_string()))?;

        self.execute_function_by_index(main_idx, Vec::new())
            .map_err(|e| self.wrap_error(e))
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
        parent_scope: Option<&IndexMap<String, CfmlValue>>,
    ) -> CfmlResult {
        // Guard against runaway recursion — checked before allocating locals
        // to avoid blowing the native Rust stack.
        //
        // Strategy:
        //  1. Hard ceiling at 2500 (matches CFML engine defaults)
        //  2. Early infinite-recursion detection: once depth > 64, check if
        //     the last 32 frames show a repeating cycle (covers both direct
        //     self-recursion and mutual recursion like A→B→A→B)
        let depth = self.call_stack.len();
        if depth > 2500 {
            return Err(self.wrap_error(CfmlError::runtime(
                format!("Call stack overflow (depth {})", depth)
            )));
        }
        if depth > 64 {
            // Collect the last 32 function names
            let window = 32.min(depth);
            let recent: Vec<&str> = self.call_stack[depth - window..]
                .iter()
                .map(|f| f.function_name.as_str())
                .collect();
            // Try cycle lengths 1 (A,A,A), 2 (A,B,A,B), 3 (A,B,C,A,B,C)...
            'cycle: for cycle_len in 1..=4 {
                if window < cycle_len * 4 {
                    continue; // need at least 4 full repeats to be confident
                }
                let pattern = &recent[recent.len() - cycle_len..];
                let check_count = window / cycle_len;
                for i in 0..check_count {
                    let offset = recent.len() - cycle_len * (i + 1);
                    let chunk = &recent[offset..offset + cycle_len];
                    if chunk != pattern {
                        continue 'cycle;
                    }
                }
                // All checked repetitions matched — likely infinite
                let cycle_desc = pattern.join(" -> ");
                return Err(self.wrap_error(CfmlError::runtime(
                    format!(
                        "Likely infinite recursion detected: {} (depth {})",
                        cycle_desc, depth
                    )
                )));
            }
        }

        let mut locals: IndexMap<String, CfmlValue> = IndexMap::new();
        let mut stack: Vec<CfmlValue> = Vec::new();
        let mut ip = 0;
        // Track variables declared with `var` (function-local, not written back to parent)
        let mut declared_locals: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Shared closure environment: all closures defined within this function
        // invocation share one Rc<RefCell<HashMap>>. Lazily created on first DefineFunction.
        let mut closure_env: Option<Arc<RwLock<IndexMap<String, CfmlValue>>>> = None;

        // Copy parent scope variables (closures and nested functions see parent vars)
        if let Some(parent) = parent_scope {
            for (k, v) in parent {
                locals.insert(k.clone(), v.clone());
            }
        }

        // Build CFML arguments scope
        let mut arguments_map: IndexMap<String, CfmlValue> = IndexMap::new();
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

        // Push call frame for stack trace tracking (skip __main__ — it's the root)
        if func.name != "__main__" {
            self.call_stack.push(CallFrame {
                function_name: func.name.clone(),
                template: func.source_file.clone()
                    .or_else(|| self.source_file.clone())
                    .unwrap_or_default(),
                line: 0,
                caller_line: self.current_line,
            });
        }

        loop {
            if ip >= func.instructions.len() {
                break;
            }


            let op = func.instructions[ip].clone();
            ip += 1;
            let is_inside_function = func.name != "__main__";

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
                    let val = if name_lower == "variables" || (name_lower == "local" && is_inside_function) {
                        // Return a struct representing the local/variables scope
                        // At top level, merge globals so setVariable/direct-global writes are visible
                        if !is_inside_function {
                            let mut merged = self.globals.clone();
                            for (k, v) in &locals {
                                merged.insert(k.clone(), v.clone());
                            }
                            CfmlValue::Struct(merged)
                        } else {
                            CfmlValue::Struct(locals.clone())
                        }
                    } else if name_lower == "request" {
                        CfmlValue::Struct(self.request_scope.clone())
                    } else if name_lower == "application" {
                        if let Some(ref app_scope) = self.application_scope {
                            if let Ok(scope) = app_scope.lock() {
                                CfmlValue::Struct(scope.clone())
                            } else {
                                CfmlValue::Struct(IndexMap::new())
                            }
                        } else {
                            CfmlValue::Struct(IndexMap::new())
                        }
                    } else if name_lower == "session" {
                        self.get_session_scope()
                    } else if name_lower == "cookie" {
                        self.globals.get("cookie").cloned()
                            .unwrap_or(CfmlValue::Struct(IndexMap::new()))
                    } else if name_lower == "server" {
                        let mut info = IndexMap::new();
                        info.insert("coldfusion".to_string(), CfmlValue::Struct({
                            let mut cf = IndexMap::new();
                            cf.insert("productname".to_string(), CfmlValue::String("RustCFML".to_string()));
                            cf.insert("productversion".to_string(), CfmlValue::String(env!("CARGO_PKG_VERSION").to_string()));
                            cf
                        }));
                        info.insert("os".to_string(), CfmlValue::Struct({
                            let mut os = IndexMap::new();
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
                        if let Some(val) = locals
                            .iter()
                            .find(|(k, _)| k.to_lowercase() == name_lower)
                            .map(|(_, v)| v.clone()) {
                            val
                        } else if let Some(val) = self.globals.iter()
                            .find(|(k, _)| k.to_lowercase() == name_lower)
                            .map(|(_, v)| v.clone()) {
                            // Case-insensitive globals lookup
                            val
                        } else {
                            // Variable not found — check try_stack for error handler
                            if let Some(handler) = self.try_stack.pop() {
                                let mut exception = IndexMap::new();
                                exception.insert("message".to_string(), CfmlValue::String(format!("Variable '{}' is undefined", name)));
                                exception.insert("type".to_string(), CfmlValue::String("expression".to_string()));
                                exception.insert("detail".to_string(), CfmlValue::String(String::new()));
                                stack.truncate(handler.stack_depth);
                                let exc = CfmlValue::Struct(exception);
                                self.last_exception = Some(exc.clone());
                                locals.insert("cfcatch".to_string(), exc);
                                ip = handler.catch_ip;
                                continue;
                            }
                            return Err(self.wrap_error(CfmlError::runtime(
                                format!("Variable '{}' is undefined", name)
                            )));
                        }
                    };
                    stack.push(val);
                }
                BytecodeOp::TryLoadLocal(name) => {
                    // Safe load: returns Null for undefined vars (used by Elvis, null-safe, isNull)
                    let name_lower = name.to_lowercase();
                    let val = if name_lower == "variables" || (name_lower == "local" && is_inside_function) {
                        CfmlValue::Struct(locals.clone())
                    } else if name_lower == "request" {
                        CfmlValue::Struct(self.request_scope.clone())
                    } else if name_lower == "application" {
                        if let Some(ref app_scope) = self.application_scope {
                            if let Ok(scope) = app_scope.lock() {
                                CfmlValue::Struct(scope.clone())
                            } else {
                                CfmlValue::Null
                            }
                        } else {
                            CfmlValue::Null
                        }
                    } else if name_lower == "server" {
                        CfmlValue::Null // server scope handled by LoadLocal
                    } else if let Some(val) = locals.get(&name) {
                        val.clone()
                    } else if let Some(val) = self.globals.get(&name) {
                        val.clone()
                    } else {
                        locals
                            .iter()
                            .find(|(k, _)| k.to_lowercase() == name_lower)
                            .map(|(_, v)| v.clone())
                            .or_else(|| self.globals.iter()
                                .find(|(k, _)| k.to_lowercase() == name_lower)
                                .map(|(_, v)| v.clone()))
                            .unwrap_or(CfmlValue::Null)
                    };
                    stack.push(val);
                }
                BytecodeOp::DeclareLocal(name) => {
                    // Mark this variable as function-local (var keyword)
                    declared_locals.insert(name);
                }
                BytecodeOp::StoreLocal(name) => {
                    if let Some(val) = stack.pop() {
                        let name_lower = name.to_lowercase();
                        if name_lower == "variables" || (name_lower == "local" && is_inside_function) {
                            // Writing to local/variables scope: merge keys back
                            // (preserves arguments, this, etc. that may not be in the snapshot)
                            if let CfmlValue::Struct(s) = val {
                                for (k, v) in s {
                                    locals.insert(k, v);
                                }
                            }
                        } else if name_lower == "request" {
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
                        } else if name_lower == "session" {
                            if let CfmlValue::Struct(s) = &val {
                                self.set_session_scope(s.clone());
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
                            captured_scope: None,
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
                            captured_scope: None,
                        }));
                    // 4. Check VM-intercepted function names (custom tags, etc.)
                    } else if matches!(name_lower.as_str(),
                        "__cfcustomtag" | "__cfcustomtag_start" | "__cfcustomtag_end"
                    ) {
                        stack.push(CfmlValue::Function(cfml_common::dynamic::CfmlFunction {
                            name: name.clone(),
                            params: Vec::new(),
                            body: cfml_common::dynamic::CfmlClosureBody::Expression(
                                Box::new(CfmlValue::Null),
                            ),
                            return_type: None,
                            access: cfml_common::dynamic::CfmlAccess::Public,
                            captured_scope: None,
                        }));
                    } else {
                        return Err(self.wrap_error(CfmlError::runtime(
                            format!("Variable '{}' is undefined", name)
                        )));
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
                    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
                        let x = to_number(&a).unwrap_or(0.0);
                        let y = to_number(&b).unwrap_or(1.0);
                        if y == 0.0 {
                            // CFML throws on division by zero
                            let mut exception = IndexMap::new();
                            exception.insert("message".to_string(), CfmlValue::String("Division by zero is not allowed.".to_string()));
                            exception.insert("type".to_string(), CfmlValue::String("Expression".to_string()));
                            exception.insert("detail".to_string(), CfmlValue::String(String::new()));
                            exception.insert("tagcontext".to_string(), self.build_tag_context());
                            let error_val = CfmlValue::Struct(exception);
                            self.last_exception = Some(error_val.clone());
                            if let Some(handler) = self.try_stack.pop() {
                                while stack.len() > handler.stack_depth {
                                    stack.pop();
                                }
                                stack.push(error_val);
                                ip = handler.catch_ip;
                                continue;
                            } else {
                                return Err(CfmlError::runtime("Division by zero is not allowed.".to_string()));
                            }
                        } else {
                            stack.push(CfmlValue::Double(x / y));
                        }
                    }
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
                        self.closure_parent_writeback = None;
                        // If the function has a captured scope (closure), snapshot from
                        // the shared RefCell so the closure sees its defining scope's vars.
                        // Merge caller's locals on top so the closure also sees the
                        // caller's scope (but captured vars take lower priority — the
                        // closure's own params will override via execute_function_with_args).
                        let effective_locals = if let CfmlValue::Function(ref f) = func_ref {
                            if let Some(ref shared_env) = f.captured_scope {
                                let mut merged = shared_env.read().unwrap().clone();
                                // Caller's locals overlay (for nested closures that
                                // also need to see their immediate caller's vars)
                                for (k, v) in &locals {
                                    if !merged.contains_key(k) {
                                        merged.insert(k.clone(), v.clone());
                                    }
                                }
                                merged
                            } else {
                                locals.clone()
                            }
                        } else {
                            locals.clone()
                        };
                        // Isolate try-stack so throws inside the callee
                        // don't consume the caller's handlers
                        let saved_try_stack = std::mem::take(&mut self.try_stack);
                        let call_result = self.call_function(&func_ref, args, &effective_locals);
                        self.try_stack = saved_try_stack;
                        match call_result {
                            Ok(result) => {
                                // Write back mutations into the shared closure environment
                                if let Some(ref writeback) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&func_ref, writeback);
                                }
                                // Merge closure write-back into caller's locals
                                if let Some(writeback) = self.closure_parent_writeback.take() {
                                    for (k, v) in writeback {
                                        locals.insert(k, v);
                                    }
                                }
                                stack.push(result);
                            }
                            Err(e) => {
                                // Route error through try-catch mechanism
                                if let Some(handler) = self.try_stack.pop() {
                                    while stack.len() > handler.stack_depth {
                                        stack.pop();
                                    }
                                    let error_val = self.last_exception.clone().unwrap_or_else(|| {
                                        let mut err_struct = IndexMap::new();
                                        err_struct.insert("message".to_string(), CfmlValue::String(e.message.clone()));
                                        err_struct.insert("type".to_string(), CfmlValue::String(format!("{}", e.error_type)));
                                        err_struct.insert("detail".to_string(), CfmlValue::String(String::new()));
                                        err_struct.insert("tagcontext".to_string(), self.build_tag_context());
                                        CfmlValue::Struct(err_struct)
                                    });
                                    stack.push(error_val);
                                    ip = handler.catch_ip;
                                } else {
                                    return Err(e);
                                }
                            }
                        }
                    } else {
                        stack.push(CfmlValue::Null);
                    }
                }

                BytecodeOp::Return => {
                    // Save modified 'this' for component method write-back
                    if let Some(this_val) = locals.get("this") {
                        self.method_this_writeback = Some(this_val.clone());
                    }
                    // Closure parent scope write-back on early return
                    if let Some(parent) = parent_scope {
                        let mut writeback = IndexMap::new();
                        for (k, v) in &locals {
                            // Skip function params, arguments scope, 'this', and var-declared locals
                            if k == "arguments" || k == "this" || func.params.contains(k)
                                || declared_locals.contains(k.as_str())
                            {
                                continue;
                            }
                            if let Some(parent_val) = parent.get(k) {
                                if !Self::values_equal_shallow(v, parent_val) {
                                    writeback.insert(k.clone(), v.clone());
                                }
                            } else {
                                writeback.insert(k.clone(), v.clone());
                            }
                        }
                        if !writeback.is_empty() {
                            self.closure_parent_writeback = Some(writeback);
                        }
                    }
                    // Pop call frame before early return (matches push at function entry)
                    self.call_stack.pop();
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
                    let mut map = IndexMap::new();
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
                            let val = s.get(&key)
                                .or_else(|| s.get(&key.to_uppercase()))
                                .or_else(|| s.get(&key.to_lowercase()))
                                .or_else(|| {
                                    let key_lower = key.to_lowercase();
                                    s.iter().find(|(k, _)| k.to_lowercase() == key_lower).map(|(_, v)| v)
                                })
                                .cloned()
                                .unwrap_or(CfmlValue::Null);
                            stack.push(val);
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
                                    .or_else(|| {
                                        // Full case-insensitive scan for mixed-case keys
                                        let name_lower = name.to_lowercase();
                                        s.iter().find(|(k, _)| k.to_lowercase() == name_lower).map(|(_, v)| v)
                                    })
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
                                .unwrap_or(CfmlValue::Struct(IndexMap::new()))
                        };

                        // Resolve inheritance chain
                        let instance = self.resolve_inheritance(template, &locals);

                        // Validate interface implementation and collect transitive interfaces
                        let instance = if let CfmlValue::Struct(ref s) = instance {
                            let all_ifaces = self.validate_interface_implementation(s, &locals)?;
                            if !all_ifaces.is_empty() {
                                let mut s = s.clone();
                                let chain: Vec<CfmlValue> = all_ifaces.into_iter()
                                    .map(|name| CfmlValue::String(name))
                                    .collect();
                                s.insert("__implements_chain".to_string(), CfmlValue::Array(chain));
                                CfmlValue::Struct(s)
                            } else {
                                instance
                            }
                        } else {
                            instance
                        };

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
                    self.user_functions.insert(func_name.clone(), self.program.functions[func_idx].clone());
                    // Create or reuse a shared closure environment so all closures
                    // defined in this function invocation share the same mutable state.
                    let env = closure_env.get_or_insert_with(|| {
                        Arc::new(RwLock::new(locals.clone()))
                    });
                    // Sync current locals into the shared env so that closures defined
                    // later see variables declared between earlier DefineFunction ops.
                    {
                        let mut m = env.write().unwrap();
                        for (k, v) in &locals {
                            m.insert(k.clone(), v.clone());
                        }
                    }
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
                        captured_scope: Some(Arc::clone(env)),
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
                    self.last_exception = Some(error_val.clone());
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
                BytecodeOp::Rethrow => {
                    let error_val = self.last_exception.clone().unwrap_or(
                        CfmlValue::String("No exception to rethrow".to_string()),
                    );
                    if let Some(handler) = self.try_stack.pop() {
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
                    // Isolate try-stack so throws inside the callee
                    // don't consume the caller's handlers
                    let saved_try_stack_method = std::mem::take(&mut self.try_stack);
                    let method_result: Result<CfmlValue, CfmlError> = if let CfmlValue::Struct(ref s) = object {
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
                                let mut method_locals = IndexMap::new();
                                // Merge captured scope first (closure vars from defining scope)
                                if let Some(ref shared_env) = f.captured_scope {
                                    for (k, v) in shared_env.read().unwrap().iter() {
                                        method_locals.insert(k.clone(), v.clone());
                                    }
                                }
                                // Inject component __variables scope from this
                                let this_ref = locals.get("this").unwrap_or(&object);
                                if let CfmlValue::Struct(ref ts) = this_ref {
                                    if let Some(CfmlValue::Struct(vars)) = ts.get("__variables") {
                                        for (k, v) in vars {
                                            method_locals.insert(k.clone(), v.clone());
                                        }
                                    }
                                }
                                // Use the actual child 'this' from caller's locals
                                if let Some(real_this) = locals.get("this") {
                                    method_locals.insert("this".to_string(), real_this.clone());
                                } else {
                                    method_locals.insert("this".to_string(), object.clone());
                                }
                                // Execute directly by index to avoid name collision
                                self.closure_parent_writeback = None;
                                let call_result = if let Some(idx) = func_idx {
                                    if idx < self.program.functions.len() {
                                        let parent_func = self.program.functions[idx].clone();
                                        self.execute_function_with_args(&parent_func, args, Some(&method_locals))
                                    } else {
                                        self.call_function(&prop, args, &method_locals)
                                    }
                                } else {
                                    self.call_function(&prop, args, &method_locals)
                                };
                                // Write back closure mutations to shared environment
                                if let Ok(ref _val) = call_result {
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&prop, wb);
                                    }
                                }
                                call_result
                            } else {
                                self.call_member_function(&object, &method_name, &mut extra_args)
                            }
                        } else {
                            self.call_member_function(&object, &method_name, &mut extra_args)
                        }
                    } else {
                        self.call_member_function(&object, &method_name, &mut extra_args)
                    };
                    self.try_stack = saved_try_stack_method;
                    let result = match method_result {
                        Ok(val) => val,
                        Err(e) => {
                            // Route error through try-catch mechanism
                            if let Some(handler) = self.try_stack.pop() {
                                while stack.len() > handler.stack_depth {
                                    stack.pop();
                                }
                                let error_val = self.last_exception.clone().unwrap_or_else(|| {
                                    let mut err_struct = IndexMap::new();
                                    err_struct.insert("message".to_string(), CfmlValue::String(e.message.clone()));
                                    err_struct.insert("type".to_string(), CfmlValue::String(format!("{}", e.error_type)));
                                    err_struct.insert("detail".to_string(), CfmlValue::String(String::new()));
                                    err_struct.insert("tagcontext".to_string(), self.build_tag_context());
                                    CfmlValue::Struct(err_struct)
                                });
                                stack.push(error_val);
                                ip = handler.catch_ip;
                                continue;
                            } else {
                                return Err(e);
                            }
                        }
                    };

                    // Write-back: emulate CFML pass-by-reference semantics for mutating methods.
                    // The compiler encodes a path vec: ["var"], ["var", "prop"], ["a", "b", "c"], etc.
                    if let Some(ref path) = write_back {
                        if path.len() == 1 {
                            // Direct variable write-back: var.method(args)
                            let var_name = &path[0];
                            if Self::is_mutating_method(&method_name) {
                                self.scope_aware_store(var_name, result.clone(), &mut locals);
                            }
                        } else if path.len() >= 2 && Self::is_mutating_method(&method_name) {
                            // Deep property write-back: var.prop1.prop2...propN.method(args)
                            let var_name = &path[0];
                            if let Some(mut root_obj) = self.scope_aware_load(var_name, &locals) {
                                let props = &path[1..];
                                Self::deep_set(&mut root_obj, props, result.clone());
                                self.scope_aware_store(var_name, root_obj, &mut locals);
                            }
                        }
                    }

                    // Propagate component method `this` modifications back to caller.
                    // When a component method modifies `this` internally, the modified
                    // `this` is saved by execute_function_with_args. Write it back.
                    if let Some(modified_this) = self.method_this_writeback.take() {
                        if let Some(ref path) = write_back {
                            let var_name = &path[0];
                            if path.len() == 1 {
                                self.scope_aware_store(var_name, modified_this, &mut locals);
                            } else {
                                // Deep write-back for component this
                                if let Some(mut root_obj) = self.scope_aware_load(var_name, &locals) {
                                    let props = &path[1..];
                                    Self::deep_set(&mut root_obj, props, modified_this);
                                    self.scope_aware_store(var_name, root_obj, &mut locals);
                                }
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
                            CfmlValue::Query(q) => {
                                // Iterating over a query: convert to array of row structs
                                let rows: Vec<CfmlValue> = q.rows.iter()
                                    .map(|row| CfmlValue::Struct(row.clone()))
                                    .collect();
                                stack.push(CfmlValue::Array(rows));
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

                    // If relative resolution fails and path starts with "/", try mappings
                    let resolved = if !std::path::Path::new(&resolved).exists() && path.starts_with('/') {
                        // Convert /taffy/core/foo.cfm → try mapping lookup
                        self.resolve_include_with_mappings(&path).unwrap_or(resolved)
                    } else {
                        resolved
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
                                    let old_source = self.source_file.clone();
                                    self.source_file = Some(resolved.clone());
                                    let main_idx = self.program.functions.iter()
                                        .position(|f| f.name == "__main__")
                                        .unwrap_or(0);
                                    let inc_func = self.program.functions[main_idx].clone();
                                    // Isolate try-stack so throws inside the include
                                    // don't consume outer handlers
                                    let saved_try_stack = std::mem::take(&mut self.try_stack);
                                    let result = self.execute_function_with_args(&inc_func, Vec::new(), Some(&locals));
                                    self.try_stack = saved_try_stack;
                                    self.program = old_program;
                                    self.source_file = old_source;
                                    // Propagate include errors through try-catch
                                    if let Err(e) = result {
                                        if let Some(handler) = self.try_stack.pop() {
                                            while stack.len() > handler.stack_depth {
                                                stack.pop();
                                            }
                                            let mut err_struct = IndexMap::new();
                                            err_struct.insert("message".to_string(), CfmlValue::String(e.message.clone()));
                                            err_struct.insert("type".to_string(), CfmlValue::String(format!("{}", e.error_type)));
                                            err_struct.insert("detail".to_string(), CfmlValue::String(String::new()));
                                            err_struct.insert("tagcontext".to_string(), self.build_tag_context());
                                            let error_val = CfmlValue::Struct(err_struct);
                                            stack.push(error_val);
                                            ip = handler.catch_ip;
                                        } else {
                                            return Err(e);
                                        }
                                    }
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
                BytecodeOp::IsDefined(var_name) => {
                    let defined = self.is_variable_defined(&var_name, &locals);
                    stack.push(CfmlValue::Bool(defined));
                }

                BytecodeOp::ConcatArrays => {
                    let right = stack.pop().unwrap_or(CfmlValue::Array(Vec::new()));
                    let left = stack.pop().unwrap_or(CfmlValue::Array(Vec::new()));
                    if let (CfmlValue::Array(mut a), CfmlValue::Array(b)) = (left, right) {
                        a.extend(b);
                        stack.push(CfmlValue::Array(a));
                    } else {
                        stack.push(CfmlValue::Array(Vec::new()));
                    }
                }

                BytecodeOp::MergeStructs => {
                    let right = stack.pop().unwrap_or(CfmlValue::Struct(IndexMap::new()));
                    let left = stack.pop().unwrap_or(CfmlValue::Struct(IndexMap::new()));
                    if let (CfmlValue::Struct(mut a), CfmlValue::Struct(b)) = (left, right) {
                        for (k, v) in b {
                            a.insert(k, v);
                        }
                        stack.push(CfmlValue::Struct(a));
                    } else {
                        stack.push(CfmlValue::Struct(IndexMap::new()));
                    }
                }

                BytecodeOp::CallSpread => {
                    // Stack: [func_ref, args_array]
                    let args_val = stack.pop().unwrap_or(CfmlValue::Array(Vec::new()));
                    let func_ref = stack.pop().unwrap_or(CfmlValue::Null);
                    let args = if let CfmlValue::Array(a) = args_val {
                        a
                    } else {
                        vec![args_val]
                    };
                    self.closure_parent_writeback = None;
                    let result = self.call_function(&func_ref, args, &locals)?;
                    // Write back mutations into the shared closure environment
                    if let Some(ref writeback) = self.closure_parent_writeback {
                        Self::write_back_to_captured_scope(&func_ref, writeback);
                    }
                    if let Some(writeback) = self.closure_parent_writeback.take() {
                        for (k, v) in writeback {
                            locals.insert(k, v);
                        }
                    }
                    stack.push(result);
                }

                BytecodeOp::LineInfo(line, col) => {
                    self.current_line = line;
                    self.current_column = col;
                    // Update the current call frame's line so the stack trace
                    // reflects where execution is within this function
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.line = line;
                    }
                }

                BytecodeOp::Halt => break,
            }
        }

        // Pop call frame on function exit
        self.call_stack.pop();

        // Save modified 'this' for component method write-back
        if let Some(this_val) = locals.get("this") {
            self.method_this_writeback = Some(this_val.clone());
        }

        // Closure parent scope write-back: compute diff of parent-scope vars
        if let Some(parent) = parent_scope {
            let mut writeback = IndexMap::new();
            for (k, v) in &locals {
                // Skip function params, arguments scope, 'this', and var-declared locals
                if k == "arguments" || k == "this" || func.params.contains(k)
                    || declared_locals.contains(k.as_str())
                {
                    continue;
                }
                // Only write back vars that existed in parent scope OR are new
                if let Some(parent_val) = parent.get(k) {
                    // Only include if changed
                    if !Self::values_equal_shallow(v, parent_val) {
                        writeback.insert(k.clone(), v.clone());
                    }
                } else {
                    // New variable created in closure - propagate to parent
                    writeback.insert(k.clone(), v.clone());
                }
            }
            if !writeback.is_empty() {
                self.closure_parent_writeback = Some(writeback);
            }
        }

        // Capture locals for component variables scope (only for __main__)
        if func.name == "__main__" {
            self.captured_locals = Some(locals);
        }

        Ok(stack.pop().unwrap_or(CfmlValue::Null))
    }

    fn call_function(
        &mut self,
        func_ref: &CfmlValue,
        args: Vec<CfmlValue>,
        parent_locals: &IndexMap<String, CfmlValue>,
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
                | "arraysome" | "arrayevery" | "arrayfindall" | "arrayfindallnocase"
                | "structeach" | "structmap" | "structfilter"
                | "structreduce" | "structsome" | "structevery"
                | "listeach" | "listmap" | "listfilter" | "listreduce"
                | "listsome" | "listevery" | "listreduceright"
                | "stringeach" | "stringmap" | "stringfilter" | "stringreduce"
                | "stringsome" | "stringevery" | "stringsort"
                | "collectioneach" | "collectionmap" | "collectionfilter"
                | "collectionreduce" | "collectionsome" | "collectionevery"
                | "each"
                | "queryeach" | "querymap" | "queryfilter" | "queryreduce"
                | "querysort" | "querysome" | "queryevery"
                | "createobject"
                | "getcurrenttemplatepath"
                | "getcomponentmetadata"
                | "__cfheader" | "__cfcontent" | "__cflocation" | "__cfabort"
                | "gethttprequestdata" | "__cfinvoke"
                | "__cfsavecontent_start" | "__cfsavecontent_end" | "invoke"
                | "getbasetemplatepath" | "gettimezone"
                | "expandpath"
                | "isdefined"
                | "queryexecute"
                | "__cftransaction_start" | "__cftransaction_commit" | "__cftransaction_rollback"
                | "__cflog" | "__cfsetting" | "__cflock_start" | "__cflock_end" | "__cfcookie"
                | "fileupload" | "fileuploadall" | "__cffile_upload"
                | "sessioninvalidate" | "sessionrotate" | "sessiongetmetadata"
                | "getauthuser" | "isuserinrole" | "isuserloggedin"
                | "__cfloginuser" | "__cflogout"
                | "setvariable" | "getvariable" | "throw"
                | "__cfcustomtag" | "__cfcustomtag_start" | "__cfcustomtag_end"
                | "cacheput" | "cacheget" | "cachedelete" | "cacheclear"
                | "cachekeyexists" | "cachecount" | "cachegetall" | "cachegetallids"
                | "__cfcache" | "__cfexecute"
                | "__cfthread_run" | "__cfthread_join" | "__cfthread_terminate" => {
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

            // If the function has a captured scope (closure), merge it with
            // the caller's parent_locals so the closure sees its defining scope.
            let effective_locals;
            let effective_parent = if let Some(ref shared_env) = func.captured_scope {
                effective_locals = {
                    let mut merged = shared_env.read().unwrap().clone();
                    for (k, v) in parent_locals {
                        if !merged.contains_key(k) {
                            merged.insert(k.clone(), v.clone());
                        }
                    }
                    merged
                };
                &effective_locals
            } else {
                parent_locals
            };

            // If the function has a stored bytecode index, use it directly
            // (avoids name collision when parent/child have same-named methods)
            if let cfml_common::dynamic::CfmlClosureBody::Expression(ref body) = func.body {
                if let CfmlValue::Int(idx) = body.as_ref() {
                    let idx = *idx as usize;
                    if idx < self.program.functions.len() {
                        let user_func = self.program.functions[idx].clone();
                        return self.execute_function_with_args(&user_func, args, Some(effective_parent));
                    }
                }
            }

            // Check user-defined functions by name
            if let Some(user_func) = self.user_functions.get(&func.name).cloned() {
                return self.execute_function_with_args(&user_func, args, Some(parent_locals));
            }

            // Case-insensitive user function lookup
            let user_match = self
                .user_functions
                .iter()
                .find(|(k, _)| k.to_lowercase() == name_lower)
                .map(|(_, v)| v.clone());

            if let Some(user_func) = user_match {
                return self.execute_function_with_args(&user_func, args, Some(parent_locals));
            }

            // Higher-order standalone functions (arrayMap, arrayFilter, arrayReduce, etc.)
            match name_lower.as_str() {
                "arraymap" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let mut result = Vec::new();
                            let callback = callback.clone();
                            let mut pl = parent_locals.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                self.closure_parent_writeback = None;
                                let mapped = self.call_function(&callback, cb_args, &pl)?;
                                if let Some(wb) = self.closure_parent_writeback.take() {
                                    for (k, v) in &wb { pl.insert(k.clone(), v.clone()); }
                                    Self::write_back_to_captured_scope(&callback, &wb);
                                    self.closure_parent_writeback = Some(wb);
                                }
                                result.push(mapped);
                            }
                            self.set_ho_final_writeback(&pl, parent_locals);
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
                            let mut pl = parent_locals.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                self.closure_parent_writeback = None;
                                let keep = self.call_function(&callback, cb_args, &pl)?;
                                if let Some(wb) = self.closure_parent_writeback.take() {
                                    for (k, v) in &wb { pl.insert(k.clone(), v.clone()); }
                                    Self::write_back_to_captured_scope(&callback, &wb);
                                    self.closure_parent_writeback = Some(wb);
                                }
                                if keep.is_true() {
                                    result.push(item.clone());
                                }
                            }
                            self.set_ho_final_writeback(&pl, parent_locals);
                            return Ok(CfmlValue::Array(result));
                        }
                    }
                    return Ok(CfmlValue::Array(Vec::new()));
                }
                "arrayfindall" | "arrayfindallnocase" => {
                    // arrayFindAll(array, callback) - callback(item, index, array)
                    // When called with a callback, returns indices where callback returns true
                    if let (Some(arr_val), Some(arg1)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            // Check if second arg is a callback (Function) or a simple value
                            if matches!(arg1, CfmlValue::Function(_)) {
                                let callback = arg1.clone();
                                let mut pl = parent_locals.clone();
                                let mut result = Vec::new();
                                for (i, item) in arr.iter().enumerate() {
                                    let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                    self.closure_parent_writeback = None;
                                    let keep = self.call_function(&callback, cb_args, &pl)?;
                                    if let Some(wb) = self.closure_parent_writeback.take() {
                                        for (k, v) in &wb { pl.insert(k.clone(), v.clone()); }
                                        Self::write_back_to_captured_scope(&callback, &wb);
                                        self.closure_parent_writeback = Some(wb);
                                    }
                                    if keep.is_true() {
                                        result.push(CfmlValue::Int((i + 1) as i64));
                                    }
                                }
                                self.set_ho_final_writeback(&pl, parent_locals);
                                return Ok(CfmlValue::Array(result));
                            } else {
                                // Simple value comparison: fall through to builtin
                            }
                        }
                    }
                    // Fall through to the builtin fn_array_find_all for simple value comparison
                }
                "arrayreduce" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let mut acc = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                            let callback = callback.clone();
                            let mut pl = parent_locals.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![acc.clone(), item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                self.closure_parent_writeback = None;
                                acc = self.call_function(&callback, cb_args, &pl)?;
                                if let Some(wb) = self.closure_parent_writeback.take() {
                                    for (k, v) in &wb { pl.insert(k.clone(), v.clone()); }
                                    Self::write_back_to_captured_scope(&callback, &wb);
                                    self.closure_parent_writeback = Some(wb);
                                }
                            }
                            self.set_ho_final_writeback(&pl, parent_locals);
                            return Ok(acc);
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "arrayeach" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let callback = callback.clone();
                            let mut pl = parent_locals.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                self.closure_parent_writeback = None;
                                self.call_function(&callback, cb_args, &pl)?;
                                if let Some(wb) = self.closure_parent_writeback.take() {
                                    for (k, v) in &wb { pl.insert(k.clone(), v.clone()); }
                                    Self::write_back_to_captured_scope(&callback, &wb);
                                    self.closure_parent_writeback = Some(wb);
                                }
                            }
                            self.set_ho_final_writeback(&pl, parent_locals);
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "structeach" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let callback = callback.clone();
                            let mut pl = parent_locals.clone();
                            for (k, v) in s {
                                let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                self.closure_parent_writeback = None;
                                self.call_function(&callback, cb_args, &pl)?;
                                if let Some(wb) = self.closure_parent_writeback.take() {
                                    for (k, v) in &wb { pl.insert(k.clone(), v.clone()); }
                                    Self::write_back_to_captured_scope(&callback, &wb);
                                    self.closure_parent_writeback = Some(wb);
                                }
                            }
                            self.set_ho_final_writeback(&pl, parent_locals);
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "structmap" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let mut result = IndexMap::new();
                            let callback = callback.clone();
                            let mut pl = parent_locals.clone();
                            for (k, v) in s {
                                let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                self.closure_parent_writeback = None;
                                let mapped = self.call_function(&callback, cb_args, &pl)?;
                                if let Some(wb) = self.closure_parent_writeback.take() {
                                    for (k2, v2) in &wb { pl.insert(k2.clone(), v2.clone()); }
                                    Self::write_back_to_captured_scope(&callback, &wb);
                                    self.closure_parent_writeback = Some(wb);
                                }
                                result.insert(k.clone(), mapped);
                            }
                            self.set_ho_final_writeback(&pl, parent_locals);
                            return Ok(CfmlValue::Struct(result));
                        }
                    }
                    return Ok(CfmlValue::Struct(IndexMap::new()));
                }
                "structfilter" => {
                    if let (Some(struct_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Struct(s) = struct_val {
                            let mut result = IndexMap::new();
                            let callback = callback.clone();
                            let mut pl = parent_locals.clone();
                            for (k, v) in s {
                                let cb_args = vec![CfmlValue::String(k.clone()), v.clone(), struct_val.clone()];
                                self.closure_parent_writeback = None;
                                let keep = self.call_function(&callback, cb_args, &pl)?;
                                if let Some(wb) = self.closure_parent_writeback.take() {
                                    for (k2, v2) in &wb { pl.insert(k2.clone(), v2.clone()); }
                                    Self::write_back_to_captured_scope(&callback, &wb);
                                    self.closure_parent_writeback = Some(wb);
                                }
                                if keep.is_true() {
                                    result.insert(k.clone(), v.clone());
                                }
                            }
                            self.set_ho_final_writeback(&pl, parent_locals);
                            return Ok(CfmlValue::Struct(result));
                        }
                    }
                    return Ok(CfmlValue::Struct(IndexMap::new()));
                }
                "arraysome" => {
                    if let (Some(arr_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Array(arr) = arr_val {
                            let callback = callback.clone();
                            for (i, item) in arr.iter().enumerate() {
                                let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), arr_val.clone()];
                                self.closure_parent_writeback = None;
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
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
                                self.closure_parent_writeback = None;
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
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
                                self.closure_parent_writeback = None;
                                acc = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
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
                                self.closure_parent_writeback = None;
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
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
                                self.closure_parent_writeback = None;
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
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
                            self.closure_parent_writeback = None;
                            self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            let keep = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            acc = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "listreduceright" => {
                    if let (Some(list_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let list = list_val.as_string();
                        let mut acc = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                        let delimiter = args.get(3).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string());
                        let callback = callback.clone();
                        let items: Vec<&str> = list.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
                        for (i, item) in items.iter().enumerate().rev() {
                            let cb_args = vec![acc.clone(), CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), list_val.clone()];
                            self.closure_parent_writeback = None;
                            acc = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "listsome" => {
                    if let (Some(list_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let list = list_val.as_string();
                        let delimiter = args.get(2).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string());
                        let callback = callback.clone();
                        let items: Vec<&str> = list.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
                        for (i, item) in items.iter().enumerate() {
                            let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), list_val.clone()];
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if result.is_true() {
                                return Ok(CfmlValue::Bool(true));
                            }
                        }
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "listevery" => {
                    if let (Some(list_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let list = list_val.as_string();
                        let delimiter = args.get(2).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string());
                        let callback = callback.clone();
                        let items: Vec<&str> = list.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
                        for (i, item) in items.iter().enumerate() {
                            let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), list_val.clone()];
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if !result.is_true() {
                                return Ok(CfmlValue::Bool(false));
                            }
                        }
                    }
                    return Ok(CfmlValue::Bool(true));
                }
                // ---- String Higher-Order Functions ----
                "stringeach" => {
                    if let (Some(str_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let s = str_val.as_string();
                        let callback = callback.clone();
                        for (i, ch) in s.chars().enumerate() {
                            let cb_args = vec![CfmlValue::String(ch.to_string()), CfmlValue::Int((i + 1) as i64), str_val.clone()];
                            self.closure_parent_writeback = None;
                            self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "stringmap" => {
                    if let (Some(str_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let s = str_val.as_string();
                        let callback = callback.clone();
                        let mut result = String::new();
                        for (i, ch) in s.chars().enumerate() {
                            let cb_args = vec![CfmlValue::String(ch.to_string()), CfmlValue::Int((i + 1) as i64), str_val.clone()];
                            self.closure_parent_writeback = None;
                            let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            result.push_str(&mapped.as_string());
                        }
                        return Ok(CfmlValue::String(result));
                    }
                    return Ok(CfmlValue::String(String::new()));
                }
                "stringfilter" => {
                    if let (Some(str_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let s = str_val.as_string();
                        let callback = callback.clone();
                        let mut result = String::new();
                        for (i, ch) in s.chars().enumerate() {
                            let cb_args = vec![CfmlValue::String(ch.to_string()), CfmlValue::Int((i + 1) as i64), str_val.clone()];
                            self.closure_parent_writeback = None;
                            let keep = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if keep.is_true() {
                                result.push(ch);
                            }
                        }
                        return Ok(CfmlValue::String(result));
                    }
                    return Ok(CfmlValue::String(String::new()));
                }
                "stringreduce" => {
                    if let (Some(str_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let s = str_val.as_string();
                        let mut acc = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                        let callback = callback.clone();
                        for (i, ch) in s.chars().enumerate() {
                            let cb_args = vec![acc.clone(), CfmlValue::String(ch.to_string()), CfmlValue::Int((i + 1) as i64), str_val.clone()];
                            self.closure_parent_writeback = None;
                            acc = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "stringsome" => {
                    if let (Some(str_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let s = str_val.as_string();
                        let callback = callback.clone();
                        for (i, ch) in s.chars().enumerate() {
                            let cb_args = vec![CfmlValue::String(ch.to_string()), CfmlValue::Int((i + 1) as i64), str_val.clone()];
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if result.is_true() {
                                return Ok(CfmlValue::Bool(true));
                            }
                        }
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "stringevery" => {
                    if let (Some(str_val), Some(callback)) = (args.get(0), args.get(1)) {
                        let s = str_val.as_string();
                        let callback = callback.clone();
                        for (i, ch) in s.chars().enumerate() {
                            let cb_args = vec![CfmlValue::String(ch.to_string()), CfmlValue::Int((i + 1) as i64), str_val.clone()];
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, parent_locals)?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if !result.is_true() {
                                return Ok(CfmlValue::Bool(false));
                            }
                        }
                    }
                    return Ok(CfmlValue::Bool(true));
                }
                "stringsort" => {
                    if let Some(str_val) = args.get(0) {
                        let s = str_val.as_string();
                        let mut chars: Vec<char> = s.chars().collect();
                        if let Some(callback) = args.get(1) {
                            let callback = callback.clone();
                            // Bubble sort with callback comparator
                            let len = chars.len();
                            for i in 0..len {
                                for j in 0..len - 1 - i {
                                    let cb_args = vec![
                                        CfmlValue::String(chars[j].to_string()),
                                        CfmlValue::String(chars[j + 1].to_string()),
                                    ];
                                    self.closure_parent_writeback = None;
                                    let cmp = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    let cmp_val = match &cmp {
                                        CfmlValue::Int(n) => *n,
                                        CfmlValue::Double(d) => *d as i64,
                                        _ => 0,
                                    };
                                    if cmp_val > 0 {
                                        chars.swap(j, j + 1);
                                    }
                                }
                            }
                        } else {
                            chars.sort();
                        }
                        return Ok(CfmlValue::String(chars.into_iter().collect()));
                    }
                    return Ok(CfmlValue::String(String::new()));
                }
                // ---- Collection Higher-Order Functions ----
                "collectioneach" | "each" => {
                    if let (Some(collection), Some(callback)) = (args.get(0), args.get(1)) {
                        let callback = callback.clone();
                        match collection {
                            CfmlValue::Array(arr) => {
                                for (i, item) in arr.iter().enumerate() {
                                    let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                }
                            }
                            CfmlValue::Struct(s) => {
                                for (key, val) in s.iter() {
                                    let cb_args = vec![CfmlValue::String(key.clone()), val.clone(), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                }
                            }
                            CfmlValue::Query(q) => {
                                for (i, row) in q.rows.iter().enumerate() {
                                    let row_struct = CfmlValue::Struct(row.clone());
                                    let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                }
                            }
                            _ => {
                                // Treat as list
                                let list = collection.as_string();
                                let items: Vec<&str> = list.split(',').filter(|s| !s.is_empty()).collect();
                                for (i, item) in items.iter().enumerate() {
                                    let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                }
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "collectionmap" => {
                    if let (Some(collection), Some(callback)) = (args.get(0), args.get(1)) {
                        let callback = callback.clone();
                        match collection {
                            CfmlValue::Array(arr) => {
                                let mut result = Vec::new();
                                for (i, item) in arr.iter().enumerate() {
                                    let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    result.push(mapped);
                                }
                                return Ok(CfmlValue::Array(result));
                            }
                            CfmlValue::Struct(s) => {
                                let mut result = IndexMap::new();
                                for (key, val) in s.iter() {
                                    let cb_args = vec![CfmlValue::String(key.clone()), val.clone(), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    result.insert(key.clone(), mapped);
                                }
                                return Ok(CfmlValue::Struct(result));
                            }
                            _ => {
                                // Treat as list
                                let list = collection.as_string();
                                let items: Vec<&str> = list.split(',').filter(|s| !s.is_empty()).collect();
                                let mut result: Vec<String> = Vec::new();
                                for (i, item) in items.iter().enumerate() {
                                    let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    result.push(mapped.as_string());
                                }
                                return Ok(CfmlValue::String(result.join(",")));
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "collectionfilter" => {
                    if let (Some(collection), Some(callback)) = (args.get(0), args.get(1)) {
                        let callback = callback.clone();
                        match collection {
                            CfmlValue::Array(arr) => {
                                let mut result = Vec::new();
                                for (i, item) in arr.iter().enumerate() {
                                    let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let keep = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if keep.is_true() {
                                        result.push(item.clone());
                                    }
                                }
                                return Ok(CfmlValue::Array(result));
                            }
                            CfmlValue::Struct(s) => {
                                let mut result = IndexMap::new();
                                for (key, val) in s.iter() {
                                    let cb_args = vec![CfmlValue::String(key.clone()), val.clone(), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let keep = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if keep.is_true() {
                                        result.insert(key.clone(), val.clone());
                                    }
                                }
                                return Ok(CfmlValue::Struct(result));
                            }
                            _ => {
                                // Treat as list
                                let list = collection.as_string();
                                let items: Vec<&str> = list.split(',').filter(|s| !s.is_empty()).collect();
                                let mut result = Vec::new();
                                for (i, item) in items.iter().enumerate() {
                                    let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let keep = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if keep.is_true() {
                                        result.push(item.to_string());
                                    }
                                }
                                return Ok(CfmlValue::String(result.join(",")));
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "collectionreduce" => {
                    if let (Some(collection), Some(callback)) = (args.get(0), args.get(1)) {
                        let mut acc = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                        let callback = callback.clone();
                        match collection {
                            CfmlValue::Array(arr) => {
                                for (i, item) in arr.iter().enumerate() {
                                    let cb_args = vec![acc.clone(), item.clone(), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    acc = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                }
                            }
                            CfmlValue::Struct(s) => {
                                for (key, val) in s.iter() {
                                    let cb_args = vec![acc.clone(), CfmlValue::String(key.clone()), val.clone(), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    acc = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                }
                            }
                            _ => {
                                let list = collection.as_string();
                                let items: Vec<&str> = list.split(',').filter(|s| !s.is_empty()).collect();
                                for (i, item) in items.iter().enumerate() {
                                    let cb_args = vec![acc.clone(), CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    acc = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                }
                            }
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "collectionsome" => {
                    if let (Some(collection), Some(callback)) = (args.get(0), args.get(1)) {
                        let callback = callback.clone();
                        match collection {
                            CfmlValue::Array(arr) => {
                                for (i, item) in arr.iter().enumerate() {
                                    let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let result = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if result.is_true() {
                                        return Ok(CfmlValue::Bool(true));
                                    }
                                }
                            }
                            CfmlValue::Struct(s) => {
                                for (key, val) in s.iter() {
                                    let cb_args = vec![CfmlValue::String(key.clone()), val.clone(), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let result = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if result.is_true() {
                                        return Ok(CfmlValue::Bool(true));
                                    }
                                }
                            }
                            _ => {
                                let list = collection.as_string();
                                let items: Vec<&str> = list.split(',').filter(|s| !s.is_empty()).collect();
                                for (i, item) in items.iter().enumerate() {
                                    let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let result = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if result.is_true() {
                                        return Ok(CfmlValue::Bool(true));
                                    }
                                }
                            }
                        }
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "collectionevery" => {
                    if let (Some(collection), Some(callback)) = (args.get(0), args.get(1)) {
                        let callback = callback.clone();
                        match collection {
                            CfmlValue::Array(arr) => {
                                for (i, item) in arr.iter().enumerate() {
                                    let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let result = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if !result.is_true() {
                                        return Ok(CfmlValue::Bool(false));
                                    }
                                }
                            }
                            CfmlValue::Struct(s) => {
                                for (key, val) in s.iter() {
                                    let cb_args = vec![CfmlValue::String(key.clone()), val.clone(), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let result = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if !result.is_true() {
                                        return Ok(CfmlValue::Bool(false));
                                    }
                                }
                            }
                            _ => {
                                let list = collection.as_string();
                                let items: Vec<&str> = list.split(',').filter(|s| !s.is_empty()).collect();
                                for (i, item) in items.iter().enumerate() {
                                    let cb_args = vec![CfmlValue::String(item.to_string()), CfmlValue::Int((i + 1) as i64), collection.clone()];
                                    self.closure_parent_writeback = None;
                                    let result = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    if !result.is_true() {
                                        return Ok(CfmlValue::Bool(false));
                                    }
                                }
                            }
                        }
                    }
                    return Ok(CfmlValue::Bool(true));
                }
                "queryeach" => {
                    if let (Some(q_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Query(q) = q_val {
                            let callback = callback.clone();
                            for (i, row) in q.rows.iter().enumerate() {
                                let row_struct = CfmlValue::Struct(row.clone());
                                let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), q_val.clone()];
                                self.closure_parent_writeback = None;
                                self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "querymap" => {
                    if let (Some(q_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Query(q) = q_val {
                            let callback = callback.clone();
                            let mut new_rows = Vec::new();
                            for (i, row) in q.rows.iter().enumerate() {
                                let row_struct = CfmlValue::Struct(row.clone());
                                let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), q_val.clone()];
                                self.closure_parent_writeback = None;
                                let mapped = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
                                if let CfmlValue::Struct(s) = mapped {
                                    new_rows.push(s);
                                } else {
                                    new_rows.push(row.clone());
                                }
                            }
                            let mut result = q.clone();
                            result.rows = new_rows;
                            return Ok(CfmlValue::Query(result));
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "queryfilter" => {
                    if let (Some(q_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Query(q) = q_val {
                            let callback = callback.clone();
                            let mut new_rows = Vec::new();
                            for (i, row) in q.rows.iter().enumerate() {
                                let row_struct = CfmlValue::Struct(row.clone());
                                let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), q_val.clone()];
                                self.closure_parent_writeback = None;
                                let keep = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
                                if keep.is_true() {
                                    new_rows.push(row.clone());
                                }
                            }
                            let mut result = q.clone();
                            result.rows = new_rows;
                            return Ok(CfmlValue::Query(result));
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "queryreduce" => {
                    if let (Some(q_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Query(q) = q_val {
                            let mut acc = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                            let callback = callback.clone();
                            for (i, row) in q.rows.iter().enumerate() {
                                let row_struct = CfmlValue::Struct(row.clone());
                                let cb_args = vec![acc.clone(), row_struct, CfmlValue::Int((i + 1) as i64), q_val.clone()];
                                self.closure_parent_writeback = None;
                                acc = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
                            }
                            return Ok(acc);
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "querysort" => {
                    if let (Some(q_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Query(q) = q_val {
                            let callback = callback.clone();
                            let mut rows = q.rows.clone();
                            // Bubble sort (closure calls can't be used with sort_by)
                            let n = rows.len();
                            for i in 0..n {
                                for j in 0..n - 1 - i {
                                    let a = CfmlValue::Struct(rows[j].clone());
                                    let b = CfmlValue::Struct(rows[j + 1].clone());
                                    let cb_args = vec![a, b];
                                    self.closure_parent_writeback = None;
                                    let cmp = self.call_function(&callback, cb_args, parent_locals)?;
                                    if let Some(ref wb) = self.closure_parent_writeback {
                                        Self::write_back_to_captured_scope(&callback, wb);
                                    }
                                    let cmp_val = match &cmp {
                                        CfmlValue::Int(n) => *n,
                                        CfmlValue::Double(d) => *d as i64,
                                        _ => 0,
                                    };
                                    if cmp_val > 0 {
                                        rows.swap(j, j + 1);
                                    }
                                }
                            }
                            let mut result = q.clone();
                            result.rows = rows;
                            return Ok(CfmlValue::Query(result));
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "querysome" => {
                    if let (Some(q_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Query(q) = q_val {
                            let callback = callback.clone();
                            for (i, row) in q.rows.iter().enumerate() {
                                let row_struct = CfmlValue::Struct(row.clone());
                                let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), q_val.clone()];
                                self.closure_parent_writeback = None;
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
                                if result.is_true() {
                                    return Ok(CfmlValue::Bool(true));
                                }
                            }
                            return Ok(CfmlValue::Bool(false));
                        }
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "queryevery" => {
                    if let (Some(q_val), Some(callback)) = (args.get(0), args.get(1)) {
                        if let CfmlValue::Query(q) = q_val {
                            let callback = callback.clone();
                            for (i, row) in q.rows.iter().enumerate() {
                                let row_struct = CfmlValue::Struct(row.clone());
                                let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), q_val.clone()];
                                self.closure_parent_writeback = None;
                                let result = self.call_function(&callback, cb_args, parent_locals)?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
                                if !result.is_true() {
                                    return Ok(CfmlValue::Bool(false));
                                }
                            }
                            return Ok(CfmlValue::Bool(true));
                        }
                    }
                    return Ok(CfmlValue::Bool(true));
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
                "expandpath" => {
                    // CFML expandPath: resolve relative to current template dir,
                    // absolute paths (starting with /) resolve via mappings then source dir
                    let rel = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let base_dir = self.source_file.as_ref()
                        .and_then(|s| std::path::Path::new(s).parent())
                        .unwrap_or_else(|| std::path::Path::new("."));

                    let resolved = if rel.starts_with('/') {
                        // Try mappings first
                        let mut found = None;
                        for mapping in &self.mappings {
                            let prefix = mapping.name.trim_end_matches('/');
                            if rel.to_lowercase().starts_with(&prefix.to_lowercase()) {
                                let remainder = &rel[prefix.len()..];
                                let remainder = remainder.trim_start_matches('/');
                                let candidate = std::path::PathBuf::from(&mapping.path).join(remainder);
                                found = Some(candidate);
                                break;
                            }
                        }
                        found.unwrap_or_else(|| base_dir.join(rel.trim_start_matches('/')))
                    } else {
                        base_dir.join(&rel)
                    };

                    // Canonicalize if it exists, otherwise return the joined path
                    let result = std::fs::canonicalize(&resolved)
                        .unwrap_or(resolved);
                    return Ok(CfmlValue::String(result.to_string_lossy().to_string()));
                }
                "isdefined" => {
                    // Runtime isDefined: argument is a string variable name
                    let var_name = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let defined = self.is_variable_defined(&var_name, parent_locals);
                    return Ok(CfmlValue::Bool(defined));
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
                                let mut meta = IndexMap::new();
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
                                            let mut func_meta = IndexMap::new();
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
                    return Ok(CfmlValue::Struct(IndexMap::new()));
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
                "__cfabort" => {
                    return Err(CfmlError::new(
                        "__cfabort".to_string(),
                        CfmlErrorType::Custom("abort".to_string()),
                    ));
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
                    let mut empty = IndexMap::new();
                    empty.insert("headers".to_string(), CfmlValue::Struct(IndexMap::new()));
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
                "queryexecute" => {
                    // VM intercept for queryExecute — routes through transaction conn if active
                    if self.transaction_conn.is_some() {
                        if let Some(txn_execute) = self.txn_execute {
                            let sql = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                            let params_arg = args.get(1).cloned().unwrap_or(CfmlValue::Null);
                            let options_arg = args.get(2).cloned().unwrap_or(CfmlValue::Null);
                            let return_type = match &options_arg {
                                CfmlValue::Struct(opts) => opts.iter()
                                    .find(|(k, _)| k.eq_ignore_ascii_case("returntype"))
                                    .map(|(_, v)| v.as_string().to_lowercase())
                                    .unwrap_or_else(|| "query".to_string()),
                                _ => "query".to_string(),
                            };
                            let txn_conn = self.transaction_conn.as_mut().unwrap();
                            return txn_execute(txn_conn, &sql, &params_arg, &return_type);
                        }
                    }
                    // No active transaction — delegate to normal builtin (via fn pointer or registered builtin)
                    if let Some(qe_fn) = self.query_execute_fn {
                        return qe_fn(args);
                    }
                    // Fall through to normal builtin dispatch
                    let builtin_match = self.builtins.iter()
                        .find(|(k, _)| k.to_lowercase() == "queryexecute")
                        .map(|(_, v)| *v);
                    if let Some(builtin) = builtin_match {
                        return builtin(args);
                    }
                    return Err(CfmlError::runtime("queryExecute: database features not enabled".to_string()));
                }
                "__cftransaction_start" => {
                    if self.transaction_conn.is_some() {
                        return Err(CfmlError::runtime("cftransaction: nested transactions are not supported".to_string()));
                    }
                    // Args: __cftransaction_start("begin", [isolation], [datasource])
                    // Try arg[2] first (datasource after isolation), then arg[1] (datasource without isolation)
                    let datasource = args.get(2).map(|v| v.as_string()).filter(|s| !s.is_empty())
                        .or_else(|| args.get(1).map(|v| v.as_string()).filter(|s| !s.is_empty() && s != "begin"))
                        .unwrap_or_else(|| self.get_default_datasource(parent_locals));
                    if datasource.is_empty() {
                        return Err(CfmlError::runtime("cftransaction: no datasource specified and no default datasource configured".to_string()));
                    }
                    if let Some(txn_begin) = self.txn_begin {
                        let conn = txn_begin(&datasource)?;
                        self.transaction_conn = Some(conn);
                        self.transaction_datasource = Some(datasource);
                        return Ok(CfmlValue::Null);
                    }
                    return Err(CfmlError::runtime("cftransaction: transaction support not initialized".to_string()));
                }
                "__cftransaction_commit" => {
                    if let Some(ref mut conn) = self.transaction_conn {
                        if let Some(txn_commit) = self.txn_commit {
                            txn_commit(conn)?;
                        }
                    }
                    self.transaction_conn = None;
                    self.transaction_datasource = None;
                    return Ok(CfmlValue::Null);
                }
                "__cftransaction_rollback" => {
                    if let Some(ref mut conn) = self.transaction_conn {
                        if let Some(txn_rollback) = self.txn_rollback {
                            txn_rollback(conn)?;
                        }
                    }
                    self.transaction_conn = None;
                    self.transaction_datasource = None;
                    return Ok(CfmlValue::Null);
                }
                "__cflog" => {
                    // Extract log message from struct argument
                    if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        let text = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "text")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_default();
                        let log_type = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "type")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_else(|| "Information".to_string());
                        let file = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "file")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_else(|| "application".to_string());
                        eprintln!("[CFLOG {}:{}] {}", file, log_type, text);
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cfsetting" => {
                    // Handle cfsetting options
                    if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        let _show_debug = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "showdebugoutput")
                            .map(|(_,v)| v.as_string().to_lowercase() == "true" || v.as_string() == "yes");
                        let _request_timeout = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "requesttimeout")
                            .map(|(_, v)| v.as_string());
                        let enable_output = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "enablecfoutputonly")
                            .map(|(_,v)| v.as_string().to_lowercase() == "true" || v.as_string() == "yes");
                        if let Some(_enabled) = enable_output {
                            // Would control output suppression - stub for now
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cflock_start" => {
                    // Extract lock attributes from struct argument
                    let (lock_name, lock_type, timeout_ms) = if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        let name = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "name")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_else(|| "default".to_string());
                        let ltype = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "type")
                            .map(|(_, v)| v.as_string().to_lowercase())
                            .unwrap_or_else(|| "exclusive".to_string());
                        let timeout = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "timeout")
                            .and_then(|(_, v)| match v {
                                CfmlValue::Int(i) => Some(*i as u64 * 1000),
                                CfmlValue::Double(d) => Some((*d * 1000.0) as u64),
                                CfmlValue::String(s) => s.parse::<f64>().ok().map(|d| (d * 1000.0) as u64),
                                _ => None,
                            })
                            .unwrap_or(5000);
                        (name, ltype, timeout)
                    } else {
                        // Positional args: name, type, timeout
                        let name = args.get(0).map(|v| v.as_string()).unwrap_or_else(|| "default".to_string());
                        let ltype = args.get(1).map(|v| v.as_string().to_lowercase()).unwrap_or_else(|| "exclusive".to_string());
                        let timeout = args.get(2).and_then(|v| match v {
                            CfmlValue::Int(i) => Some(*i as u64 * 1000),
                            CfmlValue::Double(d) => Some((*d * 1000.0) as u64),
                            CfmlValue::String(s) => s.parse::<f64>().ok().map(|d| (d * 1000.0) as u64),
                            _ => None,
                        }).unwrap_or(5000);
                        (name, ltype, timeout)
                    };

                    if let Some(ref server_state) = self.server_state {
                        // Get or create the named lock
                        let lock = {
                            let mut locks = server_state.named_locks.lock().unwrap();
                            locks.entry(lock_name.clone()).or_insert_with(|| Arc::new(RwLock::new(()))).clone()
                        };

                        // Acquire lock with timeout using try_lock in a spin loop
                        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
                        let is_exclusive = lock_type != "readonly";

                        if is_exclusive {
                            loop {
                                if let Ok(guard) = lock.try_write() {
                                    // SAFETY: We extend the lifetime because the Arc keeps the RwLock alive.
                                    // The guard is dropped in __cflock_end before the Arc can be dropped.
                                    let guard: std::sync::RwLockWriteGuard<'static, ()> = unsafe {
                                        std::mem::transmute(guard)
                                    };
                                    self.held_locks.push((lock_name, HeldLock::Write(guard)));
                                    break;
                                }
                                if std::time::Instant::now() >= deadline {
                                    return Err(CfmlError::runtime(
                                        format!("cflock timeout: could not acquire exclusive lock within {}ms", timeout_ms)
                                    ));
                                }
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            }
                        } else {
                            loop {
                                if let Ok(guard) = lock.try_read() {
                                    let guard: std::sync::RwLockReadGuard<'static, ()> = unsafe {
                                        std::mem::transmute(guard)
                                    };
                                    self.held_locks.push((lock_name, HeldLock::Read(guard)));
                                    break;
                                }
                                if std::time::Instant::now() >= deadline {
                                    return Err(CfmlError::runtime(
                                        format!("cflock timeout: could not acquire readonly lock within {}ms", timeout_ms)
                                    ));
                                }
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            }
                        }
                    }
                    // Without server_state (CLI mode), locks are a no-op
                    return Ok(CfmlValue::Null);
                }
                "__cflock_end" => {
                    // Release the most recently acquired lock
                    // Args may contain the lock name for matching
                    let lock_name = if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "name")
                            .map(|(_, v)| v.as_string())
                    } else {
                        args.get(0).map(|v| v.as_string())
                    };

                    if let Some(name) = lock_name {
                        // Find and remove the matching lock guard
                        if let Some(pos) = self.held_locks.iter().rposition(|(n, _)| *n == name) {
                            self.held_locks.remove(pos);
                        }
                    } else {
                        // Pop the most recent lock
                        self.held_locks.pop();
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cfcookie" => {
                    // Set a cookie via response headers
                    if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        let name = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "name")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_default();
                        let value = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "value")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_default();
                        let mut cookie = format!("{}={}", name, value);
                        if let Some((_, expires)) = opts.iter().find(|(k, _)| k.to_lowercase() == "expires") {
                            cookie.push_str(&format!("; Expires={}", expires.as_string()));
                        }
                        if let Some((_, domain)) = opts.iter().find(|(k, _)| k.to_lowercase() == "domain") {
                            cookie.push_str(&format!("; Domain={}", domain.as_string()));
                        }
                        if let Some((_, path)) = opts.iter().find(|(k, _)| k.to_lowercase() == "path") {
                            cookie.push_str(&format!("; Path={}", path.as_string()));
                        }
                        if let Some((_, secure)) = opts.iter().find(|(k, _)| k.to_lowercase() == "secure") {
                            if secure.as_string().to_lowercase() == "true" || secure.as_string() == "yes" {
                                cookie.push_str("; Secure");
                            }
                        }
                        if let Some((_, httponly)) = opts.iter().find(|(k, _)| k.to_lowercase() == "httponly") {
                            if httponly.as_string().to_lowercase() == "true" || httponly.as_string() == "yes" {
                                cookie.push_str("; HttpOnly");
                            }
                        }
                        self.response_headers.push(("Set-Cookie".to_string(), cookie));
                    }
                    return Ok(CfmlValue::Null);
                }
                "fileupload" | "__cffile_upload" => {
                    // fileUpload(destination, formField, accept, nameConflict)
                    let destination = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let form_field = args.get(1).map(|v| v.as_string()).unwrap_or_default();
                    let _accept = args.get(2).map(|v| v.as_string()).unwrap_or_default();
                    let name_conflict = args.get(3).map(|v| v.as_string().to_lowercase()).unwrap_or_else(|| "error".to_string());

                    // Look up the form field to find uploaded file info
                    let form_scope = self.globals.get("form")
                        .cloned()
                        .unwrap_or(CfmlValue::Struct(IndexMap::new()));

                    if let CfmlValue::Struct(form) = form_scope {
                        let field_lower = form_field.to_lowercase();
                        if let Some(CfmlValue::Struct(file_info)) = form.iter()
                            .find(|(k, _)| k.to_lowercase() == field_lower)
                            .map(|(_, v)| v)
                        {
                            let temp_path = file_info.iter()
                                .find(|(k, _)| k.to_lowercase() == "tempfilepath")
                                .map(|(_, v)| v.as_string())
                                .unwrap_or_default();
                            let client_file = file_info.iter()
                                .find(|(k, _)| k.to_lowercase() == "clientfile")
                                .map(|(_, v)| v.as_string())
                                .unwrap_or_default();

                            if !temp_path.is_empty() {
                                let dest_dir = std::path::Path::new(&destination);
                                let _ = std::fs::create_dir_all(dest_dir);
                                let dest_file = dest_dir.join(&client_file);

                                let final_path = if dest_file.exists() && name_conflict == "makeunique" {
                                    let stem = dest_file.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                                    let ext = dest_file.extension().map(|s| format!(".{}", s.to_string_lossy())).unwrap_or_default();
                                    let unique = dest_dir.join(format!("{}_{}{}", stem, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(), ext));
                                    unique
                                } else {
                                    dest_file
                                };

                                match std::fs::copy(&temp_path, &final_path) {
                                    Ok(_) => {
                                        let _ = std::fs::remove_file(&temp_path);
                                        let mut result = file_info.clone();
                                        result.insert("serverDirectory".to_string(), CfmlValue::String(destination));
                                        result.insert("serverFile".to_string(), CfmlValue::String(final_path.file_name().unwrap_or_default().to_string_lossy().to_string()));
                                        result.insert("fileWasSaved".to_string(), CfmlValue::Bool(true));
                                        return Ok(CfmlValue::Struct(result));
                                    }
                                    Err(e) => return Err(CfmlError::runtime(format!("fileUpload: {}", e))),
                                }
                            }
                        }
                    }
                    return Err(CfmlError::runtime(format!("fileUpload: form field '{}' not found or no file uploaded", form_field)));
                }
                "fileuploadall" => {
                    // fileUploadAll(destination, accept, nameConflict)
                    let destination = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let _accept = args.get(1).map(|v| v.as_string()).unwrap_or_default();
                    let name_conflict = args.get(2).map(|v| v.as_string().to_lowercase()).unwrap_or_else(|| "error".to_string());

                    let form_scope = self.globals.get("form")
                        .cloned()
                        .unwrap_or(CfmlValue::Struct(IndexMap::new()));

                    let mut results = Vec::new();
                    if let CfmlValue::Struct(form) = form_scope {
                        for (_, val) in &form {
                            if let CfmlValue::Struct(file_info) = val {
                                let temp_path = file_info.iter()
                                    .find(|(k, _)| k.to_lowercase() == "tempfilepath")
                                    .map(|(_, v)| v.as_string())
                                    .unwrap_or_default();
                                if temp_path.is_empty() { continue; }

                                let client_file = file_info.iter()
                                    .find(|(k, _)| k.to_lowercase() == "clientfile")
                                    .map(|(_, v)| v.as_string())
                                    .unwrap_or_default();

                                let dest_dir = std::path::Path::new(&destination);
                                let _ = std::fs::create_dir_all(dest_dir);
                                let dest_file = dest_dir.join(&client_file);

                                let final_path = if dest_file.exists() && name_conflict == "makeunique" {
                                    let stem = dest_file.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                                    let ext = dest_file.extension().map(|s| format!(".{}", s.to_string_lossy())).unwrap_or_default();
                                    let unique = dest_dir.join(format!("{}_{}{}", stem, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(), ext));
                                    unique
                                } else {
                                    dest_file
                                };

                                if let Ok(_) = std::fs::copy(&temp_path, &final_path) {
                                    let _ = std::fs::remove_file(&temp_path);
                                    let mut result = file_info.clone();
                                    result.insert("serverDirectory".to_string(), CfmlValue::String(destination.clone()));
                                    result.insert("serverFile".to_string(), CfmlValue::String(final_path.file_name().unwrap_or_default().to_string_lossy().to_string()));
                                    result.insert("fileWasSaved".to_string(), CfmlValue::Bool(true));
                                    results.push(CfmlValue::Struct(result));
                                }
                            }
                        }
                    }
                    return Ok(CfmlValue::Array(results));
                }
                "sessioninvalidate" => {
                    if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
                        if let Ok(mut sessions) = state.sessions.lock() {
                            sessions.remove(sid);
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "sessionrotate" => {
                    // Generate a new session ID and migrate data
                    if let (Some(ref state), Some(ref old_sid)) = (&self.server_state, &self.session_id) {
                        let new_sid = {
                            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
                            format!("{:x}", ts)
                        };
                        if let Ok(mut sessions) = state.sessions.lock() {
                            if let Some(old_data) = sessions.remove(old_sid) {
                                sessions.insert(new_sid.clone(), old_data);
                            }
                        }
                        self.session_id = Some(new_sid);
                    }
                    return Ok(CfmlValue::Null);
                }
                "sessiongetmetadata" => {
                    let mut meta = IndexMap::new();
                    if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
                        if let Ok(sessions) = state.sessions.lock() {
                            if let Some(session) = sessions.get(sid) {
                                meta.insert("sessionId".to_string(), CfmlValue::String(sid.clone()));
                                meta.insert("timeCreated".to_string(), CfmlValue::Int(session.created.elapsed().as_secs() as i64));
                                meta.insert("lastAccessed".to_string(), CfmlValue::Int(session.last_accessed.elapsed().as_secs() as i64));
                            }
                        }
                    }
                    return Ok(CfmlValue::Struct(meta));
                }
                "getauthuser" => {
                    if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
                        if let Ok(sessions) = state.sessions.lock() {
                            if let Some(session) = sessions.get(sid) {
                                if let Some(ref user) = session.auth_user {
                                    return Ok(CfmlValue::String(user.clone()));
                                }
                            }
                        }
                    }
                    return Ok(CfmlValue::String(String::new()));
                }
                "isuserloggedin" => {
                    if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
                        if let Ok(sessions) = state.sessions.lock() {
                            if let Some(session) = sessions.get(sid) {
                                return Ok(CfmlValue::Bool(session.auth_user.is_some()));
                            }
                        }
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "isuserinrole" => {
                    let role = args.get(0).map(|v| v.as_string().to_lowercase()).unwrap_or_default();
                    if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
                        if let Ok(sessions) = state.sessions.lock() {
                            if let Some(session) = sessions.get(sid) {
                                let has_role = session.auth_roles.iter().any(|r| r.to_lowercase() == role);
                                return Ok(CfmlValue::Bool(has_role));
                            }
                        }
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "__cfloginuser" => {
                    // cfloginuser name="..." password="..." roles="..."
                    let name = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let roles_str = args.get(2).map(|v| v.as_string()).unwrap_or_default();
                    let roles: Vec<String> = roles_str.split(',').map(|r| r.trim().to_string()).filter(|r| !r.is_empty()).collect();
                    if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
                        if let Ok(mut sessions) = state.sessions.lock() {
                            if let Some(session) = sessions.get_mut(sid) {
                                session.auth_user = Some(name);
                                session.auth_roles = roles;
                            } else {
                                sessions.insert(sid.clone(), SessionData {
                                    variables: IndexMap::new(),
                                    created: std::time::Instant::now(),
                                    last_accessed: std::time::Instant::now(),
                                    auth_user: Some(name),
                                    auth_roles: roles,
                                    timeout_secs: 1800,
                                });
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cflogout" => {
                    if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
                        if let Ok(mut sessions) = state.sessions.lock() {
                            if let Some(session) = sessions.get_mut(sid) {
                                session.auth_user = None;
                                session.auth_roles.clear();
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "getvariable" => {
                    // getVariable(name) — walk scope chain to find variable
                    let var_name = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let var_lower = var_name.to_lowercase();

                    // Handle dotted names like "variables.foo" or "request.bar"
                    if var_lower.contains('.') {
                        let parts: Vec<&str> = var_lower.splitn(2, '.').collect();
                        let scope_name = parts[0];
                        let key = parts.get(1).copied().unwrap_or("");
                        match scope_name {
                            "request" => {
                                if let Some(val) = self.request_scope.iter()
                                    .find(|(k, _)| k.to_lowercase() == key)
                                    .map(|(_, v)| v.clone())
                                {
                                    return Ok(val);
                                }
                                return Ok(CfmlValue::Null);
                            }
                            "session" => {
                                if let CfmlValue::Struct(s) = self.get_session_scope() {
                                    if let Some(val) = s.iter()
                                        .find(|(k, _)| k.to_lowercase() == key)
                                        .map(|(_, v)| v.clone())
                                    {
                                        return Ok(val);
                                    }
                                }
                                return Ok(CfmlValue::Null);
                            }
                            "application" => {
                                if let Some(ref app_scope) = self.application_scope {
                                    if let Ok(scope) = app_scope.lock() {
                                        if let Some(val) = scope.iter()
                                            .find(|(k, _)| k.to_lowercase() == key)
                                            .map(|(_, v)| v.clone())
                                        {
                                            return Ok(val);
                                        }
                                    }
                                }
                                return Ok(CfmlValue::Null);
                            }
                            _ => {}
                        }
                    }

                    // Check parent_locals
                    if let Some(val) = parent_locals.iter()
                        .find(|(k, _)| k.to_lowercase() == var_lower)
                        .map(|(_, v)| v.clone())
                    {
                        return Ok(val);
                    }
                    // Request scope
                    if let Some(val) = self.request_scope.iter()
                        .find(|(k, _)| k.to_lowercase() == var_lower)
                        .map(|(_, v)| v.clone())
                    {
                        return Ok(val);
                    }
                    // Session scope
                    if let CfmlValue::Struct(s) = self.get_session_scope() {
                        if let Some(val) = s.iter()
                            .find(|(k, _)| k.to_lowercase() == var_lower)
                            .map(|(_, v)| v.clone())
                        {
                            return Ok(val);
                        }
                    }
                    // Application scope
                    if let Some(ref app_scope) = self.application_scope {
                        if let Ok(scope) = app_scope.lock() {
                            if let Some(val) = scope.iter()
                                .find(|(k, _)| k.to_lowercase() == var_lower)
                                .map(|(_, v)| v.clone())
                            {
                                return Ok(val);
                            }
                        }
                    }
                    // Globals
                    if let Some(val) = self.globals.iter()
                        .find(|(k, _)| k.to_lowercase() == var_lower)
                        .map(|(_, v)| v.clone())
                    {
                        return Ok(val);
                    }
                    return Ok(CfmlValue::Null);
                }
                "setvariable" => {
                    // setVariable(name, value) — set a variable by dynamic name, return value
                    let var_name = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let value = args.get(1).cloned().unwrap_or(CfmlValue::Null);

                    // Handle dotted scope names
                    let var_lower = var_name.to_lowercase();
                    if var_lower.starts_with("variables.") {
                        let key = var_name[10..].to_string();
                        self.globals.insert(key, value.clone());
                    } else if var_lower.starts_with("request.") {
                        let key = var_name[8..].to_string();
                        self.request_scope.insert(key, value.clone());
                    } else if var_lower.starts_with("session.") {
                        let key = var_name[8..].to_string();
                        self.set_session_variable(&key, value.clone());
                    } else if var_lower.starts_with("application.") {
                        let key = var_name[12..].to_string();
                        if let Some(ref app_scope) = self.application_scope {
                            if let Ok(mut scope) = app_scope.lock() {
                                scope.insert(key, value.clone());
                            }
                        }
                    } else {
                        // Default: set in variables (globals) scope
                        self.globals.insert(var_name, value.clone());
                    }
                    return Ok(value);
                }
                "throw" => {
                    // throw(message="...", type="...", detail="...", errorcode="...")
                    // Build exception struct from named args or positional
                    let mut exception = IndexMap::new();
                    let message = args.get(0).map(|v| v.as_string()).unwrap_or_else(|| "".to_string());
                    let error_type = args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "Application".to_string());
                    let detail = args.get(2).map(|v| v.as_string()).unwrap_or_default();
                    let errorcode = args.get(3).map(|v| v.as_string()).unwrap_or_default();

                    exception.insert("message".to_string(), CfmlValue::String(message.clone()));
                    exception.insert("type".to_string(), CfmlValue::String(error_type));
                    exception.insert("detail".to_string(), CfmlValue::String(detail));
                    exception.insert("errorcode".to_string(), CfmlValue::String(errorcode));
                    exception.insert("tagcontext".to_string(), self.build_tag_context());

                    let error_val = CfmlValue::Struct(exception);
                    self.last_exception = Some(error_val.clone());

                    return Err(CfmlError::runtime(message));
                }
                // ---- Cache functions ----
                "cacheput" => {
                    let key = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let value = args.get(1).cloned().unwrap_or(CfmlValue::Null);
                    let expiry = args.get(2).and_then(|v| {
                        // Timespan: value < 1 treated as fractional days (×86400→secs)
                        let secs = match v {
                            CfmlValue::Int(i) => *i as f64,
                            CfmlValue::Double(d) => {
                                if *d < 1.0 { *d * 86400.0 } else { *d }
                            }
                            CfmlValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
                            _ => 0.0,
                        };
                        if secs > 0.0 {
                            Some(std::time::Instant::now() + std::time::Duration::from_secs_f64(secs))
                        } else {
                            None
                        }
                    });
                    self.cache.insert(key, (value, expiry));
                    return Ok(CfmlValue::Null);
                }
                "cacheget" => {
                    let key = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    if let Some((val, expiry)) = self.cache.get(&key).cloned() {
                        if let Some(exp) = expiry {
                            if std::time::Instant::now() > exp {
                                self.cache.remove(&key);
                                return Ok(CfmlValue::Null);
                            }
                        }
                        return Ok(val);
                    }
                    return Ok(CfmlValue::Null);
                }
                "cachedelete" => {
                    let key = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let throw_on_error = args.get(1).map(|v| match v {
                        CfmlValue::Bool(b) => *b,
                        CfmlValue::String(s) => s.to_lowercase() == "true" || s.to_lowercase() == "yes",
                        _ => false,
                    }).unwrap_or(false);
                    if self.cache.remove(&key).is_none() && throw_on_error {
                        return Err(CfmlError::runtime(format!("Cache key '{}' does not exist", key)));
                    }
                    return Ok(CfmlValue::Null);
                }
                "cacheclear" => {
                    let filter = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    if filter.is_empty() {
                        self.cache.clear();
                    } else {
                        // Simple wildcard matching: * matches any sequence
                        let pattern = filter.to_lowercase();
                        let keys_to_remove: Vec<String> = self.cache.keys()
                            .filter(|k| wildcard_match(&pattern, &k.to_lowercase()))
                            .cloned()
                            .collect();
                        for k in keys_to_remove {
                            self.cache.remove(&k);
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "cachekeyexists" => {
                    let key = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    if let Some((_, expiry)) = self.cache.get(&key) {
                        if let Some(exp) = expiry {
                            if std::time::Instant::now() > *exp {
                                self.cache.remove(&key);
                                return Ok(CfmlValue::Bool(false));
                            }
                        }
                        return Ok(CfmlValue::Bool(true));
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "cachecount" => {
                    let now = std::time::Instant::now();
                    let count = self.cache.iter()
                        .filter(|(_, (_, exp))| exp.map_or(true, |e| now <= e))
                        .count();
                    return Ok(CfmlValue::Int(count as i64));
                }
                "cachegetall" => {
                    let now = std::time::Instant::now();
                    let mut result = IndexMap::new();
                    for (k, (v, exp)) in &self.cache {
                        if exp.map_or(true, |e| now <= e) {
                            result.insert(k.clone(), v.clone());
                        }
                    }
                    return Ok(CfmlValue::Struct(result));
                }
                "cachegetallids" => {
                    let now = std::time::Instant::now();
                    let ids: Vec<CfmlValue> = self.cache.iter()
                        .filter(|(_, (_, exp))| exp.map_or(true, |e| now <= e))
                        .map(|(k, _)| CfmlValue::String(k.clone()))
                        .collect();
                    return Ok(CfmlValue::Array(ids));
                }

                // ---- cfcache tag handler ----
                "__cfcache" => {
                    // Stub/no-op; in serve mode could push Cache-Control header
                    return Ok(CfmlValue::Null);
                }

                // ---- cfexecute tag handler ----
                "__cfexecute" => {
                    if let Some(CfmlValue::Struct(opts)) = args.get(0) {
                        let cmd_name = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "name")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_default();
                        let arguments = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "arguments")
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_default();
                        let has_variable = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "variable")
                            .map(|(_,v)| match v {
                                CfmlValue::Bool(b) => *b,
                                CfmlValue::String(s) => s.to_lowercase() == "true",
                                _ => false,
                            })
                            .unwrap_or(false);
                        let body = opts.iter()
                            .find(|(k, _)| k.to_lowercase() == "body")
                            .map(|(_, v)| v.as_string());

                        let cmd_args: Vec<&str> = if arguments.is_empty() {
                            Vec::new()
                        } else {
                            arguments.split_whitespace().collect()
                        };

                        let mut command = std::process::Command::new(&cmd_name);
                        command.args(&cmd_args);
                        if body.is_some() {
                            command.stdin(std::process::Stdio::piped());
                        }
                        command.stdout(std::process::Stdio::piped());
                        command.stderr(std::process::Stdio::piped());

                        match command.spawn() {
                            Ok(mut child) => {
                                if let Some(ref stdin_data) = body {
                                    if let Some(ref mut stdin) = child.stdin {
                                        use std::io::Write;
                                        let _ = stdin.write_all(stdin_data.as_bytes());
                                    }
                                    // Drop stdin to signal EOF
                                    child.stdin.take();
                                }
                                match child.wait_with_output() {
                                    Ok(output) => {
                                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                                        if has_variable {
                                            let mut result = IndexMap::new();
                                            result.insert("output".to_string(), CfmlValue::String(stdout));
                                            result.insert("error".to_string(), CfmlValue::String(stderr));
                                            return Ok(CfmlValue::Struct(result));
                                        } else {
                                            self.output_buffer.push_str(&stdout);
                                            return Ok(CfmlValue::Null);
                                        }
                                    }
                                    Err(e) => {
                                        return Err(CfmlError::runtime(format!("cfexecute: {}", e)));
                                    }
                                }
                            }
                            Err(e) => {
                                return Err(CfmlError::runtime(format!("cfexecute: failed to spawn '{}': {}", cmd_name, e)));
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }

                // ---- cfthread handlers ----
                "__cfthread_run" => {
                    let thread_name = args.get(0).map(|v| v.as_string()).unwrap_or_else(|| "thread1".to_string());
                    if let Some(callback) = args.get(1) {
                        let callback = callback.clone();
                        // Execute the closure immediately (sequential execution model)
                        let cb_args = vec![];
                        let _ = self.call_function(&callback, cb_args, parent_locals);
                    }
                    // Store thread metadata as completed
                    let mut thread_meta = IndexMap::new();
                    thread_meta.insert("status".to_string(), CfmlValue::String("COMPLETED".to_string()));
                    thread_meta.insert("name".to_string(), CfmlValue::String(thread_name.clone()));
                    thread_meta.insert("output".to_string(), CfmlValue::String(String::new()));
                    thread_meta.insert("error".to_string(), CfmlValue::String(String::new()));
                    thread_meta.insert("elapsedtime".to_string(), CfmlValue::Int(0));
                    // Store in cfthread scope (on variables scope)
                    let thread_struct = self.get_or_create_cfthread_scope();
                    if let CfmlValue::Struct(ref mut ts) = thread_struct {
                        ts.insert(thread_name.to_lowercase(), CfmlValue::Struct(thread_meta));
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cfthread_join" => {
                    // No-op since execution already happened (thread is already complete)
                    return Ok(CfmlValue::Null);
                }
                "__cfthread_terminate" => {
                    // No-op (thread already finished)
                    return Ok(CfmlValue::Null);
                }

                "__cfcustomtag" => {
                    // Self-closing custom tag: __cfcustomtag(path_spec, attrs_struct)
                    let path_spec = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let attrs_val = args.get(1).cloned().unwrap_or(CfmlValue::Struct(IndexMap::new()));

                    let resolved = self.resolve_custom_tag_path(&path_spec)?;
                    let mut this_tag = IndexMap::new();
                    this_tag.insert("executionmode".to_string(), CfmlValue::String("start".to_string()));
                    this_tag.insert("hasendtag".to_string(), CfmlValue::Bool(false));
                    this_tag.insert("generatedcontent".to_string(), CfmlValue::String(String::new()));

                    let caller_snapshot = parent_locals.clone();
                    let mut tag_locals = IndexMap::new();
                    tag_locals.insert("attributes".to_string(), attrs_val);
                    tag_locals.insert("caller".to_string(), CfmlValue::Struct(caller_snapshot.clone()));
                    tag_locals.insert("thistag".to_string(), CfmlValue::Struct(this_tag));

                    self.execute_custom_tag_template(&resolved, &tag_locals)?;

                    // Caller write-back: read modified caller from captured_locals
                    if let Some(ref captured) = self.captured_locals {
                        if let Some(CfmlValue::Struct(modified_caller)) = captured.get("caller") {
                            let mut wb = IndexMap::new();
                            for (k, v) in modified_caller {
                                if let Some(orig) = caller_snapshot.get(k) {
                                    if !Self::values_equal_shallow(v, orig) {
                                        wb.insert(k.clone(), v.clone());
                                    }
                                } else {
                                    wb.insert(k.clone(), v.clone());
                                }
                            }
                            if !wb.is_empty() {
                                self.closure_parent_writeback = Some(wb);
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "__cfcustomtag_start" => {
                    // Body custom tag start: __cfcustomtag_start(path_spec, attrs_struct)
                    let path_spec = args.get(0).map(|v| v.as_string()).unwrap_or_default();
                    let attrs_val = args.get(1).cloned().unwrap_or(CfmlValue::Struct(IndexMap::new()));

                    let resolved = self.resolve_custom_tag_path(&path_spec)?;

                    let mut this_tag = IndexMap::new();
                    this_tag.insert("executionmode".to_string(), CfmlValue::String("start".to_string()));
                    this_tag.insert("hasendtag".to_string(), CfmlValue::Bool(true));
                    this_tag.insert("generatedcontent".to_string(), CfmlValue::String(String::new()));

                    let caller_snapshot = parent_locals.clone();
                    let mut tag_locals = IndexMap::new();
                    tag_locals.insert("attributes".to_string(), attrs_val.clone());
                    tag_locals.insert("caller".to_string(), CfmlValue::Struct(caller_snapshot.clone()));
                    tag_locals.insert("thistag".to_string(), CfmlValue::Struct(this_tag));

                    self.execute_custom_tag_template(&resolved, &tag_locals)?;

                    // Caller write-back from start execution
                    if let Some(ref captured) = self.captured_locals {
                        if let Some(CfmlValue::Struct(modified_caller)) = captured.get("caller") {
                            let mut wb = IndexMap::new();
                            for (k, v) in modified_caller {
                                if let Some(orig) = caller_snapshot.get(k) {
                                    if !Self::values_equal_shallow(v, orig) {
                                        wb.insert(k.clone(), v.clone());
                                    }
                                } else {
                                    wb.insert(k.clone(), v.clone());
                                }
                            }
                            if !wb.is_empty() {
                                self.closure_parent_writeback = Some(wb);
                            }
                        }
                    }

                    // Push state for end tag
                    self.custom_tag_stack.push(CustomTagState {
                        template_path: resolved,
                        attributes: attrs_val,
                    });

                    // Push output buffer to capture body content (like savecontent)
                    self.saved_output_buffers.push(std::mem::take(&mut self.output_buffer));

                    return Ok(CfmlValue::Null);
                }
                "__cfcustomtag_end" => {
                    // Body custom tag end: capture body output, re-execute tag in "end" mode
                    let body_content = std::mem::take(&mut self.output_buffer);
                    self.output_buffer = self.saved_output_buffers.pop().unwrap_or_default();

                    let state = match self.custom_tag_stack.pop() {
                        Some(s) => s,
                        None => return Err(CfmlError::runtime("__cfcustomtag_end without matching start".to_string())),
                    };

                    let mut this_tag = IndexMap::new();
                    this_tag.insert("executionmode".to_string(), CfmlValue::String("end".to_string()));
                    this_tag.insert("hasendtag".to_string(), CfmlValue::Bool(true));
                    this_tag.insert("generatedcontent".to_string(), CfmlValue::String(body_content));

                    let caller_snapshot = parent_locals.clone();
                    let mut tag_locals = IndexMap::new();
                    tag_locals.insert("attributes".to_string(), state.attributes);
                    tag_locals.insert("caller".to_string(), CfmlValue::Struct(caller_snapshot.clone()));
                    tag_locals.insert("thistag".to_string(), CfmlValue::Struct(this_tag));

                    self.execute_custom_tag_template(&state.template_path, &tag_locals)?;

                    // Read back generatedContent and append to output
                    if let Some(ref captured) = self.captured_locals {
                        if let Some(CfmlValue::Struct(tag_info)) = captured.get("thistag") {
                            if let Some(CfmlValue::String(content)) = tag_info.get("generatedcontent") {
                                self.output_buffer.push_str(content);
                            }
                        }
                    }

                    // Caller write-back from end execution
                    if let Some(ref captured) = self.captured_locals {
                        if let Some(CfmlValue::Struct(modified_caller)) = captured.get("caller") {
                            let mut wb = IndexMap::new();
                            for (k, v) in modified_caller {
                                if let Some(orig) = caller_snapshot.get(k) {
                                    if !Self::values_equal_shallow(v, orig) {
                                        wb.insert(k.clone(), v.clone());
                                    }
                                } else {
                                    wb.insert(k.clone(), v.clone());
                                }
                            }
                            if !wb.is_empty() {
                                self.closure_parent_writeback = Some(wb);
                            }
                        }
                    }

                    return Ok(CfmlValue::Null);
                }
                _ => {}
            }
        }

        Err(self.wrap_error(CfmlError::runtime(
            format!("Variable is not a function or function '{}' is not defined",
                if let CfmlValue::Function(f) = func_ref { &f.name } else { "<unknown>" })
        )))
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
            "delete" | "insert" | "update" |
            // Query mutators
            "addrow" | "setcell" | "addcolumn" | "deleterow" | "deletecolumn"
        )
    }

    /// Set a value at an arbitrary depth in a nested struct.
    /// path = ["prop1", "prop2"] means set root.prop1.prop2 = value
    /// path = ["prop1"] means set root.prop1 = value
    fn deep_set(root: &mut CfmlValue, path: &[String], value: CfmlValue) {
        if path.is_empty() {
            return;
        }
        if path.len() == 1 {
            root.set(path[0].clone(), value);
            return;
        }
        // Recurse into the nested struct
        if let CfmlValue::Struct(ref mut s) = root {
            if let Some(child) = s.get_mut(&path[0]) {
                Self::deep_set(child, &path[1..], value);
            }
        }
    }

    /// Load a variable by name, checking locals, globals, and special scopes (application, request, local/variables).
    fn scope_aware_load(&self, name: &str, locals: &IndexMap<String, CfmlValue>) -> Option<CfmlValue> {
        let name_lower = name.to_lowercase();
        // "local" / "variables" → snapshot of all locals as a struct (mirrors LoadLocal behavior)
        if name_lower == "local" || name_lower == "variables" {
            return Some(CfmlValue::Struct(locals.clone()));
        }
        if name_lower == "application" {
            if let Some(ref app_scope) = self.application_scope {
                if let Ok(scope) = app_scope.lock() {
                    return Some(CfmlValue::Struct(scope.clone()));
                }
            }
        }
        if name_lower == "request" {
            return Some(CfmlValue::Struct(self.request_scope.clone()));
        }
        if let Some(v) = locals.get(name) {
            return Some(v.clone());
        }
        if let Some(v) = self.globals.get(name) {
            return Some(v.clone());
        }
        None
    }

    /// Store a variable by name, routing to the correct scope (locals, globals, application, request, local/variables).
    fn scope_aware_store(&mut self, name: &str, val: CfmlValue, locals: &mut IndexMap<String, CfmlValue>) {
        let name_lower = name.to_lowercase();
        // "local" / "variables" → merge keys back into locals (mirrors StoreLocal behavior)
        if name_lower == "local" || name_lower == "variables" {
            if let CfmlValue::Struct(s) = val {
                for (k, v) in s {
                    locals.insert(k, v);
                }
            }
        } else if name_lower == "application" {
            if let CfmlValue::Struct(s) = &val {
                if let Some(ref app_scope) = self.application_scope {
                    if let Ok(mut scope) = app_scope.lock() {
                        *scope = s.clone();
                    }
                }
            }
        } else if name_lower == "request" {
            if let CfmlValue::Struct(s) = &val {
                self.request_scope = s.clone();
            }
        } else if locals.contains_key(name) {
            locals.insert(name.to_string(), val);
        } else if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), val);
        } else {
            locals.insert(name.to_string(), val);
        }
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
                "ucfirst" => Some("ucFirst"),
                "lcfirst" => Some("lcFirst"),
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
                "findall" => Some("arrayFindAll"),
                "findallnocase" => Some("arrayFindAllNoCase"),
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
                            self.closure_parent_writeback = None;
                            let mapped = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            let keep = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            acc = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "some" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        for (i, item) in arr.iter().enumerate() {
                            let cb_args = vec![item.clone(), CfmlValue::Int((i + 1) as i64), object.clone()];
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "map" => {
                    // struct.map(callback) - callback(key, value, struct) returns new value
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut result = IndexMap::new();
                        for (k, v) in s {
                            let cb_args = vec![
                                CfmlValue::String(k.clone()),
                                v.clone(),
                                object.clone(),
                            ];
                            self.closure_parent_writeback = None;
                            let mapped = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            result.insert(k.clone(), mapped);
                        }
                        return Ok(CfmlValue::Struct(result));
                    }
                    return Ok(object.clone());
                }
                "filter" => {
                    // struct.filter(callback) - callback(key, value, struct) returns boolean
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut result = IndexMap::new();
                        for (k, v) in s {
                            let cb_args = vec![
                                CfmlValue::String(k.clone()),
                                v.clone(),
                                object.clone(),
                            ];
                            self.closure_parent_writeback = None;
                            let keep = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
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
                            self.closure_parent_writeback = None;
                            acc = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "tojson" | "serializejson" => Some("serializeJSON"),
                _ => None,
            },
            CfmlValue::Query(q) => match method_lower.as_str() {
                "recordcount" | "len" | "size" => {
                    return Ok(CfmlValue::Int(q.rows.len() as i64));
                }
                "columnlist" => {
                    return Ok(CfmlValue::String(q.columns.join(",")));
                }
                "addrow" => Some("queryAddRow"),
                "getrow" => Some("queryGetRow"),
                "each" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        for (i, row) in q.rows.iter().enumerate() {
                            let row_struct = CfmlValue::Struct(row.clone());
                            let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), object.clone()];
                            self.closure_parent_writeback = None;
                            self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                    }
                    return Ok(CfmlValue::Null);
                }
                "map" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut new_rows = Vec::new();
                        for (i, row) in q.rows.iter().enumerate() {
                            let row_struct = CfmlValue::Struct(row.clone());
                            let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), object.clone()];
                            self.closure_parent_writeback = None;
                            let mapped = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if let CfmlValue::Struct(s) = mapped {
                                new_rows.push(s);
                            } else {
                                new_rows.push(row.clone());
                            }
                        }
                        let mut result = q.clone();
                        result.rows = new_rows;
                        return Ok(CfmlValue::Query(result));
                    }
                    return Ok(object.clone());
                }
                "filter" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut new_rows = Vec::new();
                        for (i, row) in q.rows.iter().enumerate() {
                            let row_struct = CfmlValue::Struct(row.clone());
                            let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), object.clone()];
                            self.closure_parent_writeback = None;
                            let keep = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if keep.is_true() {
                                new_rows.push(row.clone());
                            }
                        }
                        let mut result = q.clone();
                        result.rows = new_rows;
                        return Ok(CfmlValue::Query(result));
                    }
                    return Ok(object.clone());
                }
                "reduce" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut acc = extra_args.get(1).cloned().unwrap_or(CfmlValue::Null);
                        for (i, row) in q.rows.iter().enumerate() {
                            let row_struct = CfmlValue::Struct(row.clone());
                            let cb_args = vec![acc.clone(), row_struct, CfmlValue::Int((i + 1) as i64), object.clone()];
                            self.closure_parent_writeback = None;
                            acc = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                        }
                        return Ok(acc);
                    }
                    return Ok(CfmlValue::Null);
                }
                "sort" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        let mut rows = q.rows.clone();
                        let n = rows.len();
                        for i in 0..n {
                            for j in 0..n.saturating_sub(1 + i) {
                                let a = CfmlValue::Struct(rows[j].clone());
                                let b = CfmlValue::Struct(rows[j + 1].clone());
                                let cb_args = vec![a, b];
                                self.closure_parent_writeback = None;
                                let cmp = self.call_function(&callback, cb_args, &IndexMap::new())?;
                                if let Some(ref wb) = self.closure_parent_writeback {
                                    Self::write_back_to_captured_scope(&callback, wb);
                                }
                                let cmp_val = match &cmp {
                                    CfmlValue::Int(n) => *n,
                                    CfmlValue::Double(d) => *d as i64,
                                    _ => 0,
                                };
                                if cmp_val > 0 {
                                    rows.swap(j, j + 1);
                                }
                            }
                        }
                        let mut result = q.clone();
                        result.rows = rows;
                        return Ok(CfmlValue::Query(result));
                    }
                    return Ok(object.clone());
                }
                "some" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        for (i, row) in q.rows.iter().enumerate() {
                            let row_struct = CfmlValue::Struct(row.clone());
                            let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), object.clone()];
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if result.is_true() { return Ok(CfmlValue::Bool(true)); }
                        }
                        return Ok(CfmlValue::Bool(false));
                    }
                    return Ok(CfmlValue::Bool(false));
                }
                "every" => {
                    if let Some(callback) = extra_args.first().cloned() {
                        for (i, row) in q.rows.iter().enumerate() {
                            let row_struct = CfmlValue::Struct(row.clone());
                            let cb_args = vec![row_struct, CfmlValue::Int((i + 1) as i64), object.clone()];
                            self.closure_parent_writeback = None;
                            let result = self.call_function(&callback, cb_args, &IndexMap::new())?;
                            if let Some(ref wb) = self.closure_parent_writeback {
                                Self::write_back_to_captured_scope(&callback, wb);
                            }
                            if !result.is_true() { return Ok(CfmlValue::Bool(false)); }
                        }
                        return Ok(CfmlValue::Bool(true));
                    }
                    return Ok(CfmlValue::Bool(true));
                }
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

            // For string member functions where the standalone signature has the
            // "main" string as the second arg (e.g., find(substring, string),
            // insert(substring, string, pos)), swap the first two args so the
            // object (which the member was called on) becomes the second arg.
            if matches!(object, CfmlValue::String(_)) && args.len() >= 2 {
                match name {
                    "find" | "findNoCase" | "insert" => {
                        args.swap(0, 1);
                    }
                    _ => {}
                }
            }

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
            // Bind 'this' to the object + inject component variables scope
            let mut method_locals = IndexMap::new();
            if let CfmlValue::Struct(ref s) = object {
                if let Some(CfmlValue::Struct(vars)) = s.get("__variables") {
                    for (k, v) in vars {
                        method_locals.insert(k.clone(), v.clone());
                    }
                }
            }
            method_locals.insert("this".to_string(), object.clone());
            self.closure_parent_writeback = None;
            let result = self.call_function(&func_ref, args, &method_locals)?;
            if let Some(ref wb) = self.closure_parent_writeback {
                Self::write_back_to_captured_scope(&func_ref, wb);
            }
            return Ok(result);
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
                let mut missing_args = IndexMap::new();
                for (i, a) in args_array.iter().enumerate() {
                    missing_args.insert((i + 1).to_string(), a.clone());
                }
                let mut method_locals = IndexMap::new();
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

    /// Check if a variable name (possibly dotted like "request.data.name") is defined
    /// by walking the scope chain: locals → request → application → server → globals
    fn is_variable_defined(&self, var_name: &str, locals: &IndexMap<String, CfmlValue>) -> bool {
        let parts: Vec<&str> = var_name.split('.').collect();
        if parts.is_empty() {
            return false;
        }

        let root = parts[0].to_lowercase();

        // Try to resolve the root variable from scope chain
        let root_val = if root == "local" || root == "variables" {
            Some(CfmlValue::Struct(locals.clone()))
        } else if root == "request" {
            Some(CfmlValue::Struct(self.request_scope.clone()))
        } else if root == "application" {
            if let Some(ref app_scope) = self.application_scope {
                if let Ok(scope) = app_scope.lock() {
                    Some(CfmlValue::Struct(scope.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        } else if root == "session" {
            Some(self.get_session_scope())
        } else if root == "cookie" {
            self.globals.get("cookie").cloned()
                .or(Some(CfmlValue::Struct(IndexMap::new())))
        } else if root == "server" {
            Some(CfmlValue::Struct(IndexMap::new())) // server scope always exists
        } else {
            // Check locals (exact then CI)
            locals.get(parts[0]).cloned()
                .or_else(|| locals.iter()
                    .find(|(k, _)| k.to_lowercase() == root)
                    .map(|(_, v)| v.clone()))
                // Check request scope
                .or_else(|| self.request_scope.iter()
                    .find(|(k, _)| k.to_lowercase() == root)
                    .map(|(_, v)| v.clone()))
                // Check globals
                .or_else(|| self.globals.get(parts[0]).cloned())
                .or_else(|| self.globals.iter()
                    .find(|(k, _)| k.to_lowercase() == root)
                    .map(|(_, v)| v.clone()))
        };

        let root_val = match root_val {
            Some(v) => v,
            None => return false,
        };

        if parts.len() == 1 {
            return true;
        }

        // Walk the dotted path segments
        let mut current = root_val;
        // For scope-named roots (request, local, etc.), start resolving from parts[1]
        // For regular vars, start from parts[1] too
        for &segment in &parts[1..] {
            let seg_lower = segment.to_lowercase();
            match &current {
                CfmlValue::Struct(s) => {
                    if let Some(v) = s.iter()
                        .find(|(k, _)| k.to_lowercase() == seg_lower)
                        .map(|(_, v)| v.clone())
                    {
                        current = v;
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }

    /// Shallow equality check for CfmlValues — avoids recursing into captured
    /// scopes (which could cause infinite recursion with shared environments).
    fn values_equal_shallow(a: &CfmlValue, b: &CfmlValue) -> bool {
        match (a, b) {
            (CfmlValue::Null, CfmlValue::Null) => true,
            (CfmlValue::Bool(a), CfmlValue::Bool(b)) => a == b,
            (CfmlValue::Int(a), CfmlValue::Int(b)) => a == b,
            (CfmlValue::Double(a), CfmlValue::Double(b)) => a == b,
            (CfmlValue::String(a), CfmlValue::String(b)) => a == b,
            (CfmlValue::Array(a), CfmlValue::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Self::values_equal_shallow(x, y))
            }
            (CfmlValue::Struct(a), CfmlValue::Struct(b)) => {
                a.len() == b.len() && a.iter().all(|(k, v)| {
                    b.get(k).map_or(false, |bv| Self::values_equal_shallow(v, bv))
                })
            }
            // Functions/Closures/Components/Queries/Binary: treat as always different
            // to ensure write-back happens (avoids recursing into captured scopes).
            _ => false,
        }
    }

    /// Write back mutations into a closure's shared Arc<RwLock> environment.
    /// Only updates variables that already exist in the captured scope (prevents pollution).
    fn write_back_to_captured_scope(func_ref: &CfmlValue, writeback: &IndexMap<String, CfmlValue>) {
        if let CfmlValue::Function(ref f) = func_ref {
            if let Some(ref shared_env) = f.captured_scope {
                let mut env = shared_env.write().unwrap();
                for (k, v) in writeback {
                    if env.contains_key(k) {
                        env.insert(k.clone(), v.clone());
                    }
                }
            }
        }
    }

    /// Compute final closure write-back after a higher-order function loop.
    /// Compares modified locals against original parent_locals and sets closure_parent_writeback.
    fn set_ho_final_writeback(
        &mut self,
        modified: &IndexMap<String, CfmlValue>,
        original: &IndexMap<String, CfmlValue>,
    ) {
        let mut final_wb = IndexMap::new();
        for (k, v) in modified {
            match original.get(k) {
                Some(pv) => {
                    if !Self::values_equal_shallow(v, pv) {
                        final_wb.insert(k.clone(), v.clone());
                    }
                }
                None => {
                    final_wb.insert(k.clone(), v.clone());
                }
            }
        }
        if !final_wb.is_empty() {
            self.closure_parent_writeback = Some(final_wb);
        }
    }

    /// Resolve a dot-path class name to a .cfc file path using component mappings.
    /// Mappings are sorted longest-prefix-first for correct precedence.
    fn resolve_path_with_mappings(&self, class_name: &str) -> Option<String> {
        if self.mappings.is_empty() {
            return None;
        }
        // Convert dot-path to slash-path: "taffy.core.api" → "/taffy/core/api"
        let slash_path = format!("/{}", class_name.replace('.', "/"));
        let slash_lower = slash_path.to_lowercase();

        for mapping in &self.mappings {
            let prefix_lower = mapping.name.to_lowercase();
            if slash_lower.starts_with(&prefix_lower) || (mapping.name == "/" && slash_lower.starts_with('/')) {
                let remainder = if mapping.name == "/" {
                    &slash_path[1..] // Strip leading /
                } else {
                    &slash_path[mapping.name.len()..]
                };
                let remainder = remainder.trim_start_matches('/');
                let cfc_path = format!(
                    "{}/{}.cfc",
                    mapping.path.trim_end_matches('/'),
                    remainder.replace('/', std::path::MAIN_SEPARATOR_STR)
                );
                if std::path::Path::new(&cfc_path).exists() {
                    return Some(cfc_path);
                }
            }
        }
        None
    }

    /// Resolve an include path (e.g. "/taffy/core/foo.cfm") using component mappings.
    fn resolve_include_with_mappings(&self, include_path: &str) -> Option<String> {
        if self.mappings.is_empty() {
            return None;
        }
        let path_lower = include_path.to_lowercase();
        for mapping in &self.mappings {
            let prefix_lower = mapping.name.to_lowercase();
            if path_lower.starts_with(&prefix_lower) || (mapping.name == "/" && path_lower.starts_with('/')) {
                let remainder = if mapping.name == "/" {
                    &include_path[1..]
                } else {
                    &include_path[mapping.name.len()..]
                };
                let remainder = remainder.trim_start_matches('/');
                let resolved = format!(
                    "{}/{}",
                    mapping.path.trim_end_matches('/'),
                    remainder
                );
                if std::path::Path::new(&resolved).exists() {
                    return Some(resolved);
                }
            }
        }
        None
    }

    /// Get or create the cfthread scope on the variables scope.
    fn get_or_create_cfthread_scope(&mut self) -> &mut CfmlValue {
        if !self.globals.contains_key("cfthread") {
            self.globals.insert("cfthread".to_string(), CfmlValue::Struct(IndexMap::new()));
        }
        self.globals.get_mut("cfthread").unwrap()
    }

    /// Resolve a custom tag path specification to an actual filesystem path.
    fn resolve_custom_tag_path(&self, path_spec: &str) -> Result<String, CfmlError> {
        if path_spec.starts_with("__cf_:") {
            // cf_ prefix tag: find tagname.cfm
            let tag_name = &path_spec[6..];
            let filename = format!("{}.cfm", tag_name);

            // 1) Look in calling template directory
            if let Some(ref source) = self.source_file {
                let source_dir = std::path::Path::new(source).parent()
                    .unwrap_or_else(|| std::path::Path::new("."));
                let candidate = source_dir.join(&filename);
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }

            // 2) Look in custom_tag_paths
            for dir in &self.custom_tag_paths {
                let candidate = std::path::Path::new(dir).join(&filename);
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }

            // 3) Look in mappings
            for mapping in &self.mappings {
                let candidate = std::path::Path::new(&mapping.path).join(&filename);
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }

            Err(CfmlError::runtime(format!("Custom tag 'cf_{}' not found", tag_name)))
        } else if path_spec.starts_with("__name:") {
            // cfmodule name="dot.path" → convert dots to slashes
            let dot_path = &path_spec[7..];
            let rel_path = format!("{}.cfm", dot_path.replace('.', "/"));

            // Search in custom_tag_paths then mappings
            for dir in &self.custom_tag_paths {
                let candidate = std::path::Path::new(dir).join(&rel_path);
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }

            for mapping in &self.mappings {
                let candidate = std::path::Path::new(&mapping.path).join(&rel_path);
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }

            Err(CfmlError::runtime(format!("Custom tag with name '{}' not found", dot_path)))
        } else {
            // Plain path: resolve relative to source_file
            let resolved = if let Some(ref source) = self.source_file {
                let source_dir = std::path::Path::new(source).parent()
                    .unwrap_or_else(|| std::path::Path::new("."));
                source_dir.join(path_spec).to_string_lossy().to_string()
            } else {
                path_spec.to_string()
            };

            if std::path::Path::new(&resolved).exists() {
                Ok(resolved)
            } else {
                Err(CfmlError::runtime(format!("Custom tag template '{}' not found", path_spec)))
            }
        }
    }

    /// Execute a custom tag template file with the given tag-local variables.
    /// Reuses the include pattern: save/restore source_file, program, try_stack.
    fn execute_custom_tag_template(
        &mut self,
        template_path: &str,
        tag_locals: &IndexMap<String, CfmlValue>,
    ) -> Result<(), CfmlError> {
        let source_code = std::fs::read_to_string(template_path).map_err(|e| {
            CfmlError::runtime(format!("Cannot read custom tag '{}': {}", template_path, e))
        })?;

        let source_code = if cfml_compiler::tag_parser::has_cfml_tags(&source_code) {
            cfml_compiler::tag_parser::tags_to_script(&source_code)
        } else {
            source_code
        };

        let mut parser = cfml_compiler::parser::Parser::new(source_code);
        let ast = parser.parse().map_err(|e| {
            CfmlError::runtime(format!("Custom tag parse error in '{}': {}", template_path, e.message))
        })?;

        let compiler = cfml_codegen::compiler::CfmlCompiler::new();
        let sub_program = compiler.compile(ast);

        let old_program = std::mem::replace(&mut self.program, sub_program);
        let old_source = self.source_file.clone();
        self.source_file = Some(template_path.to_string());

        let main_idx = self.program.functions.iter()
            .position(|f| f.name == "__main__")
            .unwrap_or(0);
        let tag_func = self.program.functions[main_idx].clone();

        let saved_try_stack = std::mem::take(&mut self.try_stack);
        let result = self.execute_function_with_args(&tag_func, Vec::new(), Some(tag_locals));
        self.try_stack = saved_try_stack;
        self.program = old_program;
        self.source_file = old_source;

        result.map(|_| ())
    }

    /// Adjust func_idx values in CfmlFunction bodies within a component struct.
    /// When sub-program functions are merged into the main program at an offset,
    /// the stored func_idx values need to be updated to reflect their new positions.
    fn fixup_func_indices(val: &mut CfmlValue, offset: usize) {
        match val {
            CfmlValue::Struct(s) => {
                // We need to collect keys first, then mutate
                let keys: Vec<String> = s.keys().cloned().collect();
                for key in keys {
                    if let Some(v) = s.get_mut(&key) {
                        match v {
                            CfmlValue::Function(ref mut f) => {
                                // Update the func_idx stored in the body
                                if let cfml_common::dynamic::CfmlClosureBody::Expression(ref mut body) = f.body {
                                    if let CfmlValue::Int(ref mut idx) = body.as_mut() {
                                        *idx += offset as i64;
                                    }
                                }
                            }
                            CfmlValue::Struct(_) => {
                                Self::fixup_func_indices(v, offset);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Adjust func_idx values by a delta for indices >= min_index.
    /// Used when restoring cached functions at a different program offset
    /// than where they were originally inserted.
    fn adjust_func_indices(val: &mut CfmlValue, min_index: usize, delta: i64) {
        match val {
            CfmlValue::Struct(s) => {
                let keys: Vec<String> = s.keys().cloned().collect();
                for key in keys {
                    if let Some(v) = s.get_mut(&key) {
                        match v {
                            CfmlValue::Function(ref mut f) => {
                                if let cfml_common::dynamic::CfmlClosureBody::Expression(ref mut body) = f.body {
                                    if let CfmlValue::Int(ref mut idx) = body.as_mut() {
                                        if *idx >= min_index as i64 {
                                            *idx += delta;
                                        }
                                    }
                                }
                            }
                            CfmlValue::Struct(_) | CfmlValue::Array(_) => {
                                Self::adjust_func_indices(v, min_index, delta);
                            }
                            _ => {}
                        }
                    }
                }
            }
            CfmlValue::Array(arr) => {
                for item in arr.iter_mut() {
                    Self::adjust_func_indices(item, min_index, delta);
                }
            }
            _ => {}
        }
    }

    /// Get default datasource from application scope or request scope
    fn get_default_datasource(&self, parent_locals: &IndexMap<String, CfmlValue>) -> String {
        // Check application scope for datasource config
        if let Some(ref app_scope) = self.application_scope {
            if let Ok(scope) = app_scope.lock() {
                if let Some(ds) = scope.get("datasource").or_else(|| scope.get("defaultdatasource")) {
                    let s = ds.as_string();
                    if !s.is_empty() {
                        return s;
                    }
                }
            }
        }
        // Check local variables
        if let Some(ds) = parent_locals.get("datasource") {
            let s = ds.as_string();
            if !s.is_empty() {
                return s;
            }
        }
        String::new()
    }

    /// Get the session scope for the current request
    fn get_session_scope(&self) -> CfmlValue {
        if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
            if let Ok(sessions) = state.sessions.lock() {
                if let Some(session) = sessions.get(sid) {
                    return CfmlValue::Struct(session.variables.clone());
                }
            }
        }
        CfmlValue::Struct(IndexMap::new())
    }

    /// Set the session scope for the current request
    fn set_session_scope(&self, vars: IndexMap<String, CfmlValue>) {
        if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
            if let Ok(mut sessions) = state.sessions.lock() {
                if let Some(session) = sessions.get_mut(sid) {
                    session.variables = vars;
                    session.last_accessed = std::time::Instant::now();
                } else {
                    sessions.insert(sid.clone(), SessionData {
                        variables: vars,
                        created: std::time::Instant::now(),
                        last_accessed: std::time::Instant::now(),
                        auth_user: None,
                        auth_roles: Vec::new(),
                        timeout_secs: 1800,
                    });
                }
            }
        }
    }

    /// Update a single key in the session scope
    #[allow(dead_code)]
    fn set_session_variable(&self, key: &str, value: CfmlValue) {
        if let (Some(ref state), Some(ref sid)) = (&self.server_state, &self.session_id) {
            if let Ok(mut sessions) = state.sessions.lock() {
                if let Some(session) = sessions.get_mut(sid) {
                    session.variables.insert(key.to_string(), value);
                    session.last_accessed = std::time::Instant::now();
                } else {
                    let mut vars = IndexMap::new();
                    vars.insert(key.to_string(), value);
                    sessions.insert(sid.clone(), SessionData {
                        variables: vars,
                        created: std::time::Instant::now(),
                        last_accessed: std::time::Instant::now(),
                        auth_user: None,
                        auth_roles: Vec::new(),
                        timeout_secs: 1800,
                    });
                }
            }
        }
    }

    /// Resolve a component template by name: tries locals, globals (exact + CI),
    /// then loads from a .cfc file on disk.
    fn resolve_component_template(
        &mut self,
        class_name: &str,
        locals: &IndexMap<String, CfmlValue>,
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
        // 4. Try loading .cfc file — first relative, then via mappings
        let cfc_path = {
            // If class_name is already an absolute path or has .cfc extension, use directly
            let as_path = std::path::Path::new(class_name);
            if as_path.is_absolute() || class_name.to_lowercase().ends_with(".cfc") {
                let p = if class_name.to_lowercase().ends_with(".cfc") {
                    class_name.to_string()
                } else {
                    format!("{}.cfc", class_name)
                };
                if std::path::Path::new(&p).exists() {
                    p
                } else if let Some(ref source) = self.source_file {
                    // Try relative to source file
                    let source_dir = std::path::Path::new(source).parent()
                        .unwrap_or_else(|| std::path::Path::new("."));
                    source_dir.join(&p).to_string_lossy().to_string()
                } else {
                    p
                }
            } else {
                // Dot-path: convert dots to path separators
                let relative_path = if let Some(ref source) = self.source_file {
                    let source_dir = std::path::Path::new(source).parent()
                        .unwrap_or_else(|| std::path::Path::new("."));
                    let file_name = class_name.replace('.', std::path::MAIN_SEPARATOR_STR);
                    source_dir.join(format!("{}.cfc", file_name)).to_string_lossy().to_string()
                } else {
                    format!("{}.cfc", class_name.replace('.', std::path::MAIN_SEPARATOR_STR))
                };
                if std::path::Path::new(&relative_path).exists() {
                    relative_path
                } else if let Some(mapped) = self.resolve_path_with_mappings(class_name) {
                    mapped
                } else if let Some(ref base) = self.base_template_path {
                    // Try resolving relative to the base template (web root equivalent)
                    let base_dir = std::path::Path::new(base).parent()
                        .unwrap_or_else(|| std::path::Path::new("."));
                    let file_name = class_name.replace('.', std::path::MAIN_SEPARATOR_STR);
                    let base_path = base_dir.join(format!("{}.cfc", file_name)).to_string_lossy().to_string();
                    if std::path::Path::new(&base_path).exists() {
                        base_path
                    } else {
                        relative_path
                    }
                } else {
                    relative_path // Fall back to relative (will fail at read_to_string below)
                }
            }
        };

        if std::env::var("RUSTCFML_DEBUG_RESOLVE").is_ok() {
            eprintln!("[resolve] class='{}' source_file={:?} base={:?} → cfc_path='{}'  exists={}",
                class_name, self.source_file, self.base_template_path, cfc_path,
                std::path::Path::new(&cfc_path).exists());
        }
        if let Ok(source_code) = std::fs::read_to_string(&cfc_path) {
            let source_code = if cfml_compiler::tag_parser::has_cfml_tags(&source_code) {
                let converted = cfml_compiler::tag_parser::tags_to_script(&source_code);
                if std::env::var("RUSTCFML_DUMP_TAGS").is_ok() {
                    eprintln!("=== TAG CONVERTED: {} ===\n{}\n=== END ===", cfc_path, converted);
                }
                converted
            } else {
                source_code
            };
            if let Ok(ast) = cfml_compiler::parser::Parser::new(source_code).parse() {
                let compiler = cfml_codegen::compiler::CfmlCompiler::new();
                let sub_program = compiler.compile(ast);
                let old_program = std::mem::replace(&mut self.program, sub_program);
                // Set source_file to CFC path so parent resolution works relative to CFC
                let old_source_file = self.source_file.clone();
                self.source_file = Some(cfc_path.clone());
                let main_idx = self.program.functions.iter()
                    .position(|f| f.name == "__main__")
                    .unwrap_or(0);
                let cfc_func = self.program.functions[main_idx].clone();
                let _ = self.execute_function_with_args(&cfc_func, Vec::new(), Some(locals));
                self.source_file = old_source_file;
                // Capture component body variables
                let component_variables = self.captured_locals.take().unwrap_or_default();
                // Merge sub-program functions — track base offset for func_idx fixup
                let sub_funcs = self.program.functions.clone();
                self.program = old_program;
                let base_idx = self.program.functions.len();
                for func in sub_funcs {
                    if func.name != "__main__" {
                        self.program.functions.push(func.clone());
                        if self.user_functions.contains_key(&func.name) {
                            self.user_functions.insert(func.name.clone(), func.clone());
                        }
                    }
                }
                // Fix up func_idx in the component struct stored in globals
                // Sub-program functions were at indices [0..N), now at [base_idx..base_idx+N)
                let short_name = class_name.split('.').last().unwrap_or(class_name);
                let mut result = self.globals.get(class_name).cloned()
                    .or_else(|| self.globals.get(short_name).cloned())
                    .or_else(|| {
                        let lower = class_name.to_lowercase();
                        self.globals.iter()
                            .find(|(k, _)| k.to_lowercase() == lower)
                            .map(|(_, v)| v.clone())
                    })
                    .or_else(|| self.globals.get("Anonymous").cloned());
                // Adjust func_idx in all function values.
                // Sub-program had __main__ at index 0 which was skipped during merge.
                // So sub-program func at index i is now at base_idx + (i - 1).
                // The offset to add is (base_idx - 1).
                if base_idx > 0 {
                    if let Some(ref mut val) = result {
                        Self::fixup_func_indices(val, base_idx - 1);
                    }
                }
                // Store the CFC source path for parent resolution during inheritance
                if let Some(CfmlValue::Struct(ref mut s)) = result {
                    s.insert("__source_file".to_string(), CfmlValue::String(cfc_path.clone()));
                }
                // Store component body variables as __variables
                if !component_variables.is_empty() {
                    if let Some(CfmlValue::Struct(ref mut s)) = result {
                        let mut vars_scope: IndexMap<String, CfmlValue> = IndexMap::new();
                        for (k, v) in &component_variables {
                            let k_lower = k.to_lowercase();
                            if k_lower == "this" || k_lower == "arguments" || k.starts_with("__")
                                || matches!(v, CfmlValue::Function(_))
                            {
                                continue;
                            }
                            vars_scope.insert(k.clone(), v.clone());
                        }
                        if !vars_scope.is_empty() {
                            s.insert("__variables".to_string(), CfmlValue::Struct(vars_scope));
                        }
                    }
                }
                return result;
            }
        }
        None
    }

    /// Resolve the full inheritance chain for a component template.
    /// If the template has an `__extends` key, load the parent, recursively
    /// resolve its inheritance, then merge child on top of parent.
    /// Resolve all required method names from an interface, including inherited ones.
    fn resolve_interface_methods(
        &mut self,
        iface_name: &str,
        locals: &IndexMap<String, CfmlValue>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<Vec<String>, CfmlError> {
        let name_lower = iface_name.to_lowercase();
        if visited.contains(&name_lower) {
            return Ok(Vec::new()); // Cycle detected
        }
        visited.insert(name_lower);

        // Look up the interface in globals/locals
        let iface = locals.get(iface_name)
            .or_else(|| {
                // Case-insensitive lookup in globals
                self.globals.iter()
                    .find(|(k, _)| k.to_lowercase() == iface_name.to_lowercase())
                    .map(|(_, v)| v)
            })
            .cloned();

        let iface = match iface {
            Some(iface) => iface,
            None => {
                // Try resolving as a component template (file-based)
                match self.resolve_component_template(iface_name, locals) {
                    Some(t) => t,
                    None => return Err(CfmlError::runtime(
                        format!("Interface '{}' not found", iface_name)
                    )),
                }
            }
        };

        let iface_struct = match &iface {
            CfmlValue::Struct(s) => s,
            _ => return Err(CfmlError::runtime(
                format!("'{}' is not an interface", iface_name)
            )),
        };

        // Verify it's actually an interface
        let is_interface = matches!(iface_struct.get("__is_interface"), Some(CfmlValue::Bool(true)));
        if !is_interface {
            return Err(CfmlError::runtime(
                format!("'{}' is not an interface", iface_name)
            ));
        }

        let mut methods = Vec::new();

        // Collect methods from __methods struct
        if let Some(CfmlValue::Struct(methods_map)) = iface_struct.get("__methods") {
            for key in methods_map.keys() {
                methods.push(key.clone());
            }
        }

        // Recursively collect from parent interfaces
        if let Some(CfmlValue::Array(parents)) = iface_struct.get("__extends") {
            for parent in parents {
                let parent_name = parent.as_string();
                let parent_methods = self.resolve_interface_methods(&parent_name, locals, visited)?;
                for m in parent_methods {
                    if !methods.iter().any(|existing| existing.to_lowercase() == m.to_lowercase()) {
                        methods.push(m);
                    }
                }
            }
        }

        Ok(methods)
    }

    /// Collect all transitive interface names from an interface's extends chain.
    fn collect_transitive_interfaces(
        &mut self,
        iface_name: &str,
        locals: &IndexMap<String, CfmlValue>,
        visited: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>,
    ) {
        let name_lower = iface_name.to_lowercase();
        if visited.contains(&name_lower) {
            return;
        }
        visited.insert(name_lower);
        result.push(iface_name.to_string());

        // Look up the interface
        let iface = locals.get(iface_name)
            .or_else(|| {
                self.globals.iter()
                    .find(|(k, _)| k.to_lowercase() == iface_name.to_lowercase())
                    .map(|(_, v)| v)
            })
            .cloned();

        let iface = match iface {
            Some(i) => i,
            None => match self.resolve_component_template(iface_name, locals) {
                Some(t) => t,
                None => return,
            },
        };

        if let CfmlValue::Struct(s) = &iface {
            if let Some(CfmlValue::Array(parents)) = s.get("__extends") {
                for parent in parents.clone() {
                    let parent_name = parent.as_string();
                    self.collect_transitive_interfaces(&parent_name, locals, visited, result);
                }
            }
        }
    }

    /// Validate that a component struct implements all methods required by its interfaces.
    /// Returns the full set of transitive interface names (for __implements_chain).
    fn validate_interface_implementation(
        &mut self,
        component: &IndexMap<String, CfmlValue>,
        locals: &IndexMap<String, CfmlValue>,
    ) -> Result<Vec<String>, CfmlError> {
        let iface_names = match component.get("__implements") {
            Some(CfmlValue::Array(arr)) => arr.clone(),
            _ => return Ok(Vec::new()), // No interfaces to validate
        };

        let comp_name = component.get("__name")
            .map(|v| v.as_string())
            .unwrap_or_else(|| "Anonymous".to_string());

        let mut all_interfaces = Vec::new();

        for iface_val in &iface_names {
            let iface_name = iface_val.as_string();

            // Collect all transitive interface names
            let mut visited_ifaces = std::collections::HashSet::new();
            self.collect_transitive_interfaces(&iface_name, locals, &mut visited_ifaces, &mut all_interfaces);

            // Validate methods
            let mut visited = std::collections::HashSet::new();
            let required_methods = self.resolve_interface_methods(&iface_name, locals, &mut visited)?;

            for method_name in &required_methods {
                // Check if component has this method (case-insensitive)
                let has_method = component.iter().any(|(k, v)| {
                    k.to_lowercase() == method_name.to_lowercase()
                        && matches!(v, CfmlValue::Function(_))
                });
                if !has_method {
                    return Err(CfmlError::runtime(
                        format!(
                            "Component '{}' does not implement method '{}' required by interface '{}'",
                            comp_name, method_name, iface_name
                        )
                    ));
                }
            }
        }

        Ok(all_interfaces)
    }

    fn resolve_inheritance(
        &mut self,
        template: CfmlValue,
        locals: &IndexMap<String, CfmlValue>,
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
        locals: &IndexMap<String, CfmlValue>,
        visited: &mut std::collections::HashSet<String>,
    ) -> CfmlValue {
        // Check circular
        if visited.contains(&parent_name.to_lowercase()) {
            return child;
        }
        visited.insert(parent_name.to_lowercase());

        // Temporarily set source_file to the child CFC's path so parent
        // resolution finds siblings in the same directory
        let old_source_file = if let CfmlValue::Struct(ref cs) = child {
            if let Some(CfmlValue::String(src)) = cs.get("__source_file") {
                let prev = self.source_file.clone();
                self.source_file = Some(src.clone());
                Some(prev)
            } else { None }
        } else { None };

        // Resolve parent template
        let parent = match self.resolve_component_template(parent_name, locals) {
            Some(p) => p,
            None => {
                if let Some(prev) = old_source_file { self.source_file = prev; }
                return child; // Parent not found, return child as-is
            }
        };

        // Restore source_file
        if let Some(prev) = old_source_file { self.source_file = prev; }

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
        let mut super_methods = IndexMap::new();
        for (k, v) in &parent_map {
            if matches!(v, CfmlValue::Function(_)) && !k.starts_with("__") {
                super_methods.insert(k.clone(), v.clone());
            }
        }

        // Merge __variables from parent and child (child overrides parent)
        let parent_vars = parent_map.get("__variables")
            .and_then(|v| if let CfmlValue::Struct(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();
        let child_vars = child_map.get("__variables")
            .and_then(|v| if let CfmlValue::Struct(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();
        if !parent_vars.is_empty() || !child_vars.is_empty() {
            let mut merged_vars = parent_vars;
            for (k, v) in child_vars {
                merged_vars.insert(k, v);
            }
            parent_map.insert("__variables".to_string(), CfmlValue::Struct(merged_vars));
        }

        // Layer child on top of parent (child overrides parent)
        for (k, v) in &child_map {
            if k == "__extends" || k == "__variables" {
                continue; // Already merged above; don't overwrite
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

        // Propagate __implements through inheritance: aggregate child + parent interfaces
        let mut all_implements = std::collections::HashSet::new();
        // Collect child's direct interfaces
        if let Some(CfmlValue::Array(child_ifaces)) = child_map.get("__implements") {
            for iface in child_ifaces {
                all_implements.insert(iface.as_string().to_lowercase());
            }
        }
        // Collect parent's interfaces (direct + inherited)
        if let Some(CfmlValue::Array(parent_ifaces)) = parent_map.get("__implements") {
            for iface in parent_ifaces {
                all_implements.insert(iface.as_string().to_lowercase());
            }
        }
        if let Some(CfmlValue::Array(parent_chain)) = parent_map.get("__implements_chain") {
            for iface in parent_chain {
                all_implements.insert(iface.as_string().to_lowercase());
            }
        }
        if !all_implements.is_empty() {
            let chain: Vec<CfmlValue> = all_implements.into_iter()
                .map(|s| CfmlValue::String(s))
                .collect();
            parent_map.insert("__implements_chain".to_string(), CfmlValue::Array(chain));
        }

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
        let empty_locals = IndexMap::new();
        let _ = self.execute_function_with_args(&cfc_func, Vec::new(), Some(&empty_locals));

        // Capture component body locals as the variables scope
        let component_variables = self.captured_locals.take().unwrap_or_default();

        // Merge sub-program functions into main program
        let sub_funcs = self.program.functions.clone();
        self.program = old_program;
        let base_idx = self.program.functions.len();
        for func in sub_funcs {
            if func.name != "__main__" {
                self.program.functions.push(func.clone());
                self.user_functions.insert(func.name.clone(), func.clone());
            }
        }

        // Find the component struct in globals
        let mut template = self.globals.iter()
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
            })?;

        // Fix up func_idx in the template's function values (sub-program index → merged index)
        if base_idx > 0 {
            Self::fixup_func_indices(&mut template, base_idx - 1);
        }

        // Store component body variables as __variables on the template
        // This makes variables.framework etc. accessible to component methods
        if !component_variables.is_empty() {
            let mut vars_scope: IndexMap<String, CfmlValue> = IndexMap::new();
            for (k, v) in &component_variables {
                let k_lower = k.to_lowercase();
                // Skip internal/meta keys and functions — keep only data variables
                if k_lower == "this" || k_lower == "arguments" || k.starts_with("__")
                    || matches!(v, CfmlValue::Function(_))
                {
                    continue;
                }
                vars_scope.insert(k.clone(), v.clone());
            }
            if !vars_scope.is_empty() {
                if let CfmlValue::Struct(ref mut s) = template {
                    s.insert("__variables".to_string(), CfmlValue::Struct(vars_scope));
                }
            }
        }

        // Extract and install mappings early so resolve_inheritance can find parent classes
        let (_, _, mut early_mappings, _, _, _) = Self::extract_app_config(&template);
        let app_cfc_dir = std::path::Path::new(path).parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        for mapping in &mut early_mappings {
            let expanded = if std::path::Path::new(&mapping.path).is_absolute() {
                mapping.path.clone()
            } else {
                app_cfc_dir.join(&mapping.path).canonicalize()
                    .unwrap_or_else(|_| app_cfc_dir.join(&mapping.path))
                    .to_string_lossy().to_string()
            };
            mapping.path = expanded;
        }
        if !early_mappings.is_empty() {
            early_mappings.sort_by(|a, b| b.name.len().cmp(&a.name.len()));
            // Add default "/" mapping for this directory
            if !early_mappings.iter().any(|m| m.name == "/") {
                early_mappings.push(CfmlMapping {
                    name: "/".to_string(),
                    path: app_cfc_dir.to_string_lossy().to_string(),
                });
            }
            self.mappings = early_mappings;
        }

        // Resolve inheritance (e.g. extends="taffy.core.api")
        let resolved = self.resolve_inheritance(template, &IndexMap::new());
        Some(resolved)
    }

    /// Extract application config from a component struct
    /// Returns (app_name, config, mappings, session_management, session_timeout_secs)
    fn extract_app_config(template: &CfmlValue) -> (String, IndexMap<String, CfmlValue>, Vec<CfmlMapping>, bool, u64, Vec<String>) {
        let s = match template {
            CfmlValue::Struct(s) => s,
            _ => return ("default".to_string(), IndexMap::new(), Vec::new(), false, 1800, Vec::new()),
        };

        // Case-insensitive lookup for this.name
        let app_name = s.iter()
            .find(|(k, _)| k.to_lowercase() == "name")
            .and_then(|(_, v)| match v {
                CfmlValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "default".to_string());

        let mut config = IndexMap::new();
        for (k, v) in s {
            if !k.starts_with("__") && !matches!(v, CfmlValue::Function(_)) {
                config.insert(k.to_lowercase(), v.clone());
            }
        }

        // Extract mappings from this.mappings (case-insensitive key lookup)
        let mut mappings = Vec::new();
        if let Some(mappings_val) = s.iter()
            .find(|(k, _)| k.to_lowercase() == "mappings")
            .map(|(_, v)| v.clone())
        {
            if let CfmlValue::Struct(map_struct) = mappings_val {
                for (key, val) in &map_struct {
                    // Normalize mapping name: ensure leading+trailing "/"
                    let mut name = key.clone();
                    if !name.starts_with('/') {
                        name = format!("/{}", name);
                    }
                    if !name.ends_with('/') {
                        name = format!("{}/", name);
                    }
                    // Extract path: either a String directly or a Struct with a "path" key
                    let path = match val {
                        CfmlValue::String(p) => Some(p.clone()),
                        CfmlValue::Struct(inner) => {
                            inner.iter()
                                .find(|(k, _)| k.to_lowercase() == "path")
                                .and_then(|(_, v)| match v {
                                    CfmlValue::String(p) => Some(p.clone()),
                                    _ => None,
                                })
                        }
                        _ => None,
                    };
                    if let Some(path) = path {
                        mappings.push(CfmlMapping { name, path });
                    }
                }
            }
        }

        // Extract session management config
        let session_management = s.iter()
            .find(|(k, _)| k.to_lowercase() == "sessionmanagement")
            .map(|(_, v)| match v {
                CfmlValue::Bool(b) => *b,
                CfmlValue::String(s) => s.to_lowercase() == "true" || s.to_lowercase() == "yes",
                _ => false,
            })
            .unwrap_or(false);

        let session_timeout = s.iter()
            .find(|(k, _)| k.to_lowercase() == "sessiontimeout")
            .and_then(|(_, v)| match v {
                CfmlValue::Int(i) => Some(*i as u64),
                CfmlValue::Double(d) => Some(*d as u64),
                CfmlValue::String(s) => s.parse::<u64>().ok(),
                // createTimeSpan returns a double representing days
                _ => None,
            })
            .unwrap_or(1800); // Default 30 minutes

        // Extract customTagPaths from this.customTagPaths (case-insensitive)
        let mut custom_tag_paths = Vec::new();
        if let Some(ctp_val) = s.iter()
            .find(|(k, _)| k.to_lowercase() == "customtagpaths")
            .map(|(_, v)| v.clone())
        {
            match ctp_val {
                CfmlValue::Array(arr) => {
                    for item in &arr {
                        custom_tag_paths.push(item.as_string());
                    }
                }
                CfmlValue::String(s) => {
                    for part in s.split(',') {
                        let p = part.trim();
                        if !p.is_empty() {
                            custom_tag_paths.push(p.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        (app_name, config, mappings, session_management, session_timeout, custom_tag_paths)
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
                // Bind `this` to the template + inject component variables scope
                let mut parent_locals = IndexMap::new();
                // Inject __variables (component body vars like variables.framework)
                if let Some(CfmlValue::Struct(vars)) = s.iter()
                    .find(|(k, _)| *k == "__variables")
                    .map(|(_, v)| v.clone())
                {
                    for (k, v) in vars {
                        parent_locals.insert(k, v);
                    }
                }
                parent_locals.insert("this".to_string(), template.clone());
                match self.call_function(func, args, &parent_locals) {
                    Ok(_) => Ok(true),
                    Err(e) => Err(e),
                }
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

        // 3. Extract config and mappings
        let (app_name, _config, mut mappings, session_management, session_timeout, custom_tag_paths) = Self::extract_app_config(&template);

        // 3b. Expand mapping paths relative to Application.cfc directory
        let app_cfc_dir = std::path::Path::new(&app_cfc_path).parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        for mapping in &mut mappings {
            let expanded = if std::path::Path::new(&mapping.path).is_absolute() {
                mapping.path.clone()
            } else {
                app_cfc_dir.join(&mapping.path).canonicalize()
                    .unwrap_or_else(|_| app_cfc_dir.join(&mapping.path))
                    .to_string_lossy().to_string()
            };
            mapping.path = expanded;
        }
        // Sort by name length descending (longest prefix first)
        mappings.sort_by(|a, b| b.name.len().cmp(&a.name.len()));
        // Add default "/" mapping if not already present
        if !mappings.iter().any(|m| m.name == "/") {
            let root_dir = if let Some(ref source) = self.source_file {
                std::path::Path::new(source).parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .to_string_lossy().to_string()
            } else {
                std::env::current_dir().unwrap_or_default().to_string_lossy().to_string()
            };
            mappings.push(CfmlMapping { name: "/".to_string(), path: root_dir });
        }
        self.mappings = mappings;

        // 3c. Expand customTagPaths relative to Application.cfc directory
        self.custom_tag_paths = custom_tag_paths.into_iter().map(|p| {
            if std::path::Path::new(&p).is_absolute() {
                p
            } else {
                app_cfc_dir.join(&p).canonicalize()
                    .unwrap_or_else(|_| app_cfc_dir.join(&p))
                    .to_string_lossy().to_string()
            }
        }).collect();

        // 4. Wire up application scope
        if let Some(ref server_state) = self.server_state.clone() {
            let mut apps = server_state.applications.lock().unwrap();
            if !apps.contains_key(&app_name) {
                // New application
                let app_state = ApplicationState {
                    name: app_name.clone(),
                    variables: IndexMap::new(),
                    started: false,
                    config: _config.clone(),
                    cached_functions: Vec::new(),
                    cached_functions_original_offset: 0,
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

                // Record how many functions exist before onApplicationStart.
                // Everything beyond this index was added by onApplicationStart
                // (e.g. factory components, resource CFCs).
                let funcs_before = self.program.functions.len();

                match self.call_lifecycle_method(&template, "onApplicationStart", vec![]) {
                    Ok(_) => {
                        // Cache only the functions ADDED during onApplicationStart.
                        let funcs_after = self.program.functions.len();
                        if funcs_after > funcs_before {
                            if let Some(ref server_state) = self.server_state.clone() {
                                if let Ok(mut apps) = server_state.applications.lock() {
                                    if let Some(app) = apps.get_mut(&app_name) {
                                        app.cached_functions = self.program.functions[funcs_before..].to_vec();
                                        app.cached_functions_original_offset = funcs_before;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = self.call_lifecycle_method(&template, "onError", vec![
                            CfmlValue::String(e.message.clone()),
                            CfmlValue::String("onApplicationStart".to_string()),
                        ]);
                        return Err(e);
                    }
                }
            } else {
                // Restore cached functions from onApplicationStart.
                // Append them to the current program (which already has the page's
                // functions + Application.cfc functions from load_application_cfc).
                let cached = app.cached_functions.clone();
                let original_offset = app.cached_functions_original_offset;
                drop(apps);
                if !cached.is_empty() {
                    let new_offset = self.program.functions.len();
                    self.program.functions.extend(cached);

                    // If functions ended up at a different offset than originally,
                    // fix up function indices in the application scope values.
                    if new_offset != original_offset {
                        let index_delta = new_offset as i64 - original_offset as i64;
                        if let Some(ref app_scope) = self.application_scope {
                            if let Ok(mut scope) = app_scope.lock() {
                                let keys: Vec<String> = scope.keys().cloned().collect();
                                for key in keys {
                                    if let Some(val) = scope.get_mut(&key) {
                                        Self::adjust_func_indices(val, original_offset, index_delta);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // CLI mode: fresh application scope each time
            let scope = Arc::new(Mutex::new(IndexMap::new()));
            self.application_scope = Some(scope);

            // Still call onApplicationStart in CLI mode
            let _ = self.call_lifecycle_method(&template, "onApplicationStart", vec![]);
        }

        // 5b. Session lifecycle
        if session_management {
            if let Some(ref server_state) = self.server_state.clone() {
                let sid = self.session_id.clone().unwrap_or_default();
                if !sid.is_empty() {
                    let mut sessions = server_state.sessions.lock().unwrap();
                    let is_new = !sessions.contains_key(&sid);

                    if is_new {
                        // Create new session
                        sessions.insert(sid.clone(), SessionData {
                            variables: IndexMap::new(),
                            created: std::time::Instant::now(),
                            last_accessed: std::time::Instant::now(),
                            auth_user: None,
                            auth_roles: Vec::new(),
                            timeout_secs: session_timeout,
                        });
                        drop(sessions);

                        // Call onSessionStart
                        let _ = self.call_lifecycle_method(&template, "onSessionStart", vec![]);
                    } else {
                        // Update last_accessed
                        if let Some(session) = sessions.get_mut(&sid) {
                            session.last_accessed = std::time::Instant::now();
                            session.timeout_secs = session_timeout;
                        }
                        drop(sessions);
                    }
                }
            }
        }

        // 6. onRequestStart
        let target_page = self.source_file.clone().unwrap_or_default();
        match self.call_lifecycle_method(
            &template,
            "onRequestStart",
            vec![CfmlValue::String(target_page.clone())],
        ) {
            Err(e) if e.message == "__cfabort" || e.message == "__cflocation_redirect" => {
                return Ok(CfmlValue::Null);
            }
            _ => {}
        }

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
                Err(e) if e.message == "__cflocation_redirect" || e.message == "__cfabort" => Ok(CfmlValue::Null),
                Err(e) => Err(e),
            }
        } else {
            match self.execute() {
                Ok(v) => Ok(v),
                Err(e) if e.message == "__cflocation_redirect" || e.message == "__cfabort" => Ok(CfmlValue::Null),
                Err(e) => Err(e),
            }
        };

        // 8. onRequestEnd
        let _ = self.call_lifecycle_method(
            &template,
            "onRequestEnd",
            vec![CfmlValue::String(target_page)],
        );

        // 8b. Session expiry — scan and expire timed-out sessions
        if session_management {
            if let Some(ref server_state) = self.server_state.clone() {
                let expired: Vec<(String, IndexMap<String, CfmlValue>)> = {
                    let sessions = server_state.sessions.lock().unwrap();
                    sessions.iter()
                        .filter(|(_, s)| s.last_accessed.elapsed().as_secs() > s.timeout_secs)
                        .map(|(k, s)| (k.clone(), s.variables.clone()))
                        .collect()
                };
                if !expired.is_empty() {
                    let app_scope_val = self.application_scope.as_ref()
                        .and_then(|a| a.lock().ok().map(|s| CfmlValue::Struct(s.clone())))
                        .unwrap_or(CfmlValue::Struct(IndexMap::new()));
                    for (sid, session_vars) in &expired {
                        // Call onSessionEnd(sessionScope, applicationScope)
                        let _ = self.call_lifecycle_method(
                            &template,
                            "onSessionEnd",
                            vec![CfmlValue::Struct(session_vars.clone()), app_scope_val.clone()],
                        );
                        server_state.sessions.lock().unwrap().remove(sid);
                    }
                }
            }
        }

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

/// Simple wildcard matching: '*' matches any sequence of characters.
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (plen, tlen) = (p.len(), t.len());
    let (mut pi, mut ti) = (0, 0);
    let (mut star_pi, mut star_ti) = (usize::MAX, 0);

    while ti < tlen {
        if pi < plen && (p[pi] == t[ti] || p[pi] == '?') {
            pi += 1;
            ti += 1;
        } else if pi < plen && p[pi] == '*' {
            star_pi = pi;
            star_ti = ti;
            pi += 1;
        } else if star_pi != usize::MAX {
            pi = star_pi + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }
    while pi < plen && p[pi] == '*' {
        pi += 1;
    }
    pi == plen
}

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
