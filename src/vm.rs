use crate::{
    chunk::{type_as_str, Chunk, OpCode, Value},
    result::LangError,
};
use std::collections::HashMap;

pub type GlobalsType = HashMap<String, Value>;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: GlobalsType,
}

impl VM {
    pub fn new(chunk: Chunk, globals: GlobalsType) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
            globals,
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
        use LangError::RuntimeError;
        use OpCode::*;
        use Value::*;

        let mut operands = (self.pop(), self.pop());
        let mut bad_operation = |op: &str,
                                 expected: &str,
                                 actual: (Value, Value)|
         -> Result<(), LangError> {
            self.runtime_error(&format!(
                    "Cannot use the operator `{op}` with `{}` and `{}`; expected two arguments of `{expected}`.",
                    type_as_str(actual.0),
                    type_as_str(actual.1)
                ));
            Err(RuntimeError)
        };

        match operands {
            (Int(x), Float(_)) => operands.0 = Float(x as f64),
            (Float(_), Int(x)) => operands.1 = Float(x as f64),
            _ => (),
        }

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
                (Int(b), Int(a)) => {
                    if b == 0 {
                        self.runtime_error("Division by zero");
                        return Err(RuntimeError);
                    }
                    Int(a / b)
                }
                (Float(b), Float(a)) => {
                    if b == 0.0 {
                        self.runtime_error("Division by zero");
                        return Err(RuntimeError);
                    }
                    Float(a / b)
                }
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

    #[cfg(debug_assertions)]
    fn disassemble(&self, op: OpCode) {
        if !self.stack.is_empty() {
            print!("        |  ");
            for item in &self.stack {
                print!("[ {} ]", item);
            }
            println!();
        }
        self.chunk.disassemble_op(&op, self.ip - 1);
    }

    pub fn run(&mut self) -> Result<GlobalsType, LangError> {
        loop {
            let op = self.next();

            #[cfg(debug_assertions)]
            self.disassemble(op);

            use OpCode::*;
            match op {
                Constant(index) => {
                    let constant = self.chunk.constants[index].clone();
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
                Return => return Ok(self.globals.clone()),
                Equal => self.binary_op(Equal)?,
                Greater => self.binary_op(Greater)?,
                Less => self.binary_op(Less)?,
                Print => println!("{}", self.pop()),

                Pop => {
                    self.pop();
                }

                Jump(index) => {
                    self.ip += index;
                }

                JumpIfFalse(index) => {
                    if self.is_falsy(self.peek()) {
                        self.ip += index;
                    }
                }

                JumpBack(index) => {
                    self.ip -= index;
                }

                DefineGlobal(index) => {
                    let name = self.chunk.read_string(index);
                    let value = self.pop();
                    self.globals.insert(name, value);
                }

                GetGlobal(index) => {
                    let name = self.chunk.read_string(index);
                    match self.globals.get(&name) {
                        Some(value) => {
                            let v = value.clone();
                            self.push(v);
                        }
                        None => {
                            self.runtime_error(&format!("`{}` is not defined", name));
                            return Err(LangError::RuntimeError);
                        }
                    }
                }

                SetGlobal(index) => {
                    let name = self.chunk.read_string(index);
                    if self.globals.insert(name.clone(), self.peek()).is_none() {
                        self.globals.remove(&name);
                        self.runtime_error(&format!("`{}` is not defined", name));
                        return Err(LangError::RuntimeError);
                    }
                }

                GetLocal(index) => {
                    self.push(self.stack[index].clone());
                }

                SetLocal(index) => {
                    self.stack[index] = self.peek();
                }
            }
        }
    }

    fn runtime_error(&mut self, msg: &str) {
        eprintln!("{}", msg);
        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {}] in script", line);
    }
}
