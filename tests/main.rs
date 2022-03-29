use std::fs;
use test_generator::test_resources;

#[test_resources("tests/*/*.flwm")]
fn run(resource: &str) {
    let contents = fs::read_to_string(resource).expect("Could not read test file");

    assert!(!contents.is_empty());
}
