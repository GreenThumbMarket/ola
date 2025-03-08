use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_non_think_help() {
    // Test help text for the non-think command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("non-think").arg("--help").assert().success();
    output.stdout(predicate::str::contains("--prompt"));
    output.stdout(predicate::str::contains("--clipboard"));
    output.stdout(predicate::str::contains("--quiet"));
    output.stdout(predicate::str::contains("--pipe"));
    output.stdout(predicate::str::contains("--filter_thinking"));
}

#[test]
fn test_non_think_quiet_flag() {
    // When using --quiet flag, there should be no output to stderr
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Mock the input that would normally be requested interactively
    let assert = cmd
        .arg("non-think")
        .arg("--quiet")
        .arg("--prompt")
        .arg("test prompt")
        .assert()
        .success();
    
    // With --quiet, no informational messages should be printed
    assert.stderr(predicate::str::is_empty());
}

#[test]
fn test_non_think_clipboard_flag() {
    // Testing --clipboard flag requires platform-specific tests
    // This is a basic test that just ensures the flag doesn't cause errors
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("non-think")
        .arg("--prompt")
        .arg("test prompt")
        .arg("--clipboard")
        .assert()
        .success();
    
    // We can't easily verify clipboard contents in a cross-platform test
    // So we just ensure the command executes successfully
}

#[test]
fn test_non_think_pipe_flag() {
    // Test the --pipe flag by providing input via stdin
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("non-think")
        .arg("--pipe")
        .arg("--prompt")
        .arg("Test piped input")
        .write_stdin("This is piped content\n")
        .assert()
        .success();
        
    // The piped content should be used as context
    // We'd need to mock the API responses to fully verify this
}

#[test]
fn test_non_think_filter_thinking_flag() {
    // Test the --filter_thinking flag
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("non-think")
        .arg("--prompt")
        .arg("test filter thinking")
        .arg("--filter_thinking")
        .assert()
        .success();
    
    // We'd need to mock the API responses to verify this works correctly
}

#[test]
fn test_non_think_with_prompt_arg() {
    // Test specifying a prompt directly
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("non-think")
        .arg("--prompt")
        .arg("This is a direct prompt")
        .assert()
        .success();
    
    // We'd need to mock the API call to verify the prompt is passed correctly
}

#[test]
fn test_non_think_interactive_prompt() {
    // Test the interactive prompt flow
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("non-think")
        .write_stdin("Interactive prompt text\n")
        .assert()
        .success();
    
    // We'd need to mock the interactive input and API responses to fully test this
}