//! Local Sandbox Analysis Module
//!
//! This module runs behavioral analysis LOCALLY using OS-native tools:
//! - Linux: strace
//! - macOS: dtruss / fs_usage
//! - Windows: Process Monitor / WMI
//!
//! Results are sent to the server for scoring (scoring logic is server-side).

use crate::api::{self, SandboxAnalyzeRequest, SandboxBehavior, SensitiveAccess};
use crate::config;
use crate::utils::*;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SANDBOX_TIMEOUT_SECS: u64 = 30;

/// Check if sandbox tools are available
pub fn is_sandbox_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        which::which("strace").is_ok()
    }

    #[cfg(target_os = "macos")]
    {
        which::which("dtruss").is_ok() || which::which("fs_usage").is_ok()
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows we use static analysis + basic process monitoring
        true
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

/// Run local sandbox analysis
pub fn run_local_sandbox(package: &str, version: &str) {
    if !is_sandbox_available() {
        println!("  {} Local sandbox not available", "[>]".bright_black());

        #[cfg(target_os = "linux")]
        println!(
            "     {} Install strace: sudo apt install strace",
            "Tip:".bright_black()
        );

        return;
    }

    print_info("Running behavioral sandbox...");
    println!("     {} (Collecting behavior data...)", "".bright_black());

    // Collect behavior data
    let behavior = collect_behavior(package, version);

    if behavior.is_none() {
        print_warning("Sandbox: Could not collect behavior data");
        return;
    }

    let behavior = behavior.unwrap();

    println!(
        "     {} (Sending to server for analysis...)",
        "".bright_black()
    );

    // Send to server for scoring
    let request = SandboxAnalyzeRequest {
        package: package.to_string(),
        version: version.to_string(),
        behavior,
        source: "local".to_string(),
        device_id: config::get_device_id(),
    };

    match api::analyze_sandbox(&request) {
        Ok(response) => {
            let score = response.score.unwrap_or(0);
            let risk_level = response.risk_level.as_deref().unwrap_or("safe");

            match risk_level {
                "dangerous" => {
                    println!(
                        "  {} Sandbox: DANGEROUS behavior detected (score: {})",
                        "[!]".red(),
                        score
                    );
                }
                "warning" => {
                    println!(
                        "  {} Sandbox: Suspicious behavior (score: {})",
                        "[!]".yellow(),
                        score
                    );
                }
                _ => {
                    print_ok(&format!("Sandbox: Clean behavior (score: {})", score));
                }
            }

            // Show flags
            if let Some(flags) = &response.flags {
                for flag in flags.iter().take(5) {
                    if let Some(detail) = &flag.detail {
                        print_detail(detail);
                    }
                }
            }
        }
        Err(_) => {
            print_warning("Sandbox: Server analysis unavailable");
        }
    }
}

/// Collect behavior data using OS-native tools
fn collect_behavior(package: &str, version: &str) -> Option<SandboxBehavior> {
    // Create temp directory
    let temp_dir = std::env::temp_dir().join(format!("sapo-sandbox-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).ok()?;

    // Create package.json
    let package_json = temp_dir.join("package.json");
    fs::write(
        &package_json,
        r#"{"name":"sapo-sandbox-test","version":"1.0.0","private":true}"#,
    )
    .ok()?;

    let pkg_spec = if version == "latest" {
        package.to_string()
    } else {
        format!("{}@{}", package, version)
    };

    // Collect behavior based on OS
    #[cfg(target_os = "linux")]
    let behavior = collect_behavior_linux(&temp_dir, &pkg_spec);

    #[cfg(target_os = "macos")]
    let behavior = collect_behavior_macos(&temp_dir, &pkg_spec);

    #[cfg(target_os = "windows")]
    let behavior = collect_behavior_windows(&temp_dir, &pkg_spec);

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let behavior = None;

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();

    behavior
}

#[cfg(target_os = "linux")]
fn collect_behavior_linux(sandbox_dir: &PathBuf, pkg_spec: &str) -> Option<SandboxBehavior> {
    use std::io::BufRead;

    let strace_log = sandbox_dir.join("strace.log");

    // Run npm install with strace
    let status = Command::new("timeout")
        .args([
            &format!("{}s", SANDBOX_TIMEOUT_SECS),
            "strace",
            "-f",
            "-e",
            "trace=open,openat,connect,execve",
            "-o",
            strace_log.to_str()?,
            "npm",
            "install",
            pkg_spec,
            "--ignore-scripts=false",
        ])
        .current_dir(sandbox_dir)
        .output();

    let exit_code = status.map(|s| s.status.code().unwrap_or(1)).unwrap_or(1);

    let mut behavior = SandboxBehavior {
        exit_code,
        ..Default::default()
    };

    // Parse strace output
    if let Ok(file) = fs::File::open(&strace_log) {
        let reader = std::io::BufReader::new(file);
        for line in reader.lines().flatten() {
            // Parse file access
            if line.contains("open") {
                if let Some(file_path) = extract_quoted_string(&line) {
                    behavior.files_read.push(file_path.clone());

                    // Check for sensitive paths
                    if file_path.contains(".ssh")
                        || file_path.contains(".aws")
                        || file_path.contains(".npmrc")
                    {
                        behavior.sensitive_access.push(SensitiveAccess {
                            access_type: "file".to_string(),
                            path: file_path,
                        });
                    }
                }
            }

            // Parse network connections
            if line.contains("connect") {
                if let Some(ip) = extract_ip_address(&line) {
                    behavior.network_connections.push(api::NetworkConnection {
                        host: None,
                        ip: Some(ip),
                        port: 0,
                    });
                }
            }
        }
    }

    // Also do static analysis on node_modules
    analyze_node_modules(sandbox_dir, &mut behavior);

    Some(behavior)
}

#[cfg(target_os = "macos")]
fn collect_behavior_macos(sandbox_dir: &PathBuf, pkg_spec: &str) -> Option<SandboxBehavior> {
    // On macOS, we primarily use static analysis
    // dtruss requires root and is more complex

    let status = Command::new("npm")
        .args(["install", pkg_spec, "--ignore-scripts=false"])
        .current_dir(sandbox_dir)
        .output();

    let exit_code = status.map(|s| s.status.code().unwrap_or(1)).unwrap_or(1);

    let mut behavior = SandboxBehavior {
        exit_code,
        ..Default::default()
    };

    analyze_node_modules(sandbox_dir, &mut behavior);

    Some(behavior)
}

#[cfg(target_os = "windows")]
fn collect_behavior_windows(sandbox_dir: &PathBuf, pkg_spec: &str) -> Option<SandboxBehavior> {
    // On Windows, we use static analysis

    let status = Command::new("cmd")
        .args(["/c", "npm", "install", pkg_spec, "--ignore-scripts=false"])
        .current_dir(sandbox_dir)
        .output();

    let exit_code = status.map(|s| s.status.code().unwrap_or(1)).unwrap_or(1);

    let mut behavior = SandboxBehavior {
        exit_code,
        ..Default::default()
    };

    analyze_node_modules(sandbox_dir, &mut behavior);

    Some(behavior)
}

/// Static analysis of node_modules for suspicious patterns
fn analyze_node_modules(sandbox_dir: &PathBuf, behavior: &mut SandboxBehavior) {
    let node_modules = sandbox_dir.join("node_modules");
    if !node_modules.exists() {
        return;
    }

    // Patterns to detect
    let patterns = [
        (r"process\.env\.", "process.env access"),
        (r"\.ssh|\.aws|\.npmrc", "Sensitive paths"),
        (r"http\.request|fetch\(|axios", "Network access"),
        (r"child_process|exec\(|spawn\(", "Process spawning"),
        (r"eval\(|new Function\(", "Dynamic code execution"),
    ];

    // Walk through files
    if let Ok(entries) = walkdir(&node_modules) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                for (pattern, description) in &patterns {
                    if content.contains(pattern) {
                        // Add to appropriate category
                        match *description {
                            "process.env access" => {
                                if !behavior
                                    .env_vars_accessed
                                    .contains(&description.to_string())
                                {
                                    behavior.env_vars_accessed.push(description.to_string());
                                }
                            }
                            "Sensitive paths" | "Dynamic code execution" => {
                                behavior.sensitive_access.push(SensitiveAccess {
                                    access_type: "pattern".to_string(),
                                    path: description.to_string(),
                                });
                            }
                            "Network access" => {
                                behavior.network_connections.push(api::NetworkConnection {
                                    host: Some("dynamic".to_string()),
                                    ip: None,
                                    port: 0,
                                });
                            }
                            "Process spawning" => {
                                behavior.processes_spawned.push(api::ProcessSpawned {
                                    executable: "child_process".to_string(),
                                    args: vec![],
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// Simple directory walker
fn walkdir(path: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir(&path)?);
            } else if path
                .extension()
                .map(|e| e == "js" || e == "ts" || e == "mjs" || e == "cjs")
                .unwrap_or(false)
            {
                result.push(path);
            }
        }
    }

    Ok(result)
}

/// Extract first quoted string from a line
fn extract_quoted_string(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line[start + 1..].find('"')?;
    Some(line[start + 1..start + 1 + end].to_string())
}

/// Extract IP address from a line
fn extract_ip_address(line: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"\d+\.\d+\.\d+\.\d+").ok()?;
    re.find(line).map(|m| m.as_str().to_string())
}
