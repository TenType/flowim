use crate::{
    chunk::{type_as_str, OpCode, Value},
    objects::Function,
    result::LangError,
};
use std::collections::HashMap;

const FRAME_LIMIT: usize = 64;

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
            stack: vec![Value::Void],
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

    fn peek_more(&self, n: usize) -> Value {
        self.stack[self.stack.len() - n - 1].clone()
    }

    fn frame(&self) -> &CallFrame {
        self.frames.last().expect("No frames found")
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("No frames found")
    }

    fn is_falsy(&self, value: Value) -> bool {
        match value {
            Value::Bool(v) => !v,
            _ => false,
        }
    }

    fn read_constant(&self, index: usize) -> Value {
        self.frame().function.chunk.constants[index].clone()
    }

    fn read_string(&self, index: usize) -> String {
        if let Value::Str(s) = self.read_constant(index) {
            s
        } else {
            panic!("Constant is not a string");
        }
    }

    fn binary_op(&mut self, operation: OpCode) -> Result<(), LangError> {
        use LangError::RuntimeError;
        use OpCode::*;
        use Value::*;

        let mut operands = (self.pop(), self.pop());
        let bad_operation = |op: &str,
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

    fn call_value(&mut self, value: Value, arg_len: usize) -> Result<CallFrame, LangError> {
        match value {
            Value::Fun(function) => self.call(function, arg_len),
            _ => {
                self.runtime_error("Can only call functions and classes");
                Err(LangError::RuntimeError)
            }
        }
    }

    fn call(&mut self, function: Function, arg_len: usize) -> Result<CallFrame, LangError> {
        if arg_len != function.arity {
            self.runtime_error(&format!(
                "Expected {} arguments, but found {}",
                function.arity, arg_len
            ));
            Err(LangError::RuntimeError)
        } else if self.frames.len() >= FRAME_LIMIT {
            self.runtime_error("Call stack limit exceeded");
            Err(LangError::RuntimeError)
        } else {
            let mut frame = CallFrame::new(function);
            frame.index = self.stack.len() - arg_len - 1;
            Ok(frame)
        }
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
        self.frame()
            .function
            .chunk
            .disassemble_op(&op, self.frame().counter - 1);
    }

    pub fn run(&mut self, function: Function) -> Result<GlobalsType, LangError> {
        self.frames.push(CallFrame::new(function));

        #[cfg(debug_assertions)]
        println!("== VM Debug ==");

        loop {
            let op = self.frame().function.chunk.code[self.frame().counter];

            self.frame_mut().counter += 1;

            #[cfg(debug_assertions)]
            self.disassemble(op);

            use OpCode::*;
            match op {
                Constant(index) => {
                    let constant = self.read_constant(index);
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
                    let result = self.pop();
                    let frame = self.frames.pop();

                    if self.frames.is_empty() {
                        return Ok(self.globals.clone());
                    }

                    self.stack.truncate(frame.unwrap().index);
                    self.push(result);
                }

                Equal => self.binary_op(Equal)?,
                Greater => self.binary_op(Greater)?,
                Less => self.binary_op(Less)?,
                Print => println!("{}", self.pop()),

                Pop => {
                    self.pop();
                }

                Jump(index) => {
                    self.frame_mut().counter += index;
                }

                JumpIfFalse(index) => {
                    if self.is_falsy(self.peek()) {
                        self.frame_mut().counter += index;
                    }
                }

                JumpBack(index) => {
                    self.frame_mut().counter -= index;
                }

                DefineGlobal(index) => {
                    let name = self.read_string(index);
                    let value = self.pop();
                    self.globals.insert(name, value);
                }

                GetGlobal(index) => {
                    let name = self.read_string(index);
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
                    let name = self.read_string(index);
                    if self.globals.insert(name.clone(), self.peek()).is_none() {
                        self.globals.remove(&name);
                        self.runtime_error(&format!("`{}` is not defined", name));
                        return Err(LangError::RuntimeError);
                    }
                }

                GetLocal(index) => {
                    self.push(self.stack[index + self.frame().index].clone());
                }

                SetLocal(index) => {
                    let x = index + self.frame().index;
                    self.stack[x] = self.peek();
                }

                Call(index) => {
                    let frame = self.call_value(self.peek_more(index), index)?;
                    self.frames.push(frame);
                }
            }
        }
    }

    fn runtime_error(&self, msg: &str) {
        eprintln!("{}", msg);

        for frame in self.frames.iter().rev() {
            eprintln!(
                "    at {}:{}",
                frame.function.name,
                frame.function.chunk.lines.last().unwrap()
            );
        }
    }
}
