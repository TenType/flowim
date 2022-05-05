mod chunk;
mod compiler;
mod lexer;
mod result;
mod token;
mod vm;

use result::LangError::{self, *};
use std::{
    collections::HashMap,
    env, fs,
    io::{self, Write},
    process,
};
use vm::{GlobalsType, VM};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => process::exit(64),
    }
}

fn check_result<T>(result: Result<T, LangError>) -> T {
    match result {
        Err(CompileError) => process::exit(65),
        Err(RuntimeError) => process::exit(70),
        Ok(output) => output,
    }
}

fn run_code(code: &str, globals: GlobalsType) -> Result<GlobalsType, LangError> {
    let tokens = compiler::compile(code);
    match tokens {
        Ok(chunk) => VM::new(chunk, globals).run(),
        Err(error) => Err(error),
    }
}

fn run_file(path: &str) {
    let code = fs::read_to_string(path).expect("Could not read test file");
    let result = run_code(&code, HashMap::new());
    check_result(result);
}

fn repl() {
    let mut globals = HashMap::new();
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Could not read line from REPL");
        line.pop(); // pop newline \n
        if line.is_empty() {
            continue;
        }
        if let Ok(new_globals) = run_code(&line, globals.clone()) {
            globals = new_globals;
        }
    }
}
