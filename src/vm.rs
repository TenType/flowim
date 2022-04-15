use crate::{
    chunk::{Chunk, OpCode, Value},
    result::LangError,
};

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    fn next(&mut self) -> OpCode {
        let instruction = self.chunk.code[self.ip];
        self.ip += 1;
        instruction
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Empty stack")
    }

    fn binary_op(&mut self, operation: fn(Value, Value) -> Value) {
        let b = self.pop();
        let a = self.pop();
        self.push(operation(a, b))
    }

    pub fn run(&mut self) -> Result<(), LangError> {
        loop {
            let instruction = self.next();

            if cfg!(debug_assertions) {
                print!("          ");
                for item in &self.stack {
                    print!("[ {} ]", item);
                }
                println!();
                self.chunk.disassemble_op(&instruction, self.ip - 1);
            }

            match instruction {
                OpCode::Constant(value) => {
                    let constant = self.chunk.constants[value];
                    self.push(constant);
                }

                OpCode::Add => self.binary_op(|a, b| a + b),
                OpCode::Subtract => self.binary_op(|a, b| a - b),
                OpCode::Multiply => self.binary_op(|a, b| a * b),
                OpCode::Divide => self.binary_op(|a, b| a / b),

                OpCode::Negate => {
                    let value = -self.pop();
                    self.push(value);
                }
                OpCode::Return => {
                    println!("{}", self.pop());
                    return Ok(());
                }
            }
        }
    }
}
