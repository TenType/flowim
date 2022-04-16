mod chunk;
mod compiler;
mod lexer;
mod result;
mod token;
mod vm;

use result::LangError::{self, *};
use std::{
    env, fs,
    io::{self, Write},
    process,
};
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => process::exit(64),
    }
    repl();
}

fn _check_result<T>(result: Result<T, LangError>) -> T {
    match result {
        Err(CompileError) => process::exit(65),
        Err(RuntimeError) => process::exit(70),
        Ok(output) => output,
    }
}

fn run_code(code: &str) {
    let tokens = compiler::compile(code);
    if let Ok(chunk) = tokens {
        let _ = VM::new(chunk).run();
    }
}

fn run_file(path: &str) {
    let code = fs::read_to_string(path).expect("Could not read test file");
    run_code(&code);
}

fn repl() {
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Could not read line from REPL");
        if line.is_empty() {
            continue;
        }
        run_code(&line);
    }
}
