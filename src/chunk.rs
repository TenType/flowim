#[derive(Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(isize),
    Float(f64),
    Str(String),
}

use std::fmt::{Display, Formatter, Result};
impl Display for Value {
    fn fmt(&self, format: &mut Formatter<'_>) -> Result {
        use Value::*;
        match self {
            Bool(value) => write!(format, "{}", value),
            Int(value) => write!(format, "{}", value),
            Float(value) => write!(format, "{}", value),
            Str(value) => write!(format, "{}", value),
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
    }
}

#[derive(Copy, Clone, Debug)]
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
}

pub struct Chunk {
    length: u8,
    pub lines: Vec<usize>,
    pub constants: Vec<Value>,
    pub code: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            length: 0,
            lines: Vec::new(),
            constants: Vec::new(),
            code: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
        self.length += 1;
    }

    pub fn add_constant(&mut self, value: Value) -> OpCode {
        self.constants.push(value);
        OpCode::Constant(self.constants.len() - 1)
    }

    #[cfg(debug_assertions)]
    pub fn _disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for (i, instruction) in self.code.iter().enumerate() {
            self.disassemble_op(instruction, i);
        }
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
            Constant(value) => println!(
                "{:<16} {:>4} ({})",
                "LOAD_CONST",
                value,
                self.constants[*value as usize] // print_value
            ),
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
        }
    }
}
