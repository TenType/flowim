use crate::chunk::Chunk;
use std::fmt::{self, Display};

pub enum FunctionType {
    Function,
    Script,
}

#[derive(Clone)]
pub struct Function {
    arity: usize,
    pub chunk: Chunk,
    name: String,
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
