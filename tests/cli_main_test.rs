use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_version() {
    // Test that the version flag works
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("--version").output().expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.2.0"));
}

#[test]
fn test_cli_help() {
    // Test the main help output
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("--help").output().expect("Failed to execute command");
    
    // Check that all subcommands are listed
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("start"));
    assert!(stdout.contains("prompt"));
    assert!(stdout.contains("configure"));
    assert!(stdout.contains("models"));
    assert!(stdout.contains("session"));
    assert!(stdout.contains("non-think"));
    assert!(stdout.contains("settings"));
}

#[test]
fn test_no_args() {
    // Test running without arguments shows help message
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.output().expect("Failed to execute command");
    
    // Should show message about no subcommand used
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No subcommand was used"));
}

#[test]
fn test_invalid_subcommand() {
    // Test an invalid subcommand shows an error
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("invalid-command").output().expect("Failed to execute command");
    
    // Should show error about invalid subcommand
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error: unrecognized subcommand"));
}

#[test]
fn test_start_command() {
    // Test the basic start command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("start").output().expect("Failed to execute command");
    
    // Should show starting message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Starting the application"));
}

#[test]
fn test_start_verbose() {
    // Test the start command with verbose flag
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("start").arg("--verbose").output().expect("Failed to execute command");
    
    // Should show verbose mode message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Running in verbose mode"));
}