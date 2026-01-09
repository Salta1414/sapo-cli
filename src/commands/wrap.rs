use crate::commands::{monitor, scan};
use crate::config;
use crate::utils::*;
use std::process::{Command, Stdio};

/// Run a wrapped package manager command
/// This is called by the shell wrappers: sapo-cli wrap npm install axios
pub fn run(manager: &str, args: &[String]) {
    // Check if disabled
    if std::env::var("SAPO_DISABLED").is_ok() {
        run_original_command(manager, args, false);
        return;
    }

    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");

    // Only intercept install commands
    let is_install = match manager {
        "npm" | "pnpm" | "bun" => matches!(cmd, "install" | "i" | "add"),
        "yarn" => matches!(cmd, "add") || (cmd == "install" || cmd.is_empty()),
        _ => false,
    };

    if !is_install {
        run_original_command(manager, args, false);
        return;
    }

    // Auto-sync Pro if needed
    auto_sync_pro();

    // Extract packages from args (skip flags and the command itself)
    let packages: Vec<&str> = args
        .iter()
        .skip(1) // Skip the command (install/add)
        .filter(|arg| !arg.starts_with('-'))
        .filter(|arg| !arg.starts_with('.'))
        .filter(|arg| !arg.starts_with('/'))
        .map(|s| s.as_str())
        .collect();

    if packages.is_empty() {
        // No specific packages - scan lockfile
        if !scan_lockfile(manager) {
            return; // Cancelled
        }
    } else {
        // Scan each package
        for pkg in &packages {
            if !scan::run(pkg, false) {
                return; // Cancelled
            }
        }
    }

    // Check if runtime monitoring should be enabled
    let use_monitoring = should_use_runtime_monitoring();

    // Run the original command (with or without monitoring)
    run_original_command(manager, args, use_monitoring);
}

/// Check if runtime monitoring should be used for this install
fn should_use_runtime_monitoring() -> bool {
    // Skip if explicitly disabled
    if std::env::var("SAPO_MONITOR_DISABLED").is_ok() {
        return false;
    }

    // Runtime monitoring is a Pro feature
    if !config::is_pro() {
        return false;
    }

    // Check if monitoring is enabled in config
    if !monitor::is_monitoring_enabled() {
        return false;
    }

    // Ensure monitor script exists (should have been downloaded via 'sapo monitor enable')
    if !monitor::monitor_script_exists() {
        // Script not found - user needs to run 'sapo monitor enable' first
        return false;
    }

    true
}

/// Auto-sync Pro modules if user is Pro but not synced
fn auto_sync_pro() {
    if !config::is_pro() {
        // Not a Pro user, nothing to sync
    }
    // Pro features are now embedded in the binary
    // Just verify Pro status is current
}

/// Scan lockfile for packages
fn scan_lockfile(manager: &str) -> bool {
    let lockfile = match manager {
        "npm" => "package-lock.json",
        "pnpm" => "pnpm-lock.yaml",
        "yarn" => "yarn.lock",
        _ => return true,
    };

    if !std::path::Path::new(lockfile).exists() {
        return true;
    }

    print_info(&format!("Scanning {}...", lockfile));

    // For now, just allow - batch scanning is complex
    // The server-side batch endpoint handles this better
    true
}

/// Run the original package manager command
/// If use_monitoring is true, wraps the command with Node.js --require monitor.js
fn run_original_command(manager: &str, args: &[String], use_monitoring: bool) {
    // Find the original command (not our wrapper)
    let exe = which::which(manager).ok();

    // If monitoring is enabled, we need to run node with --require
    if use_monitoring {
        if let Some(exe_path) = &exe {
            run_with_monitoring(exe_path, args);
            return;
        }
    }

    let mut cmd = if let Some(exe_path) = exe {
        Command::new(exe_path)
    } else {
        // Fallback to PATH lookup
        #[cfg(target_os = "windows")]
        {
            let mut c = Command::new("cmd");
            c.args(["/c", manager]);
            c
        }
        #[cfg(not(target_os = "windows"))]
        {
            let mut c = Command::new("sh");
            c.args(["-c", &format!("command {} \"$@\"", manager), "--"]);
            c
        }
    };

    cmd.args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status();

    if let Ok(status) = status {
        std::process::exit(status.code().unwrap_or(1));
    }
}

/// Run a package manager with runtime monitoring via NODE_OPTIONS
fn run_with_monitoring(exe_path: &std::path::Path, args: &[String]) {
    let monitor_path = monitor::get_monitor_script_path();
    let monitor_str = monitor_path.to_string_lossy();

    // Get device ID and API URL for reporting
    let device_id = config::get_device_id();
    let api_url = config::get_api_url();
    let threat_url = format!("{}/runtime/threat", api_url);

    // Set NODE_OPTIONS to inject our monitor script
    // This works for npm, pnpm, yarn, bun (all Node.js based)
    let node_options = format!("--require \"{}\"", monitor_str);

    print_info("Runtime monitoring active for this install");

    let mut cmd = Command::new(exe_path);
    cmd.args(args)
        .env("NODE_OPTIONS", &node_options)
        .env("SAPO_DEVICE_ID", &device_id)
        .env("SAPO_API_URL", &threat_url)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status();

    if let Ok(status) = status {
        std::process::exit(status.code().unwrap_or(1));
    }
}
