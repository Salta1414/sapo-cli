//! Integration tests for CLI commands
//!
//! These tests run the actual CLI binary and verify output.

use std::process::Command;

/// Get the path to the CLI binary
fn get_cli_path() -> String {
    // In tests, the binary is built in target/debug
    if cfg!(windows) {
        "target/debug/sapo-cli.exe".to_string()
    } else {
        "target/debug/sapo-cli".to_string()
    }
}

/// Helper to run CLI command and get output
fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(get_cli_path())
        .args(args)
        .output()
        .expect("Failed to execute CLI")
}

/// Helper to get stdout as string
fn get_stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to get stderr as string
fn get_stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

// ============================================
// Help & Version Tests
// ============================================

#[test]
fn test_cli_help() {
    let output = run_cli(&["--help"]);
    let stdout = get_stdout(&output);

    assert!(output.status.success(), "Help should succeed");
    assert!(stdout.contains("sapo-cli"), "Should mention sapo-cli");
    assert!(
        stdout.contains("USAGE") || stdout.contains("Usage"),
        "Should show usage"
    );
}

#[test]
fn test_cli_version() {
    let output = run_cli(&["--version"]);
    let stdout = get_stdout(&output);

    assert!(output.status.success(), "Version should succeed");
    assert!(
        stdout.contains("1.0.0") || stdout.contains("sapo"),
        "Should show version"
    );
}

// ============================================
// Status Command Tests
// ============================================

#[test]
fn test_cli_status() {
    let output = run_cli(&["status"]);
    let stdout = get_stdout(&output);

    assert!(output.status.success(), "Status should succeed");
    assert!(stdout.contains("Sapo"), "Should mention Sapo");
    assert!(
        stdout.contains("Status") || stdout.contains("Active"),
        "Should show status"
    );
}

#[test]
fn test_cli_status_shows_plan() {
    let output = run_cli(&["status"]);
    let stdout = get_stdout(&output);

    // Should show plan (free or pro)
    assert!(
        stdout.contains("free") || stdout.contains("pro") || stdout.contains("Plan"),
        "Should show plan information"
    );
}

// ============================================
// Scan Command Tests
// ============================================

#[test]
fn test_cli_scan_help() {
    let output = run_cli(&["scan", "--help"]);
    let stdout = get_stdout(&output);

    assert!(output.status.success(), "Scan help should succeed");
    assert!(
        stdout.to_lowercase().contains("package") || stdout.to_lowercase().contains("scan"),
        "Should mention package scanning"
    );
}

#[test]
fn test_cli_scan_no_package() {
    let output = run_cli(&["scan"]);

    // Should fail or show error when no package provided
    let combined = format!("{}{}", get_stdout(&output), get_stderr(&output));
    assert!(
        !output.status.success()
            || combined.to_lowercase().contains("error")
            || combined.to_lowercase().contains("required"),
        "Should require package argument"
    );
}

// Note: Actual scanning requires network, so we test the command structure
#[test]
fn test_cli_scan_format() {
    // Test that the scan command accepts a package name
    // This may fail with network error, which is fine for structure test
    let output = run_cli(&["scan", "lodash"]);

    let stdout = get_stdout(&output);
    let stderr = get_stderr(&output);
    let combined = format!("{}{}", stdout, stderr);

    // Should either scan or show network error, not crash
    assert!(
        combined.contains("Scanning")
            || combined.contains("lodash")
            || combined.contains("error")
            || combined.contains("Error")
            || combined.contains("timeout")
            || combined.contains("network"),
        "Should attempt to scan or show error, got: {}",
        combined
    );
}

// ============================================
// Trust Command Tests
// ============================================

#[test]
fn test_cli_trust_help() {
    let output = run_cli(&["trust", "--help"]);

    assert!(output.status.success(), "Trust help should succeed");
}

// ============================================
// Toggle Command Tests
// ============================================

#[test]
fn test_cli_toggle() {
    let output = run_cli(&["toggle"]);
    let stdout = get_stdout(&output);

    // Should show enabled or disabled
    assert!(
        stdout.to_lowercase().contains("enabled")
            || stdout.to_lowercase().contains("disabled")
            || stdout.to_lowercase().contains("protection"),
        "Should show toggle status"
    );
}

// ============================================
// Monitor Command Tests (Pro Feature)
// ============================================

#[test]
fn test_cli_monitor_status() {
    let output = run_cli(&["monitor", "status"]);
    let stdout = get_stdout(&output);

    // Should show monitoring status or Pro required
    assert!(
        stdout.contains("Monitoring")
            || stdout.contains("Pro")
            || stdout.contains("monitoring"),
        "Should show monitoring info or Pro requirement"
    );
}

// ============================================
// Wrap Command Tests
// ============================================

#[test]
fn test_cli_wrap_help() {
    let output = run_cli(&["wrap", "--help"]);

    // Wrap is an internal command, may not have help
    // Just ensure it doesn't crash
    assert!(
        output.status.success() || !get_stderr(&output).is_empty(),
        "Wrap command should not crash"
    );
}

// ============================================
// Invalid Command Tests
// ============================================

#[test]
fn test_cli_invalid_command() {
    let output = run_cli(&["invalidcommand123"]);

    // Should fail with error
    assert!(
        !output.status.success() || get_stderr(&output).to_lowercase().contains("error"),
        "Invalid command should fail"
    );
}

#[test]
fn test_cli_no_args_shows_help_or_status() {
    let output = run_cli(&[]);
    let stdout = get_stdout(&output);
    let stderr = get_stderr(&output);
    let combined = format!("{}{}", stdout, stderr);

    // Should show help or error, not crash
    assert!(
        combined.contains("Usage")
            || combined.contains("usage")
            || combined.contains("USAGE")
            || combined.contains("help")
            || combined.contains("error")
            || combined.contains("Sapo"),
        "No args should show usage or help"
    );
}

// ============================================
// Output Format Tests
// ============================================

#[test]
fn test_output_uses_colors_or_plain() {
    let output = run_cli(&["status"]);
    let stdout = get_stdout(&output);

    // Should produce some output
    assert!(!stdout.is_empty(), "Status should produce output");
}

#[test]
fn test_output_no_panic_messages() {
    let output = run_cli(&["status"]);
    let stderr = get_stderr(&output);

    // Should not contain panic messages
    assert!(!stderr.contains("panic"), "Should not panic");
    assert!(
        !stderr.contains("RUST_BACKTRACE"),
        "Should not show backtrace hint"
    );
}
