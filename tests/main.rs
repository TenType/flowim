#![cfg(test)]

use regex::Regex;
use std::{env, fs, process::Command};
use test_generator::test_resources;

#[derive(Debug)]
struct Expected<'a> {
    output: Vec<&'a str>,
    compile_error: Vec<&'a str>,
    runtime_error: &'a str,
}

fn parse_comments(contents: &str) -> Expected {
    let output_regex = Regex::new(r"//> (.*)").unwrap();
    let compile_error_regex = Regex::new(r"//! (.*)").unwrap();
    let runtime_error_regex = Regex::new(r"//!! (.*)").unwrap();

    let mut expected = Expected {
        output: Vec::new(),
        compile_error: Vec::new(),
        runtime_error: "",
    };

    for line in contents.lines() {
        if let Some(matched) = output_regex.captures(line) {
            expected.output.push(matched.get(1).unwrap().as_str());
        }
        if let Some(matched) = compile_error_regex.captures(line) {
            expected
                .compile_error
                .push(matched.get(1).unwrap().as_str());
        }
        if let Some(matched) = runtime_error_regex.captures(line) {
            expected.runtime_error = matched.get(1).unwrap().as_str();
        }
    }

    expected
}

#[test_resources("tests/**/*.flwm")]
fn run(resource: &str) {
    let contents = fs::read_to_string(resource).expect("Could not read test file");

    let expected = parse_comments(&contents);

    let path = env!("CARGO_BIN_EXE_flowim");
    let mut path = Command::new(&path);

    let result = path.arg(resource).output().unwrap();

    let out = String::from_utf8(result.stdout).unwrap();
    let out: Vec<&str> = out.lines().collect();

    let err = String::from_utf8(result.stderr).unwrap();
    let _err: Vec<&str> = err.lines().collect();

    assert_eq!(out, expected.output);
    // TODO: Add support for error tests once error messages are finalized
}
