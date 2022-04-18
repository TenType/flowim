use crate::{
    chunk::{type_as_str, Chunk, OpCode, Value},
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
        self.stack.last().expect("Empty stack").clone()
    }

    fn is_falsy(&self, value: Value) -> bool {
        match value {
            Value::Bool(v) => !v,
            _ => false,
        }
    }

    fn binary_op(&mut self, operation: OpCode) -> Result<(), LangError> {
        use OpCode::*;
        use Value::*;
        let operands = (self.pop(), self.pop());
        let mut bad_operation = |op: &str,
                                 expected: &str,
                                 actual: (Value, Value)|
         -> Result<(), LangError> {
            self.runtime_error(&format!(
                    "Operator `{op}` expected two arguments of `{expected}` (of the same type), but found `{}` and `{}`.",
                    type_as_str(actual.0),
                    type_as_str(actual.1)
                ));
            Err(LangError::RuntimeError)
        };
        let result = match operation {
            Add => match operands {
                (Int(b), Int(a)) => Int(a + b),
                (Float(b), Float(a)) => Float(a + b),
                (Str(b), Str(a)) => Str(a + &b),
                _ => return bad_operation("+", "int or float or str", operands),
            },
            Subtract => match operands {
                (Int(b), Int(a)) => Int(a - b),
                (Float(b), Float(a)) => Float(a - b),
                _ => return bad_operation("-", "int or float", operands),
            },
            Multiply => match operands {
                (Int(b), Int(a)) => Int(a * b),
                (Float(b), Float(a)) => Float(a * b),
                _ => return bad_operation("*", "int or float", operands),
            },
            Divide => match operands {
                (Int(b), Int(a)) => Int(a / b),
                (Float(b), Float(a)) => Float(a / b),
                _ => return bad_operation("/", "int or float", operands),
            },
            Equal => Bool(operands.0 == operands.1),
            Greater => match operands {
                (Int(b), Int(a)) => Bool(a > b),
                (Float(b), Float(a)) => Bool(a > b),
                (Str(b), Str(a)) => Bool(a > b),
                _ => return bad_operation(">", "int or float or str", operands),
            },
            Less => match operands {
                (Int(b), Int(a)) => Bool(a < b),
                (Float(b), Float(a)) => Bool(a < b),
                (Str(b), Str(a)) => Bool(a < b),
                _ => return bad_operation("<", "int or float or str", operands),
            },
            _ => panic!("Unsupported binary operation: {:?}", operation),
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
                    let constant = self.chunk.constants[value].clone();
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
                Equal => self.binary_op(Equal)?,
                Greater => self.binary_op(Greater)?,
                Less => self.binary_op(Less)?,
            }
        }
    }

    fn runtime_error(&mut self, msg: &str) {
        eprintln!("{}", msg);
        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {}] in script", line);
    }
}
