//! CFML Virtual Machine - Bytecode execution engine

use cfml_codegen::{BytecodeFunction, BytecodeOp, BytecodeProgram};
use cfml_common::dynamic::CfmlValue;
use cfml_common::vm::{CfmlError, CfmlResult};
use std::collections::HashMap;

pub type BuiltinFunction = fn(Vec<CfmlValue>) -> CfmlResult;

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
    call_stack: Vec<CallFrame>,
    /// Try-catch handler stack
    try_stack: Vec<TryHandler>,
    /// Current exception (if any)
    current_exception: Option<CfmlValue>,
    /// After a component method executes, holds the modified `this` for write-back
    /// to the caller's object variable. Set by execute_function_with_args.
    method_this_writeback: Option<CfmlValue>,
}

#[derive(Debug, Clone)]
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
                        locals.insert(name, val);
                    }
                }
                BytecodeOp::LoadGlobal(name) => {
                    // Check locals first (for user-defined functions that shadow builtins)
                    if let Some(val) = locals.get(&name) {
                        stack.push(val.clone());
                    } else if let Some(val) = self.globals.get(&name) {
                        stack.push(val.clone());
                    } else if self.builtins.contains_key(&name) || self.user_functions.contains_key(&name) {
                        // Push a function reference
                        stack.push(CfmlValue::Function(cfml_common::dynamic::CfmlFunction {
                            name: name.clone(),
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
                        let instance = if let CfmlValue::Struct(s) = &class_ref {
                            // class_ref is already a struct (e.g. from LoadLocal("Dog"))
                            CfmlValue::Struct(s.clone())
                        } else {
                            // class_ref is a string name - look it up
                            let class_name = match &class_ref {
                                CfmlValue::Function(f) => f.name.clone(),
                                CfmlValue::String(s) => s.clone(),
                                _ => class_ref.as_string(),
                            };

                            let component = locals.get(&class_name)
                                .or_else(|| self.globals.get(&class_name))
                                .cloned();

                            if let Some(CfmlValue::Struct(s)) = component {
                                CfmlValue::Struct(s)
                            } else {
                                // Try loading a .cfc file
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
                                        let _ = self.execute_function_with_args(&cfc_func, Vec::new(), Some(&locals));
                                        // Merge sub-program's functions into the main program
                                        // so user_functions indices remain valid
                                        let sub_funcs = self.program.functions.clone();
                                        self.program = old_program;
                                        let base_idx = self.program.functions.len();
                                        for func in sub_funcs {
                                            if func.name != "__main__" {
                                                self.program.functions.push(func.clone());
                                                // Update user_functions to point to new index
                                                if let Some(old_idx) = self.user_functions.get(&func.name).cloned() {
                                                    let new_idx = base_idx + old_idx;
                                                    // Only update if the old index was in the sub-program range
                                                    self.user_functions.insert(func.name.clone(), self.program.functions.len() - 1);
                                                }
                                            }
                                        }
                                        // The component should now be in globals
                                        // Try exact name, case-insensitive, or "Anonymous" (for unnamed components)
                                        if let Some(CfmlValue::Struct(s)) = self.globals.get(&class_name).cloned()
                                            .or_else(|| {
                                                let lower = class_name.to_lowercase();
                                                self.globals.iter()
                                                    .find(|(k, _)| k.to_lowercase() == lower)
                                                    .map(|(_, v)| v.clone())
                                            })
                                            .or_else(|| self.globals.get("Anonymous").cloned())
                                        {
                                            CfmlValue::Struct(s)
                                        } else {
                                            CfmlValue::Struct(HashMap::new())
                                        }
                                    } else {
                                        CfmlValue::Struct(HashMap::new())
                                    }
                                } else {
                                    CfmlValue::Struct(HashMap::new())
                                }
                            }
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
                                        if let CfmlValue::Struct(_) = &result {
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
                    // Push function reference
                    stack.push(CfmlValue::Function(cfml_common::dynamic::CfmlFunction {
                        name: func_name,
                        params: Vec::new(),
                        body: cfml_common::dynamic::CfmlClosureBody::Expression(
                            Box::new(CfmlValue::Null),
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

                    let result = self.call_member_function(&object, &method_name, &mut extra_args)?;

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
                | "structeach" | "structmap" | "structfilter" => {
                    // Will be handled at the end of this function
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

            // Check user-defined functions
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
        matches!(method.to_lowercase().as_str(),
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
        let prop = object.get(method).unwrap_or(CfmlValue::Null);
        if let CfmlValue::Function(_) = &prop {
            let func_ref = prop.clone();
            let args: Vec<CfmlValue> = extra_args.drain(..).collect();
            // Bind 'this' to the object for component method calls
            let mut method_locals = HashMap::new();
            method_locals.insert("this".to_string(), object.clone());
            return self.call_function(&func_ref, args, &method_locals);
        }

        Ok(CfmlValue::Null)
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
