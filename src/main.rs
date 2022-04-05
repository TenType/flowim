mod chunk;
use chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(constant, 123);
    chunk.write(OpCode::Return, 123);

    chunk.disassemble("Test chunk");
}
