type Value = f64;

pub enum OpCode {
    Constant(usize),
    Return,
}

pub struct Chunk {
    length: u8,
    lines: Vec<u8>,
    constants: Vec<Value>,
    code: Vec<OpCode>,
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
            print!("{:04} ", i);
            if i > 0 && self.lines[i] == self.lines[i - 1] {
                print!("   | ");
            } else {
                print!("{:>4} ", self.lines[i]);
            }

            match instruction {
                OpCode::Constant(value) => println!(
                    "{:<16} {:>4} {}",
                    "OP_CONSTANT", value, self.constants[*value as usize]
                ),
                OpCode::Return => println!("OP_RETURN"),
            }
        }
    }
}
