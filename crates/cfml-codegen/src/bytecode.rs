//! CFML Bytecode definitions

use cfml_common::position::SourceLocation;

#[derive(Debug, Clone)]
pub enum Opcode {
    // Stack operations
    Pop,
    Dup,
    Swap,

    // Constants
    Null,
    True,
    False,
    Integer(i64),
    Double(f64),
    String(String),

    // Collections
    NewArray,
    NewStruct,
    ArraySet,
    StructSet,

    // Variables
    LoadLocal(String),
    StoreLocal(String),
    LoadGlobal(String),
    StoreGlobal(String),
    LoadUpvalue(usize),
    StoreUpvalue(usize),

    // Object operations
    NewObject(String),
    NewComponent(String),
    GetProperty(String),
    SetProperty(String),
    GetMethod(String),
    CallMethod(String, usize),
    GetSuper,
    GetThis,

    // Function calls
    Call(usize),
    Return,
    TailCall(usize),

    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Loop(usize),

    // Exception handling
    Try,
    Catch(String),
    Finally,
    Throw,

    // Scope
    EnterScope,
    ExitScope,
    Import(String),

    // Misc
    Print,
    Debug,
    Halt,
}

#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    pub name: String,
    pub params: Vec<String>,
    pub locals: Vec<String>,
    pub instructions: Vec<Instruction>,
    pub line_numbers: Vec<(usize, usize)>,
}

impl BytecodeFunction {
    pub fn new(name: String) -> Self {
        Self {
            name,
            params: Vec::new(),
            locals: Vec::new(),
            instructions: Vec::new(),
            line_numbers: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, opcode: Opcode, line: usize) -> usize {
        let pos = self.instructions.len();
        self.instructions.push(Instruction {
            opcode,
            location: SourceLocation::default(),
        });
        self.line_numbers.push((pos, line));
        pos
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub location: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct BytecodeProgram {
    pub functions: Vec<BytecodeFunction>,
    pub constants: Vec<Constant>,
    pub source_name: Option<String>,
}

impl BytecodeProgram {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            constants: Vec::new(),
            source_name: None,
        }
    }

    pub fn add_constant(&mut self, constant: Constant) -> usize {
        let idx = self.constants.len();
        self.constants.push(constant);
        idx
    }
}

impl Default for BytecodeProgram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Null,
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    Function(BytecodeFunction),
}
