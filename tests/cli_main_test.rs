use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_version() {
    // Test that the version flag works
    let mut cmd = Command::cargo_bin("ola").unwrap();
    cmd.arg("--version").assert().success().stdout(predicate::str::contains("0.2.0"));
}

#[test]
fn test_cli_help() {
    // Test the main help output
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let assert = cmd.arg("--help").assert().success();
    
    // Check that all subcommands are listed
    assert.stdout(predicate::str::contains("start"));
    assert.stdout(predicate::str::contains("prompt"));
    assert.stdout(predicate::str::contains("configure"));
    assert.stdout(predicate::str::contains("models"));
    assert.stdout(predicate::str::contains("session"));
    assert.stdout(predicate::str::contains("non-think"));
    assert.stdout(predicate::str::contains("settings"));
}

#[test]
fn test_no_args() {
    // Test running without arguments shows help message
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let assert = cmd.assert().success();
    
    // Should show message about no subcommand used
    assert.stderr(predicate::str::contains("No subcommand was used"));
}

#[test]
fn test_invalid_subcommand() {
    // Test an invalid subcommand shows an error
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let assert = cmd.arg("invalid-command").assert().failure();
    
    // Should show error about invalid subcommand
    assert.stderr(predicate::str::contains("error: unrecognized subcommand"));
}

#[test]
fn test_start_command() {
    // Test the basic start command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let assert = cmd.arg("start").assert().success();
    
    // Should show starting message
    assert.stdout(predicate::str::contains("Starting the application"));
}

#[test]
fn test_start_verbose() {
    // Test the start command with verbose flag
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let assert = cmd.arg("start").arg("--verbose").assert().success();
    
    // Should show verbose mode message
    assert.stdout(predicate::str::contains("Running in verbose mode"));
}