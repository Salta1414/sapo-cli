//! Unit tests for config module

use super::*;

#[test]
fn test_generate_device_id_format() {
    let device_id = generate_device_id();

    // Should start with platform prefix
    #[cfg(target_os = "windows")]
    assert!(
        device_id.starts_with("win_"),
        "Device ID should start with 'win_'"
    );

    #[cfg(target_os = "macos")]
    assert!(
        device_id.starts_with("mac_"),
        "Device ID should start with 'mac_'"
    );

    #[cfg(target_os = "linux")]
    assert!(
        device_id.starts_with("linux_"),
        "Device ID should start with 'linux_'"
    );

    // Should contain underscore separator
    assert!(
        device_id.contains('_'),
        "Device ID should contain underscore"
    );

    // Should be reasonably long (UUID is 36 chars + prefix)
    assert!(
        device_id.len() > 30,
        "Device ID should be at least 30 chars"
    );
}

#[test]
fn test_generate_device_id_unique() {
    let id1 = generate_device_id();
    let id2 = generate_device_id();

    assert_ne!(id1, id2, "Each generated ID should be unique");
}

#[test]
fn test_config_path_contains_sapo() {
    let config_path = get_config_path();
    let path_str = config_path.to_string_lossy();

    assert!(
        path_str.contains(".sapo"),
        "Config path should contain .sapo"
    );
    assert!(
        path_str.ends_with("config"),
        "Config path should end with 'config'"
    );
}

#[test]
fn test_bin_dir_path() {
    let bin_dir = get_bin_dir();
    let path_str = bin_dir.to_string_lossy();

    assert!(path_str.contains(".sapo"), "Bin dir should contain .sapo");
    assert!(path_str.ends_with("bin"), "Bin dir should end with 'bin'");
}

#[test]
fn test_is_pro_free_plan() {
    // This tests the logic, not the actual config
    assert!(!is_plan_pro("free"), "free should not be pro");
    assert!(is_plan_pro("pro"), "pro should be pro");
    assert!(is_plan_pro("enterprise"), "enterprise should be pro");
}

/// Helper to test plan logic without touching actual config
fn is_plan_pro(plan: &str) -> bool {
    plan == "pro" || plan == "enterprise"
}

#[test]
fn test_trusted_list_parsing() {
    // Test parsing logic
    let trusted_str = "lodash,axios,react";
    let trusted: Vec<String> = trusted_str
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    assert_eq!(trusted.len(), 3);
    assert!(trusted.contains(&"lodash".to_string()));
    assert!(trusted.contains(&"axios".to_string()));
    assert!(trusted.contains(&"react".to_string()));
}

#[test]
fn test_trusted_list_empty() {
    let trusted_str = "";
    let trusted: Vec<String> = trusted_str
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    assert!(
        trusted.is_empty(),
        "Empty string should result in empty list"
    );
}

#[test]
fn test_config_line_parsing() {
    let line = "device_id=mac_abc123";
    let (key, value) = line.split_once('=').unwrap();

    assert_eq!(key, "device_id");
    assert_eq!(value, "mac_abc123");
}

#[test]
fn test_config_line_with_spaces() {
    let line = "  api_url  =  https://example.com  ";
    let (key, value) = line.split_once('=').unwrap();

    assert_eq!(key.trim(), "api_url");
    assert_eq!(value.trim(), "https://example.com");
}
