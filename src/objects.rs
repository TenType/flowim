use crate::chunk::Chunk;
use std::fmt::{self, Display};

#[derive(PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

#[derive(Clone, PartialEq)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
}

impl Function {
    pub fn new() -> Self {
        Function {
            arity: 0,
            chunk: Chunk::new(),
            name: String::from("<script>"),
        }
    }
}

impl Display for Function {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(format, "<fun {}>", self.name)
    }
}
