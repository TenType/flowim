use crate::objects::Function;

#[derive(Clone, PartialEq)]
pub enum Value {
    Void,
    Bool(bool),
    Int(isize),
    Float(f64),
    Str(String),
    Fun(Function),
}

use std::fmt::{Display, Formatter, Result};
impl Display for Value {
    fn fmt(&self, format: &mut Formatter<'_>) -> Result {
        use Value::*;
        match self {
            Bool(value) => write!(format, "{}", value),
            Int(value) => write!(format, "{}", value),
            Float(value) => {
                if value.floor() == *value {
                    write!(format, "{}.0", value)
                } else {
                    write!(format, "{}", value)
                }
            }
            Str(value) => write!(format, "{}", value),
            Fun(value) => write!(format, "{}", value),
            Void => write!(format, "void"),
        }
    }
}

pub fn type_as_str<'a>(value: Value) -> &'a str {
    use Value::*;
    match value {
        Bool(_) => "bool",
        Int(_) => "int",
        Float(_) => "float",
        Str(_) => "str",
        Fun(_) => "fun",
        Void => "void",
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpCode {
    Constant(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Not,
    Return,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    Jump(usize),
    JumpIfFalse(usize),
    JumpBack(usize),
    DefineGlobal(usize),
    GetGlobal(usize),
    SetGlobal(usize),
    GetLocal(usize),
    SetLocal(usize),
    Call(usize),
}

#[derive(Clone, PartialEq)]
pub struct Chunk {
    pub lines: Vec<usize>,
    pub constants: Vec<Value>,
    pub code: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            constants: Vec::new(),
            code: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> OpCode {
        self.constants.push(value);
        OpCode::Constant(self.constants.len() - 1)
    }

    #[cfg(debug_assertions)]
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for (i, instruction) in self.code.iter().enumerate() {
            self.disassemble_op(instruction, i);
        }
    }

    #[cfg(debug_assertions)]
    fn disassemble_constant(&self, name: &str, index: usize) {
        println!("{:<16} {:>4} ({})", name, index, self.constants[index]);
    }

    #[cfg(debug_assertions)]
    fn disassemble_large(&self, name: &str, index: usize) {
        println!("{:<16} {:>4}", name, index);
    }

    #[cfg(debug_assertions)]
    pub fn disassemble_op(&self, instruction: &OpCode, i: usize) {
        print!("{:04} ", i);
        if i > 0 && self.lines[i] == self.lines[i - 1] {
            print!("   | ");
        } else {
            print!("{:>4} ", self.lines[i]);
        }

        use OpCode::*;
        match instruction {
            Constant(index) => self.disassemble_constant("LOAD_CONST", *index),
            Add => println!("ADD"),
            Subtract => println!("SUBTRACT"),
            Multiply => println!("MULTIPLY"),
            Divide => println!("DIVIDE"),
            Negate => println!("NEGATE"),
            Not => println!("NOT"),
            Return => println!("RETURN"),
            Equal => println!("EQUAL"),
            Greater => println!("GREATER"),
            Less => println!("LESS"),
            Print => println!("PRINT"),
            Pop => println!("POP"),
            Jump(index) => self.disassemble_large("JUMP", *index + 1),
            JumpIfFalse(index) => self.disassemble_large("JUMP_IF_FALSE", *index + 1),
            JumpBack(index) => self.disassemble_large("JUMP_BACK", *index - 1),
            DefineGlobal(index) => self.disassemble_constant("DEFINE_GLOBAL", *index),
            GetGlobal(index) => self.disassemble_constant("GET_GLOBAL", *index),
            SetGlobal(index) => self.disassemble_constant("SET_GLOBAL", *index),
            GetLocal(index) => self.disassemble_large("GET_LOCAL", *index),
            SetLocal(index) => self.disassemble_large("SET_LOCAL", *index),
            Call(index) => self.disassemble_large("CALL", *index),
        }
    }
}
