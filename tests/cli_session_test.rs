use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_session_help() {
    // Test help text for the session command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("session").arg("--help").assert().success();
    
    output.stdout(predicate::str::contains("--goals"));
    output.stdout(predicate::str::contains("--return_format"));
    output.stdout(predicate::str::contains("--warnings"));
    output.stdout(predicate::str::contains("--quiet"));
    output.stdout(predicate::str::contains("--pipe"));
}

#[test]
fn test_session_basic() {
    // Test running a basic session with goals and format
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let assert = cmd.arg("session")
        .arg("--goals")
        .arg("Test session")
        .arg("--return_format")
        .arg("text")
        .assert()
        .success();
    
    // Verify the informational output is shown (stderr)
    assert.stderr(predicate::str::contains("Running session with the following parameters"));
    assert.stderr(predicate::str::contains("Goals: Test session"));
    assert.stderr(predicate::str::contains("Return Format: text"));
    
    // Verify the actual output (stdout)
    assert.stdout(predicate::str::contains("Processed session for goals: Test session"));
}

#[test]
fn test_session_with_warnings() {
    // Test running a session with warnings
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let assert = cmd.arg("session")
        .arg("--goals")
        .arg("Test session with warnings")
        .arg("--return_format")
        .arg("json")
        .arg("--warnings")
        .arg("Be careful with this")
        .assert()
        .success();
    
    // Verify warnings are shown in stderr
    assert.stderr(predicate::str::contains("Warnings: Be careful with this"));
}

#[test]
fn test_session_quiet_mode() {
    // Test quiet mode (no informational output)
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let assert = cmd.arg("session")
        .arg("--goals")
        .arg("Test quiet mode")
        .arg("--return_format")
        .arg("text")
        .arg("--quiet")
        .assert()
        .success();
    
    // With --quiet, no informational output should be shown
    assert.stderr(predicate::str::is_empty());
    
    // The main output should still be present
    assert.stdout(predicate::str::contains("Processed session for goals: Test quiet mode"));
}

#[test]
fn test_session_pipe() {
    // Test piping input to the session
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let assert = cmd.arg("session")
        .arg("--goals")
        .arg("Test piped input")
        .arg("--return_format")
        .arg("text")
        .arg("--pipe")
        .write_stdin("This is piped content\n")
        .assert()
        .success();
    
    // Verify that the piped content is mentioned in the output
    assert.stdout(predicate::str::contains("with input: This is piped content"));
    
    // The stderr should mention the piped content
    assert.stderr(predicate::str::contains("Context from stdin"));
}

#[test]
fn test_session_logs() {
    // Test that session outputs are logged
    let temp_dir = tempdir().unwrap();
    let old_dir = std::env::current_dir().unwrap();
    
    // Change to temp dir for log file creation
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    cmd.arg("session")
        .arg("--goals")
        .arg("Test logging")
        .arg("--return_format")
        .arg("text")
        .assert()
        .success();
    
    // Check that the log file was created
    let log_file = temp_dir.path().join("sessions.jsonl");
    assert!(log_file.exists());
    
    // Read the log file contents
    let log_content = fs::read_to_string(log_file).unwrap();
    assert!(log_content.contains("\"goals\":\"Test logging\""));
    
    // Restore original directory
    std::env::set_current_dir(old_dir).unwrap();
}