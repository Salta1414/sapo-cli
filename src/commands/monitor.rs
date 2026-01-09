use crate::api;
use crate::config;
use crate::utils::*;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Get the path to the monitor.js script
pub fn get_monitor_script_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".sapo")
        .join("monitor.js")
}

/// Get the path to the monitor log file
pub fn get_monitor_log_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".sapo")
        .join("monitor.log")
}

/// Download and install monitor.js from server (Pro feature)
pub fn download_monitor_script() -> bool {
    print_info("Downloading runtime monitor...");
    
    match api::download_pro_module("monitor.js") {
        Ok(script_content) => {
            let script_path = get_monitor_script_path();
            
            // Ensure .sapo directory exists
            if let Some(parent) = script_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    print_error(&format!("Failed to create .sapo directory: {}", e));
                    return false;
                }
            }
            
            // Write monitor script
            if let Err(e) = fs::write(&script_path, script_content) {
                print_error(&format!("Failed to write monitor.js: {}", e));
                return false;
            }
            
            print_ok("Runtime monitor downloaded");
            true
        }
        Err(e) => {
            if e.contains("Pro subscription required") {
                print_error("Runtime monitoring is a Pro feature");
                print_info("Upgrade at: sapo upgrade");
            } else {
                print_error(&format!("Failed to download monitor: {}", e));
            }
            false
        }
    }
}

/// Check if runtime monitoring is enabled
pub fn is_monitoring_enabled() -> bool {
    config::get_config_value("runtime_monitoring")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Check if the monitor script exists
pub fn monitor_script_exists() -> bool {
    get_monitor_script_path().exists()
}

/// Show monitoring status
pub fn show_status() {
    let is_pro = config::is_pro();
    let script_exists = monitor_script_exists();
    let is_enabled = is_monitoring_enabled();
    let log_path = get_monitor_log_path();
    
    println!();
    print_section_header("Runtime Monitoring Status (Pro Feature)");
    println!();
    
    // Pro status
    if is_pro {
        print_ok("Pro subscription active");
    } else {
        print_warning("Pro subscription required");
        print_info("Upgrade at: sapo upgrade");
        println!();
        return;
    }
    
    // Script status
    if script_exists {
        print_ok("Monitor script installed");
    } else {
        print_warning("Monitor script not installed");
        print_info("Run: sapo monitor enable");
    }
    
    // Enabled status
    if is_enabled {
        print_ok("Runtime monitoring enabled");
    } else {
        print_info("Runtime monitoring disabled");
    }
    
    // Log file
    if log_path.exists() {
        if let Ok(metadata) = fs::metadata(&log_path) {
            let size_kb = metadata.len() / 1024;
            print_info(&format!("Log file: {} ({} KB)", log_path.display(), size_kb));
            
            // Count threats
            if let Ok(file) = fs::File::open(&log_path) {
                let reader = BufReader::new(file);
                let count = reader.lines().count();
                print_info(&format!("Total threats logged: {}", count));
            }
        }
    } else {
        print_info("No threats logged yet");
    }
    
    println!();
}

/// Enable runtime monitoring (Pro feature)
pub fn enable() {
    // Check Pro status first
    if !config::is_pro() {
        print_error("Runtime monitoring is a Pro feature");
        print_info("Upgrade at: sapo upgrade");
        println!();
        print_info("Or link your device: sapo login");
        return;
    }
    
    // Download/update monitor script from server
    if !download_monitor_script() {
        return;
    }
    
    // Update config
    config::set_config_value("runtime_monitoring", "true");
    
    print_ok("Runtime monitoring enabled");
    print_info("All npm install commands will now be monitored for suspicious behavior");
}

/// Disable runtime monitoring
pub fn disable() {
    config::set_config_value("runtime_monitoring", "false");
    
    print_ok("Runtime monitoring disabled");
}

/// Toggle runtime monitoring on/off (Pro feature)
pub fn toggle() {
    let currently_enabled = is_monitoring_enabled();
    
    if currently_enabled {
        disable();
    } else {
        // enable() already checks Pro status
        enable();
    }
}

/// Show recent threats from log file
pub fn show_threats(count: usize) {
    let log_path = get_monitor_log_path();
    
    if !log_path.exists() {
        print_info("No threats logged yet");
        return;
    }
    
    // Read last N lines
    let file = match fs::File::open(&log_path) {
        Ok(f) => f,
        Err(e) => {
            print_error(&format!("Failed to read log: {}", e));
            return;
        }
    };
    
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
    
    if lines.is_empty() {
        print_info("No threats logged yet");
        return;
    }
    
    println!();
    print_section_header(&format!("Recent Threats (last {})", count));
    println!();
    
    let start = if lines.len() > count { lines.len() - count } else { 0 };
    
    for line in lines.iter().skip(start) {
        if let Ok(threat) = serde_json::from_str::<serde_json::Value>(line) {
            let threat_type = threat.get("threatType")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let package = threat.get("packageName")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let blocked = threat.get("blocked")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let status = if blocked { "BLOCKED" } else { "DETECTED" };
            let color_fn = if blocked { print_error } else { print_warning };
            
            color_fn(&format!("[{}] {} - {}", status, package, format_threat_type(threat_type)));
            
            // Show details if available
            if let Some(details) = threat.get("details") {
                if let Some(path) = details.get("path").and_then(|v| v.as_str()) {
                    println!("    Path: {}", path);
                }
                if let Some(url) = details.get("url").and_then(|v| v.as_str()) {
                    println!("    URL: {}", url);
                }
                if let Some(cmd) = details.get("command").and_then(|v| v.as_str()) {
                    println!("    Command: {}", cmd);
                }
                if let Some(var) = details.get("variable").and_then(|v| v.as_str()) {
                    println!("    Env Var: {}", var);
                }
            }
        }
    }
    
    println!();
}

/// Clear the threat log
pub fn clear_log() {
    let log_path = get_monitor_log_path();
    
    if log_path.exists() {
        if let Err(e) = fs::remove_file(&log_path) {
            print_error(&format!("Failed to clear log: {}", e));
            return;
        }
    }
    
    print_ok("Threat log cleared");
}

fn format_threat_type(threat_type: &str) -> &str {
    match threat_type {
        "credential_access" => "Credential File Access",
        "network_exfil" => "Suspicious Network Request",
        "credential_exfil" => "Credential Exfiltration Attempt",
        "process_spawn" => "Suspicious Process Spawn",
        "env_access" => "Sensitive Env Var Access",
        "suspicious_connection" => "Suspicious Network Connection",
        _ => threat_type,
    }
}
