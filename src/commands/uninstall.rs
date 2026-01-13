use crate::config;
use crate::utils::*;
use colored::Colorize;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub fn run() {
    println!();
    print_warning("Uninstalling Sapo...");
    println!();

    let mut success = true;

    // 1. Remove config directory (~/.sapo)
    let config_dir = config::get_config_dir();
    if config_dir.exists() {
        match fs::remove_dir_all(&config_dir) {
            Ok(_) => print_ok("Removed ~/.sapo directory"),
            Err(e) => {
                print_error(&format!("Could not remove ~/.sapo: {}", e));
                success = false;
            }
        }
    } else {
        print_info("~/.sapo directory not found (already removed)");
    }

    // 2. Clean shell config files
    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir().expect("No home directory");
        let shell_configs = vec![
            home.join(".bashrc"),
            home.join(".bash_profile"),
            home.join(".zshrc"),
            home.join(".profile"),
        ];

        for config_path in shell_configs {
            if config_path.exists() {
                match remove_sapo_from_file(&config_path) {
                    Ok(true) => print_ok(&format!(
                        "Cleaned {}",
                        config_path.file_name().unwrap().to_string_lossy()
                    )),
                    Ok(false) => {} // No Sapo config found, skip silently
                    Err(e) => {
                        print_error(&format!("Could not clean {}: {}", config_path.display(), e));
                        success = false;
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try to clean PowerShell profile
        if let Some(profile_path) = get_powershell_profile() {
            if profile_path.exists() {
                match remove_sapo_from_file(&profile_path) {
                    Ok(true) => print_ok("Cleaned PowerShell profile"),
                    Ok(false) => {} // No Sapo config found
                    Err(e) => {
                        print_error(&format!("Could not clean PowerShell profile: {}", e));
                        success = false;
                    }
                }
            }
        }
    }

    // 3. Final message
    println!();
    if success {
        print_ok("Sapo has been completely uninstalled!");
        println!();
        println!(
            "  {}",
            "Please restart your terminal for changes to take effect.".bright_black()
        );
    } else {
        print_warning("Sapo partially uninstalled. Some files may need manual removal.");
        println!();
        println!("  {} Manual steps:", "Note:".yellow());

        #[cfg(target_os = "windows")]
        {
            println!("  1. Open PowerShell profile: notepad $PROFILE");
            println!("  2. Remove lines between '# === SAPO START ===' and '# === SAPO END ==='");
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("  1. Edit ~/.bashrc or ~/.zshrc");
            println!("  2. Remove lines between '# === SAPO START ===' and '# === SAPO END ==='");
        }

        println!("  3. Restart your terminal");
    }
    println!();
}

/// Remove Sapo configuration block from a file
/// Returns Ok(true) if Sapo config was found and removed
/// Returns Ok(false) if no Sapo config was found
fn remove_sapo_from_file(path: &PathBuf) -> Result<bool, std::io::Error> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut new_lines: Vec<String> = Vec::new();
    let mut in_sapo_block = false;
    let mut found_sapo = false;

    for line in reader.lines().map_while(Result::ok) {
        if line.contains("# === SAPO START ===") {
            in_sapo_block = true;
            found_sapo = true;
            continue;
        }

        if line.contains("# === SAPO END ===") {
            in_sapo_block = false;
            continue;
        }

        if !in_sapo_block {
            new_lines.push(line);
        }
    }

    if !found_sapo {
        return Ok(false);
    }

    // Remove trailing empty lines
    while new_lines.last().is_some_and(|l| l.trim().is_empty()) {
        new_lines.pop();
    }

    // Write back
    let mut file = fs::File::create(path)?;
    for line in new_lines {
        writeln!(file, "{}", line)?;
    }

    Ok(true)
}

#[cfg(target_os = "windows")]
fn get_powershell_profile() -> Option<PathBuf> {
    // PowerShell profile is typically at:
    // $HOME\Documents\PowerShell\Microsoft.PowerShell_profile.ps1
    // or $HOME\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
    let home = dirs::home_dir()?;

    // Try modern PowerShell (Core) first
    let ps_core = home
        .join("Documents")
        .join("PowerShell")
        .join("Microsoft.PowerShell_profile.ps1");
    if ps_core.exists() {
        return Some(ps_core);
    }

    // Try Windows PowerShell
    let ps_windows = home
        .join("Documents")
        .join("WindowsPowerShell")
        .join("Microsoft.PowerShell_profile.ps1");
    if ps_windows.exists() {
        return Some(ps_windows);
    }

    // Return core path even if it doesn't exist (for error message)
    Some(ps_core)
}
