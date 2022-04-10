mod chunk;
mod lexer;
mod result;
mod token;
mod vm;

use result::LangError::*;
use std::{
    env, fs,
    io::{self, Write},
    process,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => process::exit(64),
    }
    repl();
}

fn run_code(code: String) {
    // let result = VM::new(chunk).run();
    let result = lexer::lex(code);
    match result {
        Err(CompileError) => process::exit(65),
        Err(RuntimeError) => process::exit(70),
        Ok(_) => (),
    }
}

fn run_file(path: &str) {
    let code = fs::read_to_string(path).expect("Could not read test file");
    run_code(code);
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
        run_code(line);
    }
}
