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

    fn peek(&self) -> Value {
        *self.stack.last().expect("Empty stack")
    }

    fn is_falsy(&self, value: Value) -> bool {
        match value {
            Value::Bool(v) => !v,
            _ => false,
        }
    }

    fn binary_op(&mut self, operation: OpCode) -> Result<(), LangError> {
        use OpCode::{Add, Divide, Multiply, Subtract};
        use Value::*;
        let operands = (self.pop(), self.pop());
        let result = match operands {
            (Int(a), Int(b)) => Int(match operation {
                Add => a + b,
                Subtract => a - b,
                Multiply => a * b,
                Divide => a / b,
                _ => panic!("Unsupported binary operation: {:?}", operation),
            }),
            (Float(a), Float(b)) => Float(match operation {
                Add => a + b,
                Subtract => a - b,
                Multiply => a * b,
                Divide => a / b,
                _ => panic!("Unsupported binary operation: {:?}", operation),
            }),
            _ => {
                self.runtime_error("Operands must be both `int` or `float`");
                return Err(LangError::RuntimeError);
            }
        };

        self.push(result);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), LangError> {
        loop {
            let op = self.next();

            if cfg!(debug_assertions) {
                if !self.stack.is_empty() {
                    print!("          ");
                    for item in &self.stack {
                        print!("[ {} ]", item);
                    }
                    println!();
                }
                self.chunk.disassemble_op(&op, self.ip - 1);
            }

            use OpCode::*;
            match op {
                Constant(value) => {
                    let constant = self.chunk.constants[value];
                    self.push(constant);
                }

                Add => self.binary_op(Add)?,
                Subtract => self.binary_op(Subtract)?,
                Multiply => self.binary_op(Multiply)?,
                Divide => self.binary_op(Divide)?,

                Negate => match self.peek() {
                    Value::Int(value) => {
                        self.pop();
                        self.push(Value::Int(-value));
                    }
                    Value::Float(value) => {
                        self.pop();
                        self.push(Value::Float(-value));
                    }
                    value => {
                        self.runtime_error(&format!(
                            "Operand of {} must be an `int` or `float`",
                            value
                        ));
                        return Err(LangError::RuntimeError);
                    }
                },
                Not => {
                    let v = self.pop();
                    self.push(Value::Bool(self.is_falsy(v)));
                }
                Return => {
                    println!("{}", self.pop());
                    return Ok(());
                }
                True => self.push(Value::Bool(true)),
                False => self.push(Value::Bool(false)),
            }
        }
    }

    fn runtime_error(&mut self, msg: &str) {
        eprintln!("{}", msg);
        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {}] in script", line);
    }
}
