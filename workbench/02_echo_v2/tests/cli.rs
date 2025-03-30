use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn dies_on_no_args() -> TestResult {
    let mut cmd = Command::cargo_bin("echor").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
    Ok(())
}

/// Run the echor command with the given arguments and compare the output to the expected file.
///
/// # Arguments
/// * `args` - The arguments to pass to the echor command.
/// * `expected_file` - The path to the file containing the expected output.
///
/// # Returns
/// A `Result` indicating success or failure.
///
/// # Example
/// ```
/// run(&vec!["Hello there"], "tests/expected/hello1.txt")
/// ```
fn run(args: &[&str], expected_file: &str) -> TestResult {
    let mut cmd = Command::cargo_bin("echor")?;
    let expected = fs::read_to_string(expected_file)?;
    cmd.args(args).assert().success().stdout(expected);
    Ok(())
}

#[test]
fn hello1() -> TestResult {
    run(&vec!["Hello there"], "tests/expected/hello1.txt")
}

#[test]
fn hello2() -> TestResult {
    run(&vec!["Hello", "there"], "tests/expected/hello2.txt")
}

#[test]
fn hello1_n() -> TestResult {
    run(&vec!["-n", "Hello  there"], "tests/expected/hello1.n.txt")
}

#[test]
fn hello2_n() -> TestResult {
    run(&vec!["-n", "Hello", "there"], "tests/expected/hello2.n.txt")
}
