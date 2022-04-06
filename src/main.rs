mod chunk;
mod vm;

use chunk::{Chunk, OpCode};
use vm::VM;

fn main() {
    let mut chunk = Chunk::new();

    // 5.0
    let constant = chunk.add_constant(5.0);
    chunk.write(constant, 100);

    // + 3.0 = 8.0
    let constant = chunk.add_constant(3.0);
    chunk.write(constant, 100);
    chunk.write(OpCode::Add, 100);

    // * 2.0 = 16.0
    let constant = chunk.add_constant(2.0);
    chunk.write(constant, 100);
    chunk.write(OpCode::Multiply, 100);

    // / 4.0 = 4.0
    let constant = chunk.add_constant(4.0);
    chunk.write(constant, 100);
    chunk.write(OpCode::Divide, 100);

    // - 1.2 = 2.8
    let constant = chunk.add_constant(1.2);
    chunk.write(constant, 100);
    chunk.write(OpCode::Subtract, 100);

    // -2.8
    chunk.write(OpCode::Negate, 100);

    chunk.write(OpCode::Return, 100);

    chunk.disassemble("Test chunk");

    VM::new(chunk).run();
}
