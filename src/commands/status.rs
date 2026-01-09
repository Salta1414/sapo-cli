use crate::commands::monitor;
use crate::config;
use colored::Colorize;

pub fn run() {
    let device_id = config::get_device_id();
    let plan = config::get_plan();
    let trusted = config::get_trusted();
    let monitoring_enabled = monitor::is_monitoring_enabled();

    let short_device_id = if device_id.len() > 20 {
        format!("{}...", &device_id[..20])
    } else {
        device_id
    };

    let os_name = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Linux"
    };

    let monitoring_status = if monitoring_enabled {
        "ON".green()
    } else {
        "OFF".bright_black()
    };

    println!();
    println!("  {}", "Sapo v1.0.0".green());
    println!("  {} {}", "|-".green(), "Status: Active".green());
    println!("  |- Plan: {} (Unlimited Scans)", plan);
    println!("  |- Trusted packages: {}", trusted.len());
    println!(
        "  |- Runtime Monitoring: {} {}",
        monitoring_status,
        "(sapo monitor toggle)".bright_black()
    );
    println!(
        "  {} Device: {}",
        "|-".bright_black(),
        short_device_id.bright_black()
    );
    println!("  {} OS: {}", "|-".bright_black(), os_name.bright_black());
    println!("  {} Shell: Rust CLI", "+-".bright_black());
    println!();
}
