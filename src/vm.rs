use crate::{
    chunk::{type_as_str, OpCode, Value},
    objects::Function,
    result::LangError,
};
use std::collections::HashMap;

pub type GlobalsType = HashMap<String, Value>;

#[derive(Clone)]
struct CallFrame {
    function: Function,
    counter: usize,
    index: usize,
}

impl CallFrame {
    fn new(function: Function) -> Self {
        CallFrame {
            function,
            counter: 0,
            index: 0,
        }
    }
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: GlobalsType,
}

impl VM {
    pub fn new(globals: GlobalsType) -> Self {
        Self {
            frames: Vec::new(),
            stack: Vec::new(),
            globals,
        }
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

    fn read_constant(&self, frame: &CallFrame, index: usize) -> Value {
        frame.function.chunk.constants[index].clone()
    }

    fn read_string(&self, frame: &CallFrame, index: usize) -> String {
        if let Value::Str(s) = self.read_constant(frame, index) {
            s.clone()
        } else {
            panic!("Constant is not a string");
        }
    }

    fn binary_op(&mut self, operation: OpCode) -> Result<(), LangError> {
        use LangError::RuntimeError;
        use OpCode::*;
        use Value::*;

        let frame = self.frames.last().cloned().expect("No frames found");

        let mut operands = (self.pop(), self.pop());
        let mut bad_operation = |op: &str,
                                 expected: &str,
                                 actual: (Value, Value)|
         -> Result<(), LangError> {
            self.runtime_error(&frame, &format!(
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
                        self.runtime_error(&frame, "Division by zero");
                        return Err(RuntimeError);
                    }
                    Int(a / b)
                }
                (Float(b), Float(a)) => {
                    if b == 0.0 {
                        self.runtime_error(&frame, "Division by zero");
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
    fn disassemble(&self, frame: &CallFrame, op: OpCode) {
        if !self.stack.is_empty() {
            print!("        |  ");
            for item in &self.stack {
                print!("[ {} ]", item);
            }
            println!();
        }
        frame.function.chunk.disassemble_op(&op, frame.counter - 1);
    }

    pub fn run(&mut self, function: Function) -> Result<GlobalsType, LangError> {
        let mut frame = CallFrame::new(function);
        self.frames.push(frame.clone());

        loop {
            let op = frame.clone().function.chunk.code[frame.counter];

            frame.counter += 1;

            #[cfg(debug_assertions)]
            self.disassemble(&frame, op);

            use OpCode::*;
            match op {
                Constant(index) => {
                    let constant = self.read_constant(&frame, index);
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
                        self.runtime_error(
                            &frame,
                            &format!("Operand of {} must be an `int` or `float`", value),
                        );
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
                    frame.counter += index;
                }

                JumpIfFalse(index) => {
                    if self.is_falsy(self.peek()) {
                        frame.counter += index;
                    }
                }

                JumpBack(index) => {
                    frame.counter -= index;
                }

                DefineGlobal(index) => {
                    let name = self.read_string(&frame, index);
                    let value = self.pop();
                    self.globals.insert(name, value);
                }

                GetGlobal(index) => {
                    let name = self.read_string(&frame, index);
                    match self.globals.get(&name) {
                        Some(value) => {
                            let v = value.clone();
                            self.push(v);
                        }
                        None => {
                            self.runtime_error(&frame, &format!("`{}` is not defined", name));
                            return Err(LangError::RuntimeError);
                        }
                    }
                }

                SetGlobal(index) => {
                    let name = self.read_string(&frame, index);
                    if self.globals.insert(name.clone(), self.peek()).is_none() {
                        self.globals.remove(&name);
                        self.runtime_error(&frame, &format!("`{}` is not defined", name));
                        return Err(LangError::RuntimeError);
                    }
                }

                GetLocal(index) => {
                    self.push(self.stack[index + frame.index].clone());
                }

                SetLocal(index) => {
                    self.stack[index + frame.index] = self.peek();
                }
            }
        }
    }

    fn runtime_error(&mut self, frame: &CallFrame, msg: &str) {
        eprintln!("{}", msg);
        let line = frame.function.chunk.lines[frame.counter - 1];
        eprintln!("[line {}] in script", line);
    }
}
