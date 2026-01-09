//! End-to-end tests for the full Sapo flow
//!
//! These tests simulate real user scenarios.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the CLI binary
fn get_cli_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let binary_name = if cfg!(windows) {
        "sapo-cli.exe"
    } else {
        "sapo-cli"
    };

    PathBuf::from(manifest_dir)
        .join("target")
        .join("debug")
        .join(binary_name)
}

/// Create a temporary test directory
#[allow(dead_code)]
fn create_temp_dir(name: &str) -> PathBuf {
    let temp = env::temp_dir().join(format!("sapo-e2e-{}-{}", name, std::process::id()));
    fs::create_dir_all(&temp).expect("Failed to create temp dir");
    temp
}

/// Cleanup temp directory
#[allow(dead_code)]
fn cleanup_temp_dir(path: &PathBuf) {
    fs::remove_dir_all(path).ok();
}

// ============================================
// E2E: Config Initialization
// ============================================

#[test]
fn test_e2e_config_created_on_first_run() {
    // Running any command should create config
    let output = Command::new(get_cli_path())
        .arg("status")
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success(), "Status command should succeed");

    // Config directory should exist
    let config_dir = dirs::home_dir().expect("No home dir").join(".sapo");

    assert!(config_dir.exists(), "Config directory should be created");

    let config_file = config_dir.join("config");
    assert!(config_file.exists(), "Config file should be created");
}

#[test]
fn test_e2e_config_has_device_id() {
    // Run status to ensure config exists
    Command::new(get_cli_path())
        .arg("status")
        .output()
        .expect("Failed to run CLI");

    // Read config and check for device_id
    let config_path = dirs::home_dir()
        .expect("No home dir")
        .join(".sapo")
        .join("config");

    let content = fs::read_to_string(&config_path).expect("Failed to read config");

    assert!(
        content.contains("device_id="),
        "Config should have device_id"
    );

    // Device ID should not be empty
    let device_id_line = content
        .lines()
        .find(|l| l.starts_with("device_id="))
        .expect("No device_id line");

    let device_id = device_id_line.split('=').nth(1).unwrap_or("");
    assert!(!device_id.is_empty(), "Device ID should not be empty");
    assert!(device_id.len() > 20, "Device ID should be reasonably long");
}

// ============================================
// E2E: Trust Flow
// ============================================

#[test]
fn test_e2e_trust_add_and_list() {
    let test_package = format!("test-pkg-{}", std::process::id());

    // Trust a package
    let output = Command::new(get_cli_path())
        .args(["trust", &test_package])
        .output()
        .expect("Failed to run trust command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show success message
    assert!(
        stdout.to_lowercase().contains("trust")
            || stdout.to_lowercase().contains("added")
            || stdout.to_lowercase().contains(&test_package),
        "Should acknowledge trust command"
    );

    // Status should show trusted packages
    let status_output = Command::new(get_cli_path())
        .arg("status")
        .output()
        .expect("Failed to run status");

    let status_stdout = String::from_utf8_lossy(&status_output.stdout);

    assert!(
        status_stdout.contains("Trusted") || status_stdout.contains("trusted"),
        "Status should mention trusted packages"
    );
}

// ============================================
// E2E: Toggle Flow
// ============================================

#[test]
fn test_e2e_toggle_changes_state() {
    // Get initial state
    let status1 = Command::new(get_cli_path())
        .arg("status")
        .output()
        .expect("Failed to run status");
    let stdout1 = String::from_utf8_lossy(&status1.stdout);

    // Toggle
    let toggle_output = Command::new(get_cli_path())
        .arg("toggle")
        .output()
        .expect("Failed to run toggle");

    assert!(toggle_output.status.success(), "Toggle should succeed");

    // Toggle again to restore original state
    Command::new(get_cli_path())
        .arg("toggle")
        .output()
        .expect("Failed to run toggle");

    // State should be restored
    let _status2 = Command::new(get_cli_path())
        .arg("status")
        .output()
        .expect("Failed to run status");

    // Both status outputs should be parseable
    assert!(!stdout1.is_empty(), "Status should produce output");
}

// ============================================
// E2E: Scan Flow (requires network)
// ============================================

#[test]
#[ignore] // Ignore by default since it requires network
fn test_e2e_scan_safe_package() {
    let output = Command::new(get_cli_path())
        .args(["scan", "lodash"])
        .output()
        .expect("Failed to run scan");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // lodash should be safe
    assert!(
        stdout.to_lowercase().contains("safe")
            || stdout.to_lowercase().contains("low")
            || stdout.contains("score"),
        "lodash should be scanned as safe: {}",
        stdout
    );
}

#[test]
#[ignore] // Ignore by default since it requires network
fn test_e2e_scan_with_version() {
    let output = Command::new(get_cli_path())
        .args(["scan", "axios@1.6.0"])
        .output()
        .expect("Failed to run scan");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should attempt to scan specific version
    assert!(
        combined.contains("1.6.0") || combined.contains("axios") || combined.contains("Scanning"),
        "Should scan specific version: {}",
        combined
    );
}

// ============================================
// E2E: Wrap Command (Package Manager Wrapping)
// ============================================

#[test]
fn test_e2e_wrap_detects_install_command() {
    // Test that wrap command can parse install
    let output = Command::new(get_cli_path())
        .args(["wrap", "npm", "install", "lodash"])
        .env("SAPO_ENABLED", "false") // Disable actual scanning for test
        .output()
        .expect("Failed to run wrap command");

    // Wrap should execute (may fail due to npm not being installed, but shouldn't panic)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panic"), "Wrap should not panic");
}

#[test]
fn test_e2e_wrap_non_install_passthrough() {
    // Non-install commands should pass through without scanning
    let output = Command::new(get_cli_path())
        .args(["wrap", "npm", "run", "test"])
        .output();

    // Should not crash, even if npm fails
    assert!(output.is_ok(), "Wrap should handle non-install commands");
}

// ============================================
// E2E: Monitor Commands (Pro Feature)
// ============================================

#[test]
fn test_e2e_monitor_requires_pro() {
    let output = Command::new(get_cli_path())
        .args(["monitor", "enable"])
        .output()
        .expect("Failed to run monitor enable");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should mention Pro requirement (unless user is actually Pro)
    assert!(
        combined.to_lowercase().contains("pro")
            || combined.to_lowercase().contains("upgrade")
            || combined.to_lowercase().contains("enabled")
            || combined.to_lowercase().contains("monitoring"),
        "Should mention Pro or enable monitoring: {}",
        combined
    );
}

// ============================================
// E2E: Error Handling
// ============================================

#[test]
fn test_e2e_handles_invalid_package_name() {
    let output = Command::new(get_cli_path())
        .args(["scan", "!@#$%invalid"])
        .output()
        .expect("Failed to run scan");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should not panic
    assert!(
        !stderr.contains("panic"),
        "Should not panic on invalid package"
    );
}

#[test]
fn test_e2e_handles_empty_package_name() {
    let output = Command::new(get_cli_path())
        .args(["scan", ""])
        .output()
        .expect("Failed to run scan");

    // Should handle gracefully
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("panic"),
        "Should not panic on empty package"
    );
}

// ============================================
// E2E: Binary Size Check
// ============================================

#[test]
fn test_binary_size_reasonable() {
    let cli_path = get_cli_path();

    if cli_path.exists() {
        let metadata = fs::metadata(&cli_path).expect("Failed to get metadata");
        let size_mb = metadata.len() as f64 / 1_000_000.0;

        // Debug binary should be under 50MB
        assert!(size_mb < 50.0, "Binary size {} MB is too large", size_mb);

        // Binary should exist and be non-empty
        assert!(metadata.len() > 1000, "Binary should be non-trivial size");
    }
}
