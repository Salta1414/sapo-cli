use crate::config;
use crate::utils::*;
use colored::Colorize;
use std::fs;

pub fn run() {
    print_warning("Uninstalling Sapo...");
    
    // Remove config directory
    let config_dir = config::get_config_dir();
    if config_dir.exists() {
        fs::remove_dir_all(&config_dir).ok();
    }
    
    println!();
    print_ok("Sapo files removed.");
    println!();
    println!("  {} To complete uninstallation:", "Note:".yellow());
    
    #[cfg(target_os = "windows")]
    {
        println!("  1. Edit your PowerShell profile: notepad $PROFILE");
        println!("  2. Remove the lines between '# === SAPO START ===' and '# === SAPO END ==='");
        println!("  3. Restart your terminal");
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        println!("  1. Edit your shell config (~/.bashrc or ~/.zshrc)");
        println!("  2. Remove the lines between '# === SAPO START ===' and '# === SAPO END ==='");
        println!("  3. Restart your terminal");
    }
    
    println!();
}
