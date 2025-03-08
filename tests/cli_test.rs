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
    
    // Check that all subcommands are listed in the help output
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("prompt"))
        .stdout(predicate::str::contains("configure"))
        .stdout(predicate::str::contains("models"))
        .stdout(predicate::str::contains("session"))
        .stdout(predicate::str::contains("non-think"))
        .stdout(predicate::str::contains("settings"));
}

#[test]
fn test_no_args() {
    // Test running without arguments shows help message
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Should show message about no subcommand used
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("No subcommand was used"));
}

#[test]
fn test_prompt_help() {
    // Test help text for the prompt command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Check that all flags are listed in the help output
    cmd.arg("prompt")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--goals"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--warnings"))
        .stdout(predicate::str::contains("--clipboard"))
        .stdout(predicate::str::contains("--quiet"))
        .stdout(predicate::str::contains("--pipe"))
        .stdout(predicate::str::contains("--no-thinking"))
        .stdout(predicate::str::contains("--recursion"));
}

#[test]
fn test_non_think_help() {
    // Test help text for the non-think command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Check that all flags are listed in the help output
    cmd.arg("non-think")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--prompt"))
        .stdout(predicate::str::contains("--clipboard"))
        .stdout(predicate::str::contains("--quiet"))
        .stdout(predicate::str::contains("--pipe"))
        .stdout(predicate::str::contains("--filter-thinking"));
}

#[test]
fn test_configure_help() {
    // Test help text for the configure command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Check that all flags are listed in the help output
    cmd.arg("configure")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--provider"))
        .stdout(predicate::str::contains("--api-key"))
        .stdout(predicate::str::contains("--model"));
}

#[test]
fn test_settings_help() {
    // Test help text for the settings command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Check that all flags are listed in the help output
    cmd.arg("settings")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--view"))
        .stdout(predicate::str::contains("--default-model"))
        .stdout(predicate::str::contains("--default-format"))
        .stdout(predicate::str::contains("--logging"))
        .stdout(predicate::str::contains("--log-file"))
        .stdout(predicate::str::contains("--reset"));
}

#[test]
fn test_models_help() {
    // Test help text for the models command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Check that all flags are listed in the help output
    cmd.arg("models")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--provider"))
        .stdout(predicate::str::contains("--quiet"));
}

#[test]
fn test_session_help() {
    // Test help text for the session command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Check that all flags are listed in the help output
    cmd.arg("session")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--goals"))
        .stdout(predicate::str::contains("--return-format"))
        .stdout(predicate::str::contains("--warnings"))
        .stdout(predicate::str::contains("--quiet"))
        .stdout(predicate::str::contains("--pipe"));
}

#[test]
fn test_start_command() {
    // Test the basic start command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Should show starting message
    cmd.arg("start")
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting the application"));
}

#[test]
fn test_start_verbose() {
    // Test the start command with verbose flag
    let mut cmd = Command::cargo_bin("ola").unwrap();
    
    // Should show verbose mode message
    cmd.arg("start")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Running in verbose mode"));
}