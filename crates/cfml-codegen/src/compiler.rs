//! CFML Code Generator - AST to bytecode

use cfml_compiler::ast::*;

pub struct CfmlCompiler {
    pub program: BytecodeProgram,
    /// Stack of (break_placeholder_indices, continue_placeholder_indices) for loops
    loop_stack: Vec<(Vec<usize>, Vec<usize>)>,
}

#[derive(Debug, Clone)]
pub struct BytecodeProgram {
    pub functions: Vec<BytecodeFunction>,
}

#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    pub name: String,
    pub params: Vec<String>,
    pub instructions: Vec<BytecodeOp>,
}

#[derive(Debug, Clone)]
pub enum BytecodeOp {
    // Literals
    Null,
    True,
    False,
    Integer(i64),
    Double(f64),
    String(String),

    // Variables
    LoadLocal(String),
    StoreLocal(String),
    LoadGlobal(String),
    StoreGlobal(String),

    // Stack
    Pop,
    Dup,
    Swap,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    IntDiv,
    Negate,

    // String
    Concat,

    // Comparison
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Contains,
    DoesNotContain,

    // Logical
    And,
    Or,
    Not,
    Xor,
    Eqv,
    Imp,

    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Call(usize),
    Return,

    // Collections
    BuildArray(usize),   // Build array from top N stack items
    BuildStruct(usize),  // Build struct from top N key-value pairs
    GetIndex,            // Get array[index] or struct[key]
    SetIndex,            // Set array[index] = value or struct[key] = value
    GetProperty(String), // Get object.property
    SetProperty(String), // Set object.property = value

    // Object
    NewObject(usize),  // arg_count for constructor

    // Function definition
    DefineFunction(usize), // Index into program.functions

    // Postfix ops
    Increment(String),  // Increment variable
    Decrement(String),  // Decrement variable

    // Exception handling
    TryStart(usize),    // Jump target for catch
    TryEnd,
    Throw,

    // Method call: object is on stack, then args, method name + arg count
    // Optional write-back: (object_var, Option<property_name>)
    //   - Some(("dog", None)) for dog.method() — write modified this back to dog
    //   - Some(("this", Some("items"))) for this.items.method() — write result back to this.items
    //   - None — no write-back needed
    CallMethod(String, usize, Option<(String, Option<String>)>),

    // For-in support
    GetKeys,  // Pop value: if struct, push array of keys; if array, leave as-is

    // Include
    Include(String),  // Include and execute a file

    // Null handling
    IsNull,                // Pop value, push bool (true if Null)
    JumpIfNotNull(usize),  // Pop value, jump if not null (pushes value back)

    // Output
    Print,
    Halt,
}

impl CfmlCompiler {
    pub fn new() -> Self {
        Self {
            program: BytecodeProgram {
                functions: vec![BytecodeFunction {
                    name: "__main__".to_string(),
                    params: Vec::new(),
                    instructions: Vec::new(),
                }],
            },
            loop_stack: Vec::new(),
        }
    }

    /// Determine write-back target for a method call from the AST.
    /// Returns Some((var_name, Some(prop_name))) for obj.prop.method()
    /// or Some((var_name, None)) for var.method()
    fn method_call_write_back(object: &Expression) -> Option<(String, Option<String>)> {
        match object {
            // this.items.method() or var.prop.method()
            Expression::MemberAccess(access) => {
                match &*access.object {
                    Expression::Identifier(ident) => {
                        Some((ident.name.clone(), Some(access.member.clone())))
                    }
                    Expression::This(_) => {
                        Some(("this".to_string(), Some(access.member.clone())))
                    }
                    _ => None,
                }
            }
            // var.method() or this.method()
            Expression::Identifier(ident) => {
                Some((ident.name.clone(), None))
            }
            Expression::This(_) => {
                Some(("this".to_string(), None))
            }
            _ => None,
        }
    }

    pub fn compile(mut self, ast: Program) -> BytecodeProgram {
        let mut instructions = Vec::new();

        for node in ast.statements {
            self.compile_node(&node, &mut instructions);
        }

        instructions.push(BytecodeOp::Halt);

        self.program.functions[0].instructions = instructions;

        self.program
    }

    fn compile_node(&mut self, node: &CfmlNode, instructions: &mut Vec<BytecodeOp>) {
        match node {
            CfmlNode::Statement(stmt) => self.compile_statement(stmt, instructions),
            CfmlNode::Expression(expr) => {
                self.compile_expression(expr, instructions);
                instructions.push(BytecodeOp::Pop);
            }
            _ => {}
        }
    }

    fn compile_statement(&mut self, stmt: &Statement, instructions: &mut Vec<BytecodeOp>) {
        match stmt {
            Statement::Expression(expr_stmt) => {
                self.compile_expression(&expr_stmt.expr, instructions);
                instructions.push(BytecodeOp::Pop);
            }
            Statement::Var(var) => {
                if let Some(value) = &var.value {
                    self.compile_expression(value, instructions);
                } else {
                    instructions.push(BytecodeOp::Null);
                }
                instructions.push(BytecodeOp::StoreLocal(var.name.clone()));
            }
            Statement::Assignment(assign) => {
                self.compile_expression(&assign.value, instructions);

                match &assign.operator {
                    AssignOp::PlusEqual => {
                        match &assign.target {
                            AssignTarget::Variable(name) => {
                                instructions.push(BytecodeOp::LoadLocal(name.clone()));
                            }
                            _ => {}
                        }
                        // Swap so we have: old_value, new_value -> old + new
                        let len = instructions.len();
                        instructions.swap(len - 2, len - 1);
                        instructions.push(BytecodeOp::Add);
                    }
                    AssignOp::MinusEqual => {
                        match &assign.target {
                            AssignTarget::Variable(name) => {
                                instructions.push(BytecodeOp::LoadLocal(name.clone()));
                            }
                            _ => {}
                        }
                        let len = instructions.len();
                        instructions.swap(len - 2, len - 1);
                        instructions.push(BytecodeOp::Sub);
                    }
                    AssignOp::StarEqual => {
                        match &assign.target {
                            AssignTarget::Variable(name) => {
                                instructions.push(BytecodeOp::LoadLocal(name.clone()));
                            }
                            _ => {}
                        }
                        let len = instructions.len();
                        instructions.swap(len - 2, len - 1);
                        instructions.push(BytecodeOp::Mul);
                    }
                    AssignOp::SlashEqual => {
                        match &assign.target {
                            AssignTarget::Variable(name) => {
                                instructions.push(BytecodeOp::LoadLocal(name.clone()));
                            }
                            _ => {}
                        }
                        let len = instructions.len();
                        instructions.swap(len - 2, len - 1);
                        instructions.push(BytecodeOp::Div);
                    }
                    AssignOp::ConcatEqual => {
                        match &assign.target {
                            AssignTarget::Variable(name) => {
                                instructions.push(BytecodeOp::LoadLocal(name.clone()));
                            }
                            _ => {}
                        }
                        let len = instructions.len();
                        instructions.swap(len - 2, len - 1);
                        instructions.push(BytecodeOp::Concat);
                    }
                    AssignOp::Equal => {} // Value already on stack
                }

                match &assign.target {
                    AssignTarget::Variable(name) => {
                        instructions.push(BytecodeOp::StoreLocal(name.clone()));
                    }
                    AssignTarget::ArrayAccess(arr, idx) => {
                        self.compile_expression(arr, instructions);
                        self.compile_expression(idx, instructions);
                        instructions.push(BytecodeOp::SetIndex);
                    }
                    AssignTarget::StructAccess(obj, member) => {
                        // Stack has [value]. SetProperty needs [obj, value].
                        // Compile obj, then swap so value is on top.
                        self.compile_expression(obj, instructions);
                        instructions.push(BytecodeOp::Swap);
                        instructions.push(BytecodeOp::SetProperty(member.clone()));
                        // SetProperty leaves modified obj on stack; store it back
                        match obj.as_ref() {
                            Expression::Identifier(ident) => {
                                instructions.push(BytecodeOp::StoreLocal(ident.name.clone()));
                            }
                            Expression::This(_) => {
                                instructions.push(BytecodeOp::StoreLocal("this".to_string()));
                            }
                            _ => {
                                // For nested access, just pop (mutation is lost)
                                instructions.push(BytecodeOp::Pop);
                            }
                        }
                    }
                }
            }
            Statement::Return(ret) => {
                if let Some(value) = &ret.value {
                    self.compile_expression(value, instructions);
                } else {
                    instructions.push(BytecodeOp::Null);
                }
                instructions.push(BytecodeOp::Return);
            }
            Statement::If(if_stmt) => {
                self.compile_if(if_stmt, instructions);
            }
            Statement::For(for_stmt) => {
                self.compile_for(for_stmt, instructions);
            }
            Statement::ForIn(for_in) => {
                self.compile_for_in(for_in, instructions);
            }
            Statement::While(while_stmt) => {
                self.compile_while(while_stmt, instructions);
            }
            Statement::Do(do_stmt) => {
                self.compile_do(do_stmt, instructions);
            }
            Statement::Switch(switch_stmt) => {
                self.compile_switch(switch_stmt, instructions);
            }
            Statement::Break(_) => {
                // Push a placeholder jump that will be patched later
                let idx = instructions.len();
                instructions.push(BytecodeOp::Jump(0)); // placeholder
                if let Some(loop_ctx) = self.loop_stack.last_mut() {
                    loop_ctx.0.push(idx); // break indices
                }
            }
            Statement::Continue(_) => {
                let idx = instructions.len();
                instructions.push(BytecodeOp::Jump(0)); // placeholder
                if let Some(loop_ctx) = self.loop_stack.last_mut() {
                    loop_ctx.1.push(idx); // continue indices
                }
            }
            Statement::Try(try_stmt) => {
                self.compile_try(try_stmt, instructions);
            }
            Statement::Throw(throw_stmt) => {
                if let Some(msg) = &throw_stmt.message {
                    self.compile_expression(msg, instructions);
                } else {
                    instructions.push(BytecodeOp::String("An error occurred".to_string()));
                }
                instructions.push(BytecodeOp::Throw);
            }
            Statement::FunctionDecl(func_decl) => {
                self.compile_function_decl(&func_decl.func, instructions);
            }
            Statement::ComponentDecl(comp_decl) => {
                // Compile component as a struct with methods
                self.compile_component(&comp_decl.component, instructions);
            }
            Statement::Include(inc) => {
                // Extract the path string from the include expression
                if let Expression::Literal(lit) = &inc.path {
                    if let LiteralValue::String(path) = &lit.value {
                        instructions.push(BytecodeOp::Include(path.clone()));
                    }
                }
            }
            Statement::Import(_) => {
                // Import not yet supported at bytecode level
            }
            Statement::Exit => {
                instructions.push(BytecodeOp::Halt);
            }
            _ => {}
        }
    }

    fn compile_if(&mut self, if_stmt: &If, instructions: &mut Vec<BytecodeOp>) {
        self.compile_expression(&if_stmt.condition, instructions);
        let jump_false_idx = instructions.len();
        instructions.push(BytecodeOp::JumpIfFalse(0)); // placeholder

        // Then branch
        for s in &if_stmt.then_branch {
            self.compile_statement(s, instructions);
        }

        if !if_stmt.else_if.is_empty() || if_stmt.else_branch.is_some() {
            let jump_end_idx = instructions.len();
            instructions.push(BytecodeOp::Jump(0)); // placeholder

            // Patch the jump-to-else
            instructions[jump_false_idx] = BytecodeOp::JumpIfFalse(instructions.len());

            // Else-if chains
            let mut end_jumps = vec![jump_end_idx];

            for (i, else_if) in if_stmt.else_if.iter().enumerate() {
                self.compile_expression(&else_if.condition, instructions);
                let jf_idx = instructions.len();
                instructions.push(BytecodeOp::JumpIfFalse(0));

                for s in &else_if.body {
                    self.compile_statement(s, instructions);
                }

                let je_idx = instructions.len();
                instructions.push(BytecodeOp::Jump(0));
                end_jumps.push(je_idx);

                instructions[jf_idx] = BytecodeOp::JumpIfFalse(instructions.len());
            }

            // Else branch
            if let Some(else_branch) = &if_stmt.else_branch {
                for s in else_branch {
                    self.compile_statement(s, instructions);
                }
            }

            // Patch all end jumps
            let end_pos = instructions.len();
            for idx in end_jumps {
                instructions[idx] = BytecodeOp::Jump(end_pos);
            }
        } else {
            instructions[jump_false_idx] = BytecodeOp::JumpIfFalse(instructions.len());
        }
    }

    fn compile_for(&mut self, for_stmt: &For, instructions: &mut Vec<BytecodeOp>) {
        // Init
        if let Some(init) = &for_stmt.init {
            self.compile_statement(init, instructions);
        }

        // Loop start
        let loop_start = instructions.len();

        // Condition
        if let Some(condition) = &for_stmt.condition {
            self.compile_expression(condition, instructions);
            let jump_false_idx = instructions.len();
            instructions.push(BytecodeOp::JumpIfFalse(0)); // placeholder

            // Push loop context for break/continue
            self.loop_stack.push((Vec::new(), Vec::new()));

            // Body
            for s in &for_stmt.body {
                self.compile_statement(s, instructions);
            }

            // Continue target
            let continue_target = instructions.len();

            // Increment
            if let Some(increment) = &for_stmt.increment {
                self.compile_expression(increment, instructions);
                instructions.push(BytecodeOp::Pop);
            }

            // Jump back to condition
            instructions.push(BytecodeOp::Jump(loop_start));

            // End of loop
            let loop_end = instructions.len();
            instructions[jump_false_idx] = BytecodeOp::JumpIfFalse(loop_end);

            // Patch break/continue
            let (break_indices, continue_indices) = self.loop_stack.pop().unwrap();
            for idx in break_indices {
                instructions[idx] = BytecodeOp::Jump(loop_end);
            }
            for idx in continue_indices {
                instructions[idx] = BytecodeOp::Jump(continue_target);
            }
        }
    }

    fn compile_for_in(&mut self, for_in: &ForIn, instructions: &mut Vec<BytecodeOp>) {
        // Compile iterable
        self.compile_expression(&for_in.iterable, instructions);

        // GetKeys: if struct, convert to array of keys; arrays pass through unchanged
        instructions.push(BytecodeOp::GetKeys);

        // Store iterable in a temp variable
        let iter_var = format!("__iter_{}", instructions.len());
        let idx_var = format!("__idx_{}", instructions.len());
        instructions.push(BytecodeOp::StoreLocal(iter_var.clone()));
        // CFML arrays are 1-based, so start index at 1
        instructions.push(BytecodeOp::Integer(1));
        instructions.push(BytecodeOp::StoreLocal(idx_var.clone()));

        let loop_start = instructions.len();

        // Check: idx <= len(iterable)
        instructions.push(BytecodeOp::LoadLocal(idx_var.clone()));
        instructions.push(BytecodeOp::LoadGlobal("len".to_string()));
        instructions.push(BytecodeOp::LoadLocal(iter_var.clone()));
        instructions.push(BytecodeOp::Call(1));
        instructions.push(BytecodeOp::Lte);

        let jump_false_idx = instructions.len();
        instructions.push(BytecodeOp::JumpIfFalse(0));

        // Set loop variable = iterable[idx]
        instructions.push(BytecodeOp::LoadLocal(iter_var.clone()));
        instructions.push(BytecodeOp::LoadLocal(idx_var.clone()));
        instructions.push(BytecodeOp::GetIndex);
        instructions.push(BytecodeOp::StoreLocal(for_in.variable.clone()));

        self.loop_stack.push((Vec::new(), Vec::new()));

        for s in &for_in.body {
            self.compile_statement(s, instructions);
        }

        let continue_target = instructions.len();

        // idx++
        instructions.push(BytecodeOp::LoadLocal(idx_var.clone()));
        instructions.push(BytecodeOp::Integer(1));
        instructions.push(BytecodeOp::Add);
        instructions.push(BytecodeOp::StoreLocal(idx_var.clone()));

        instructions.push(BytecodeOp::Jump(loop_start));

        let loop_end = instructions.len();
        instructions[jump_false_idx] = BytecodeOp::JumpIfFalse(loop_end);

        let (break_indices, continue_indices) = self.loop_stack.pop().unwrap();
        for idx in break_indices {
            instructions[idx] = BytecodeOp::Jump(loop_end);
        }
        for idx in continue_indices {
            instructions[idx] = BytecodeOp::Jump(continue_target);
        }
    }

    fn compile_while(&mut self, while_stmt: &While, instructions: &mut Vec<BytecodeOp>) {
        let loop_start = instructions.len();

        self.compile_expression(&while_stmt.condition, instructions);
        let jump_false_idx = instructions.len();
        instructions.push(BytecodeOp::JumpIfFalse(0));

        self.loop_stack.push((Vec::new(), Vec::new()));

        for s in &while_stmt.body {
            self.compile_statement(s, instructions);
        }

        instructions.push(BytecodeOp::Jump(loop_start));

        let loop_end = instructions.len();
        instructions[jump_false_idx] = BytecodeOp::JumpIfFalse(loop_end);

        let (break_indices, continue_indices) = self.loop_stack.pop().unwrap();
        for idx in break_indices {
            instructions[idx] = BytecodeOp::Jump(loop_end);
        }
        for idx in continue_indices {
            instructions[idx] = BytecodeOp::Jump(loop_start);
        }
    }

    fn compile_do(&mut self, do_stmt: &Do, instructions: &mut Vec<BytecodeOp>) {
        let loop_start = instructions.len();

        self.loop_stack.push((Vec::new(), Vec::new()));

        for s in &do_stmt.body {
            self.compile_statement(s, instructions);
        }

        let continue_target = instructions.len();

        self.compile_expression(&do_stmt.condition, instructions);
        instructions.push(BytecodeOp::JumpIfTrue(loop_start));

        let loop_end = instructions.len();

        let (break_indices, continue_indices) = self.loop_stack.pop().unwrap();
        for idx in break_indices {
            instructions[idx] = BytecodeOp::Jump(loop_end);
        }
        for idx in continue_indices {
            instructions[idx] = BytecodeOp::Jump(continue_target);
        }
    }

    fn compile_switch(&mut self, switch_stmt: &Switch, instructions: &mut Vec<BytecodeOp>) {
        // Evaluate switch expression and store
        self.compile_expression(&switch_stmt.expression, instructions);
        let switch_var = format!("__switch_{}", instructions.len());
        instructions.push(BytecodeOp::StoreLocal(switch_var.clone()));

        self.loop_stack.push((Vec::new(), Vec::new())); // break support

        let mut end_jumps = Vec::new();
        let mut next_case_jump: Option<usize> = None;

        for case in &switch_stmt.cases {
            // Patch previous case's fall-through check
            if let Some(prev_jump) = next_case_jump {
                instructions[prev_jump] = BytecodeOp::JumpIfFalse(instructions.len());
            }

            // Compare switch value to case value(s)
            // For multiple values, OR them together
            for (i, val) in case.values.iter().enumerate() {
                instructions.push(BytecodeOp::LoadLocal(switch_var.clone()));
                self.compile_expression(val, instructions);
                instructions.push(BytecodeOp::Eq);

                if i > 0 {
                    instructions.push(BytecodeOp::Or);
                }
            }

            next_case_jump = Some(instructions.len());
            instructions.push(BytecodeOp::JumpIfFalse(0));

            for s in &case.body {
                self.compile_statement(s, instructions);
            }

            // Jump to end after case body (unless there's a break which does this too)
            let je_idx = instructions.len();
            instructions.push(BytecodeOp::Jump(0));
            end_jumps.push(je_idx);
        }

        // Patch last case jump
        if let Some(prev_jump) = next_case_jump {
            instructions[prev_jump] = BytecodeOp::JumpIfFalse(instructions.len());
        }

        // Default case
        if let Some(default) = &switch_stmt.default_case {
            for s in default {
                self.compile_statement(s, instructions);
            }
        }

        let end_pos = instructions.len();
        for idx in end_jumps {
            instructions[idx] = BytecodeOp::Jump(end_pos);
        }

        // Patch break statements
        let (break_indices, _) = self.loop_stack.pop().unwrap();
        for idx in break_indices {
            instructions[idx] = BytecodeOp::Jump(end_pos);
        }
    }

    fn compile_try(&mut self, try_stmt: &Try, instructions: &mut Vec<BytecodeOp>) {
        // TryStart points to catch handler
        let try_start_idx = instructions.len();
        instructions.push(BytecodeOp::TryStart(0)); // placeholder

        // Try body
        for s in &try_stmt.body {
            self.compile_statement(s, instructions);
        }
        instructions.push(BytecodeOp::TryEnd);

        // Jump over catch blocks
        let jump_over_catch = instructions.len();
        instructions.push(BytecodeOp::Jump(0));

        // Catch handler
        let catch_start = instructions.len();
        instructions[try_start_idx] = BytecodeOp::TryStart(catch_start);

        for catch in &try_stmt.catches {
            // The error value will be on the stack
            instructions.push(BytecodeOp::StoreLocal(catch.var_name.clone()));

            for s in &catch.body {
                self.compile_statement(s, instructions);
            }
        }

        let end_pos = instructions.len();
        instructions[jump_over_catch] = BytecodeOp::Jump(end_pos);

        // Finally
        if let Some(finally_body) = &try_stmt.finally_body {
            for s in finally_body {
                self.compile_statement(s, instructions);
            }
        }
    }

    fn compile_function_decl(&mut self, func: &Function, instructions: &mut Vec<BytecodeOp>) {
        // Compile the function body into a separate BytecodeFunction
        let mut func_instructions = Vec::new();

        for s in &func.body {
            self.compile_statement(s, &mut func_instructions);
        }

        // Ensure function returns null if no explicit return
        func_instructions.push(BytecodeOp::Null);
        func_instructions.push(BytecodeOp::Return);

        let bc_func = BytecodeFunction {
            name: func.name.clone(),
            params: func.params.iter().map(|p| p.name.clone()).collect(),
            instructions: func_instructions,
        };

        let func_idx = self.program.functions.len();
        self.program.functions.push(bc_func);

        // Define the function in current scope
        instructions.push(BytecodeOp::DefineFunction(func_idx));
        instructions.push(BytecodeOp::StoreLocal(func.name.clone()));
    }

    fn compile_component(&mut self, component: &Component, instructions: &mut Vec<BytecodeOp>) {
        // Build the component as a struct containing:
        // 1. Property defaults
        // 2. Compiled methods as function references
        let mut prop_count = 0;

        // Add properties with defaults
        for prop in &component.properties {
            instructions.push(BytecodeOp::String(prop.name.clone()));
            if let Some(default) = &prop.default {
                self.compile_expression(default, instructions);
            } else {
                instructions.push(BytecodeOp::Null);
            }
            prop_count += 1;
        }

        // Add __name metadata
        instructions.push(BytecodeOp::String("__name".to_string()));
        instructions.push(BytecodeOp::String(component.name.clone()));
        prop_count += 1;

        // Build the base struct
        instructions.push(BytecodeOp::BuildStruct(prop_count));

        // Store as a component template in both local and global scope
        instructions.push(BytecodeOp::StoreLocal(component.name.clone()));
        instructions.push(BytecodeOp::LoadLocal(component.name.clone()));
        instructions.push(BytecodeOp::StoreGlobal(component.name.clone()));

        // Compile component methods and add them to the component struct
        for func in &component.functions {
            self.compile_function_decl(func, instructions);
            // SetProperty needs: stack = [object, value]
            // Load the component struct, then load the function ref
            instructions.push(BytecodeOp::LoadLocal(component.name.clone()));
            instructions.push(BytecodeOp::LoadLocal(func.name.clone()));
            instructions.push(BytecodeOp::SetProperty(func.name.clone()));
            instructions.push(BytecodeOp::StoreLocal(component.name.clone()));
        }

        // Update global copy after methods are added
        if !component.functions.is_empty() {
            instructions.push(BytecodeOp::LoadLocal(component.name.clone()));
            instructions.push(BytecodeOp::StoreGlobal(component.name.clone()));
        }
    }

    fn compile_expression(&mut self, expr: &Expression, instructions: &mut Vec<BytecodeOp>) {
        match expr {
            Expression::Literal(lit) => match &lit.value {
                LiteralValue::Null => instructions.push(BytecodeOp::Null),
                LiteralValue::Bool(true) => instructions.push(BytecodeOp::True),
                LiteralValue::Bool(false) => instructions.push(BytecodeOp::False),
                LiteralValue::Int(i) => instructions.push(BytecodeOp::Integer(*i)),
                LiteralValue::Double(d) => instructions.push(BytecodeOp::Double(*d)),
                LiteralValue::String(s) => instructions.push(BytecodeOp::String(s.clone())),
            },
            Expression::Identifier(id) => {
                instructions.push(BytecodeOp::LoadLocal(id.name.clone()));
            }
            Expression::BinaryOp(binop) => {
                if binop.operator == BinaryOpType::Assign {
                    self.compile_expression(&binop.right, instructions);

                    match &*binop.left {
                        Expression::Identifier(ident) => {
                            instructions.push(BytecodeOp::StoreLocal(ident.name.clone()));
                        }
                        Expression::MemberAccess(access) => {
                            // Stack has [value]. SetProperty needs [obj, value].
                            self.compile_expression(&access.object, instructions);
                            instructions.push(BytecodeOp::Swap);
                            instructions.push(BytecodeOp::SetProperty(access.member.clone()));
                            // Store modified object back to its variable
                            match &*access.object {
                                Expression::Identifier(ident) => {
                                    instructions.push(BytecodeOp::StoreLocal(ident.name.clone()));
                                }
                                Expression::This(_) => {
                                    instructions.push(BytecodeOp::StoreLocal("this".to_string()));
                                }
                                _ => {
                                    instructions.push(BytecodeOp::Pop);
                                }
                            }
                        }
                        Expression::ArrayAccess(access) => {
                            self.compile_expression(&access.array, instructions);
                            self.compile_expression(&access.index, instructions);
                            instructions.push(BytecodeOp::SetIndex);
                        }
                        _ => {}
                    }
                    return;
                }

                self.compile_expression(&binop.left, instructions);
                self.compile_expression(&binop.right, instructions);

                let op = match binop.operator {
                    BinaryOpType::Add => BytecodeOp::Add,
                    BinaryOpType::Sub => BytecodeOp::Sub,
                    BinaryOpType::Mul => BytecodeOp::Mul,
                    BinaryOpType::Div => BytecodeOp::Div,
                    BinaryOpType::Mod => BytecodeOp::Mod,
                    BinaryOpType::Pow => BytecodeOp::Pow,
                    BinaryOpType::IntDiv => BytecodeOp::IntDiv,
                    BinaryOpType::Concat => BytecodeOp::Concat,
                    BinaryOpType::Equal => BytecodeOp::Eq,
                    BinaryOpType::NotEqual => BytecodeOp::Neq,
                    BinaryOpType::Less => BytecodeOp::Lt,
                    BinaryOpType::LessEqual => BytecodeOp::Lte,
                    BinaryOpType::Greater => BytecodeOp::Gt,
                    BinaryOpType::GreaterEqual => BytecodeOp::Gte,
                    BinaryOpType::And => BytecodeOp::And,
                    BinaryOpType::Or => BytecodeOp::Or,
                    BinaryOpType::Xor => BytecodeOp::Xor,
                    BinaryOpType::Contains => BytecodeOp::Contains,
                    BinaryOpType::DoesNotContain => BytecodeOp::DoesNotContain,
                    BinaryOpType::Eqv => BytecodeOp::Eqv,
                    BinaryOpType::Imp => BytecodeOp::Imp,
                    BinaryOpType::Assign => BytecodeOp::Null, // Should not reach here
                };
                instructions.push(op);
            }
            Expression::UnaryOp(unary) => {
                self.compile_expression(&unary.operand, instructions);
                let op = match unary.operator {
                    UnaryOpType::Minus => BytecodeOp::Negate,
                    UnaryOpType::Not => BytecodeOp::Not,
                    UnaryOpType::BitNot => BytecodeOp::Not,
                };
                instructions.push(op);
            }
            Expression::PostfixOp(postfix) => {
                if let Expression::Identifier(ident) = &*postfix.operand {
                    match postfix.operator {
                        PostfixOpType::Increment => {
                            instructions.push(BytecodeOp::LoadLocal(ident.name.clone()));
                            instructions.push(BytecodeOp::Dup);
                            instructions.push(BytecodeOp::Integer(1));
                            instructions.push(BytecodeOp::Add);
                            instructions.push(BytecodeOp::StoreLocal(ident.name.clone()));
                            // The original value stays on the stack
                        }
                        PostfixOpType::Decrement => {
                            instructions.push(BytecodeOp::LoadLocal(ident.name.clone()));
                            instructions.push(BytecodeOp::Dup);
                            instructions.push(BytecodeOp::Integer(1));
                            instructions.push(BytecodeOp::Sub);
                            instructions.push(BytecodeOp::StoreLocal(ident.name.clone()));
                        }
                    }
                }
            }
            Expression::MemberAccess(access) => {
                self.compile_expression(&access.object, instructions);
                if access.null_safe {
                    // Null-safe: if object is null, skip property access (null stays on stack)
                    // JumpIfNotNull peeks without popping, so no Dup needed
                    let jump_idx = instructions.len();
                    instructions.push(BytecodeOp::JumpIfNotNull(0)); // placeholder
                    // Object is null - it's on the stack, skip the GetProperty
                    let jump_end = instructions.len();
                    instructions.push(BytecodeOp::Jump(0)); // placeholder
                    // Object is not null - do the property access
                    instructions[jump_idx] = BytecodeOp::JumpIfNotNull(instructions.len());
                    instructions.push(BytecodeOp::GetProperty(access.member.clone()));
                    instructions[jump_end] = BytecodeOp::Jump(instructions.len());
                } else {
                    instructions.push(BytecodeOp::GetProperty(access.member.clone()));
                }
            }
            Expression::ArrayAccess(access) => {
                self.compile_expression(&access.array, instructions);
                self.compile_expression(&access.index, instructions);
                instructions.push(BytecodeOp::GetIndex);
            }
            Expression::FunctionCall(call) => {
                // Push function reference first
                if let Expression::Identifier(ident) = &*call.name {
                    instructions.push(BytecodeOp::LoadGlobal(ident.name.clone()));
                } else {
                    self.compile_expression(&call.name, instructions);
                }
                // Push arguments
                for arg in &call.arguments {
                    self.compile_expression(arg, instructions);
                }
                instructions.push(BytecodeOp::Call(call.arguments.len()));
            }
            Expression::MethodCall(call) => {
                // Determine write-back target from the AST.
                // this.items.append(x) → write_back = Some(("this", Some("items")))
                // dog.method(x)        → write_back = Some(("dog", None))
                let write_back = Self::method_call_write_back(&call.object);

                self.compile_expression(&call.object, instructions);
                if call.null_safe {
                    let jump_idx = instructions.len();
                    instructions.push(BytecodeOp::JumpIfNotNull(0));
                    let jump_end = instructions.len();
                    instructions.push(BytecodeOp::Jump(0));
                    instructions[jump_idx] = BytecodeOp::JumpIfNotNull(instructions.len());
                    for arg in &call.arguments {
                        self.compile_expression(arg, instructions);
                    }
                    instructions.push(BytecodeOp::CallMethod(
                        call.method.clone(),
                        call.arguments.len(),
                        write_back.clone(),
                    ));
                    instructions[jump_end] = BytecodeOp::Jump(instructions.len());
                } else {
                    for arg in &call.arguments {
                        self.compile_expression(arg, instructions);
                    }
                    instructions.push(BytecodeOp::CallMethod(
                        call.method.clone(),
                        call.arguments.len(),
                        write_back,
                    ));
                }
            }
            Expression::Array(arr) => {
                for elem in &arr.elements {
                    self.compile_expression(elem, instructions);
                }
                instructions.push(BytecodeOp::BuildArray(arr.elements.len()));
            }
            Expression::Struct(st) => {
                for (key, value) in &st.pairs {
                    // Struct literal keys: identifiers should be treated as string keys,
                    // not variable lookups (e.g., {name: "Alex"} -> key is string "name")
                    match key {
                        Expression::Identifier(ident) => {
                            instructions.push(BytecodeOp::String(ident.name.clone()));
                        }
                        _ => {
                            self.compile_expression(key, instructions);
                        }
                    }
                    self.compile_expression(value, instructions);
                }
                instructions.push(BytecodeOp::BuildStruct(st.pairs.len()));
            }
            Expression::Ternary(tern) => {
                self.compile_expression(&tern.condition, instructions);
                let jump_false = instructions.len();
                instructions.push(BytecodeOp::JumpIfFalse(0));

                self.compile_expression(&tern.then_expr, instructions);
                let jump_end = instructions.len();
                instructions.push(BytecodeOp::Jump(0));

                instructions[jump_false] = BytecodeOp::JumpIfFalse(instructions.len());
                self.compile_expression(&tern.else_expr, instructions);
                instructions[jump_end] = BytecodeOp::Jump(instructions.len());
            }
            Expression::New(new_expr) => {
                // Parser may parse `new Dog(args)` as class=FunctionCall(Dog, args)
                // Extract the class name and push it for VM resolution
                match &*new_expr.class {
                    Expression::FunctionCall(call) => {
                        // Push class name as string for VM to resolve
                        if let Expression::Identifier(ident) = &*call.name {
                            instructions.push(BytecodeOp::String(ident.name.clone()));
                        } else {
                            self.compile_expression(&call.name, instructions);
                        }
                        for arg in &call.arguments {
                            self.compile_expression(arg, instructions);
                        }
                        instructions.push(BytecodeOp::NewObject(call.arguments.len()));
                    }
                    Expression::Identifier(ident) => {
                        // Push class name as string - VM will look up in locals, globals, or .cfc files
                        instructions.push(BytecodeOp::String(ident.name.clone()));
                        for arg in &new_expr.arguments {
                            self.compile_expression(arg, instructions);
                        }
                        instructions.push(BytecodeOp::NewObject(new_expr.arguments.len()));
                    }
                    _ => {
                        self.compile_expression(&new_expr.class, instructions);
                        for arg in &new_expr.arguments {
                            self.compile_expression(arg, instructions);
                        }
                        instructions.push(BytecodeOp::NewObject(new_expr.arguments.len()));
                    }
                }
            }
            Expression::Closure(closure) => {
                // Compile closure body into separate function
                let mut func_instructions = Vec::new();
                for s in &closure.body {
                    self.compile_statement(s, &mut func_instructions);
                }
                func_instructions.push(BytecodeOp::Null);
                func_instructions.push(BytecodeOp::Return);

                let func_name = format!("__closure_{}", self.program.functions.len());
                let bc_func = BytecodeFunction {
                    name: func_name.clone(),
                    params: closure.params.iter().map(|p| p.name.clone()).collect(),
                    instructions: func_instructions,
                };

                let func_idx = self.program.functions.len();
                self.program.functions.push(bc_func);
                instructions.push(BytecodeOp::DefineFunction(func_idx));
            }
            Expression::ArrowFunction(arrow) => {
                let mut func_instructions = Vec::new();
                self.compile_expression(&arrow.body, &mut func_instructions);
                func_instructions.push(BytecodeOp::Return);

                let func_name = format!("__arrow_{}", self.program.functions.len());
                let bc_func = BytecodeFunction {
                    name: func_name.clone(),
                    params: arrow.params.iter().map(|p| p.name.clone()).collect(),
                    instructions: func_instructions,
                };

                let func_idx = self.program.functions.len();
                self.program.functions.push(bc_func);
                instructions.push(BytecodeOp::DefineFunction(func_idx));
            }
            Expression::This(_) => {
                instructions.push(BytecodeOp::LoadLocal("this".to_string()));
            }
            Expression::Super(_) => {
                instructions.push(BytecodeOp::LoadLocal("super".to_string()));
            }
            Expression::StringInterpolation(interp) => {
                if interp.parts.is_empty() {
                    instructions.push(BytecodeOp::String(String::new()));
                } else {
                    // Compile first part
                    self.compile_expression(&interp.parts[0], instructions);
                    // Convert to string via Concat with empty string if needed
                    if !matches!(&interp.parts[0], Expression::Literal(Literal { value: LiteralValue::String(_), .. })) {
                        instructions.push(BytecodeOp::String(String::new()));
                        instructions.push(BytecodeOp::Concat);
                    }
                    // Concat remaining parts
                    for part in &interp.parts[1..] {
                        self.compile_expression(part, instructions);
                        instructions.push(BytecodeOp::Concat);
                    }
                }
            }
            Expression::Elvis(elvis) => {
                // Elvis operator: left ?: right
                // Eval left, if not null use it, otherwise eval right
                // JumpIfNotNull peeks without popping, so no Dup needed
                self.compile_expression(&elvis.left, instructions);
                let jump_idx = instructions.len();
                instructions.push(BytecodeOp::JumpIfNotNull(0)); // placeholder
                // Left is null, pop the null and eval right
                instructions.push(BytecodeOp::Pop);
                self.compile_expression(&elvis.right, instructions);
                instructions[jump_idx] = BytecodeOp::JumpIfNotNull(instructions.len());
            }
            Expression::Empty => {
                instructions.push(BytecodeOp::Null);
            }
            _ => {
                instructions.push(BytecodeOp::Null);
            }
        }
    }
}

impl Default for CfmlCompiler {
    fn default() -> Self {
        Self::new()
    }
}
