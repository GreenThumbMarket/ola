use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
#[ignore]
fn test_session_help() {
    // Test help text for the session command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("session").arg("--help").output().expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--goals"));
    assert!(stdout.contains("--return-format"));
    assert!(stdout.contains("--warnings"));
    assert!(stdout.contains("--quiet"));
    assert!(stdout.contains("--pipe"));
}

#[test]
#[ignore]
fn test_session_basic() {
    // Test running a basic session with goals and format
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("session")
        .arg("--goals")
        .arg("Test session")
        .arg("--return-format")
        .arg("text")
        .output()
        .expect("Failed to execute command");
    
    // Verify the informational output is shown (stderr)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Running session with the following parameters"));
    assert!(stderr.contains("Goals: Test session"));
    assert!(stderr.contains("Return Format: text"));
    
    // Verify the actual output (stdout)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Processed session for goals: Test session"));
}

#[test]
#[ignore]
fn test_session_with_warnings() {
    // Test running a session with warnings
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("session")
        .arg("--goals")
        .arg("Test session with warnings")
        .arg("--return-format")
        .arg("json")
        .arg("--warnings")
        .arg("Be careful with this")
        .output()
        .expect("Failed to execute command");
    
    // Verify warnings are shown in stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Warnings: Be careful with this"));
}

#[test]
#[ignore]
fn test_session_quiet_mode() {
    // Test quiet mode (no informational output)
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("session")
        .arg("--goals")
        .arg("Test quiet mode")
        .arg("--return-format")
        .arg("text")
        .arg("--quiet")
        .output()
        .expect("Failed to execute command");
    
    // With --quiet, no informational output should be shown
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty());
    
    // The main output should still be present
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Processed session for goals: Test quiet mode"));
}

#[test]
#[ignore]
fn test_session_pipe() {
    // Test piping input to the session
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    let output = cmd.arg("session")
        .arg("--goals")
        .arg("Test piped input")
        .arg("--return-format")
        .arg("text")
        .arg("--pipe")
        .write_stdin("This is piped content\n")
        .output()
        .expect("Failed to execute command");
    
    // Verify that the piped content is mentioned in the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("with input: This is piped content"));
    
    // The stderr should mention the piped content
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Context from stdin"));
}

#[test]
#[ignore]
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
        .arg("--return-format")
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