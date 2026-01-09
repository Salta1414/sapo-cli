use crate::config;
use crate::utils::*;
use colored::Colorize;

pub fn add(package: &str) {
    if config::is_trusted(package) {
        print_warning(&format!("{} is already trusted", package));
    } else {
        config::add_trusted(package);
        print_ok(&format!("Added {} to trusted packages", package));
    }
}

pub fn remove(package: &str) {
    if config::is_trusted(package) {
        config::remove_trusted(package);
        print_ok(&format!("Removed {} from trusted packages", package));
    } else {
        print_warning(&format!("{} is not in trusted list", package));
    }
}

pub fn list() {
    let trusted = config::get_trusted();
    
    if trusted.is_empty() {
        println!("  {}", "No trusted packages".bright_black());
    } else {
        println!();
        println!("  {}", "Trusted packages (skipped during scan):".green());
        for pkg in trusted {
            println!("    {} {}", "-".cyan(), pkg.cyan());
        }
        println!();
    }
}
