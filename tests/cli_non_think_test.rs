use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
#[ignore]
fn test_non_think_help() {
    // Test help text for the non-think command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("non-think").arg("--help").output().expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--prompt"));
    assert!(stdout.contains("--clipboard"));
    assert!(stdout.contains("--quiet"));
    assert!(stdout.contains("--pipe"));
    assert!(stdout.contains("--filter_thinking"));
}

#[test]
#[ignore]
fn test_non_think_quiet_flag() {
    // When using --quiet flag, there should be no output to stderr
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Mock the input that would normally be requested interactively
    let output = cmd
        .arg("non-think")
        .arg("--quiet")
        .arg("--prompt")
        .arg("test prompt")
        .output()
        .expect("Failed to execute command");
    
    // With --quiet, no informational messages should be printed
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty());
}

#[test]
#[ignore]
fn test_non_think_clipboard_flag() {
    // Testing --clipboard flag requires platform-specific tests
    // This is a basic test that just ensures the flag doesn't cause errors
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("non-think")
        .arg("--prompt")
        .arg("test prompt")
        .arg("--clipboard")
        .output()
        .expect("Failed to execute command");
    
    // We can't easily verify clipboard contents in a cross-platform test
    // So we just ensure the command executes successfully
    assert!(output.status.success());
}

#[test]
#[ignore]
fn test_non_think_pipe_flag() {
    // Test the --pipe flag by providing input via stdin
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("non-think")
        .arg("--pipe")
        .arg("--prompt")
        .arg("Test piped input")
        .write_stdin("This is piped content\n")
        .output()
        .expect("Failed to execute command");
        
    // The piped content should be used as context
    // We'd need to mock the API responses to fully verify this
    assert!(output.status.success());
}

#[test]
#[ignore]
fn test_non_think_filter_thinking_flag() {
    // Test the --filter_thinking flag
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("non-think")
        .arg("--prompt")
        .arg("test filter thinking")
        .arg("--filter_thinking")
        .output()
        .expect("Failed to execute command");
    
    // We'd need to mock the API responses to verify this works correctly
    assert!(output.status.success());
}

#[test]
#[ignore]
fn test_non_think_with_prompt_arg() {
    // Test specifying a prompt directly
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("non-think")
        .arg("--prompt")
        .arg("This is a direct prompt")
        .output()
        .expect("Failed to execute command");
    
    // We'd need to mock the API call to verify the prompt is passed correctly
    assert!(output.status.success());
}

#[test]
#[ignore]
fn test_non_think_interactive_prompt() {
    // Test the interactive prompt flow
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("non-think")
        .write_stdin("Interactive prompt text\n")
        .output()
        .expect("Failed to execute command");
    
    // We'd need to mock the interactive input and API responses to fully test this
    assert!(output.status.success());
}