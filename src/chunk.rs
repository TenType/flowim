#![allow(dead_code)]

pub type Value = f64;

#[derive(Copy, Clone)]
pub enum OpCode {
    Constant(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

pub struct Chunk {
    length: u8,
    lines: Vec<u8>,
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

    pub fn write(&mut self, byte: OpCode, line: u8) {
        self.code.push(byte);
        self.lines.push(line);
        self.length += 1;
    }

    pub fn add_constant(&mut self, value: Value) -> OpCode {
        self.constants.push(value);
        OpCode::Constant(self.constants.len() - 1)
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for (i, instruction) in self.code.iter().enumerate() {
            self.disassemble_op(instruction, i);
        }
    }

    pub fn disassemble_op(&self, instruction: &OpCode, i: usize) {
        print!("{:04} ", i);
        if i > 0 && self.lines[i] == self.lines[i - 1] {
            print!("   | ");
        } else {
            print!("{:>4} ", self.lines[i]);
        }

        match instruction {
            OpCode::Constant(value) => println!(
                "{:<16} {:>4} {}",
                "OP_CONSTANT",
                value,
                self.constants[*value as usize] // print_value
            ),
            OpCode::Add => println!("OP_ADD"),
            OpCode::Subtract => println!("OP_SUBTRACT"),
            OpCode::Multiply => println!("OP_MULTIPLY"),
            OpCode::Divide => println!("OP_DIVIDE"),
            OpCode::Negate => println!("OP_NEGATE"),
            OpCode::Return => println!("OP_RETURN"),
        }
    }
}
