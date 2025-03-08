use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn test_prompt_help() {
    // Test help text for the prompt command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("prompt").arg("--help").assert().success();
    output.stdout(predicate::str::contains("--goals"));
    output.stdout(predicate::str::contains("--format"));
    output.stdout(predicate::str::contains("--warnings"));
    output.stdout(predicate::str::contains("--clipboard"));
    output.stdout(predicate::str::contains("--quiet"));
    output.stdout(predicate::str::contains("--pipe"));
    output.stdout(predicate::str::contains("--no-thinking"));
    output.stdout(predicate::str::contains("--recursion"));
}

#[test]
fn test_prompt_quiet_flag() {
    // When using --quiet flag, there should be no output to stderr
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Mock the input that would normally be requested interactively
    let assert = cmd
        .arg("prompt")
        .arg("--quiet")
        .arg("--goals")
        .arg("test goals")
        .write_stdin("test\ntext\n\n") // Mock the interactive input
        .assert()
        .success();
    
    // With --quiet, no welcome message should be printed
    assert.stderr(predicate::str::is_empty());
}

#[test]
fn test_prompt_clipboard_flag() {
    // Testing --clipboard flag requires platform-specific tests
    // This is a basic test that just ensures the flag doesn't cause errors
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("prompt")
        .arg("--goals")
        .arg("test goals")
        .arg("--clipboard")
        .write_stdin("test\ntext\n\n")
        .assert()
        .success();
    
    // We can't easily verify clipboard contents in a cross-platform test
    // So we just ensure the command executes successfully
}

#[test]
fn test_prompt_pipe_flag() {
    // Test the --pipe flag by providing input via stdin
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("prompt")
        .arg("--pipe")
        .arg("--goals")
        .arg("Test piped input")
        .write_stdin("This is piped content\n")
        .assert()
        .success();
        
    // The piped content should be used as context
    // We'd need to mock the API responses to fully verify this
}

#[test]
fn test_prompt_recursion_flag() {
    // Test the basic recursion flag without actually running multiple recursions
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Set recursion to 1 (which should be the same as no recursion)
    cmd.arg("prompt")
        .arg("--goals")
        .arg("test recursion")
        .arg("--recursion")
        .arg("1")
        .write_stdin("test\ntext\n\n")
        .assert()
        .success();
        
    // The real recursion tests would need to mock the environment variables
    // and verify that multiple processes are launched
}

#[test]
fn test_prompt_format_flag() {
    // Test specifying a custom format
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("prompt")
        .arg("--goals")
        .arg("test format")
        .arg("--format")
        .arg("json")
        .write_stdin("test\n")
        .assert()
        .success();
    
    // We'd need to mock the API call to verify the format is passed correctly
}

#[test]
fn test_prompt_warnings_flag() {
    // Test specifying warnings
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("prompt")
        .arg("--goals")
        .arg("test warnings")
        .arg("--warnings")
        .arg("Be careful with this")
        .write_stdin("test\n")
        .assert()
        .success();
    
    // We'd need to mock the API call to verify the warnings are passed correctly
}

#[test]
fn test_prompt_no_thinking_flag() {
    // Test the --no-thinking flag
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("prompt")
        .arg("--goals")
        .arg("test no thinking")
        .arg("--no-thinking")
        .write_stdin("test\ntext\n\n")
        .assert()
        .success();
    
    // We'd need to mock the API responses to verify this works correctly
}