use crate::api;
use crate::commands::monitor;
use crate::config;
use crate::utils::*;
use colored::Colorize;

pub fn run() {
    println!();
    print_info("Checking Pro status...");

    match api::check_pro_status() {
        Ok(response) => {
            if response.is_pro {
                let plan = response.plan.as_deref().unwrap_or("pro");
                print_ok(&format!("Pro status: Active ({})", plan));
                config::set_config_value("plan", plan);

                // Sync Pro modules
                sync_pro_modules();

                print_ok("Pro features: Enabled");
            } else {
                println!("  {} Pro status: Free", "[>]".bright_black());
                config::set_config_value("plan", "free");
            }
        }
        Err(_) => {
            print_warning("Could not check Pro status - API unreachable");
        }
    }

    println!();
}

/// Sync Pro modules from server
fn sync_pro_modules() {
    // If runtime monitoring is enabled, update the monitor script
    if monitor::is_monitoring_enabled() {
        print_info("Updating runtime monitor...");
        if monitor::download_monitor_script() {
            // Success message already printed by download_monitor_script
        }
    }
}
