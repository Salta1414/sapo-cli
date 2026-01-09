use crate::api;
use crate::config;
use crate::sandbox;
use crate::utils::*;
use colored::Colorize;

pub fn run(package: &str, verbose: bool) -> bool {
    // Skip flags and paths
    if package.starts_with('-') || package.starts_with('.') || package.starts_with('/') {
        return true;
    }

    // Extract package name without version
    let pkg_name = package.split('@').next().unwrap_or(package);

    // Check whitelist
    if config::is_trusted(pkg_name) {
        println!();
        print_ok(&format!("[TRUSTED] {} (skipped)", pkg_name));
        return true;
    }

    println!();
    print_info(&format!("Scanning {}...", package));

    match api::scan_package(package) {
        Ok(response) => {
            let risk_level = response.risk_level.as_deref().unwrap_or("unknown");
            let message = response.message.as_deref().unwrap_or("No details");
            let pkg_display = response.package.as_deref().unwrap_or(package);
            let version = response.version.as_deref().unwrap_or("latest");

            match risk_level {
                "safe" => {
                    print_ok(&format!("{}@{}", pkg_display, version));
                    print_detail(message);
                }
                "warning" => {
                    print_warning(&format!("WARNING: {}@{}", pkg_display, version));
                    print_detail(message);
                }
                "dangerous" => {
                    print_blocked(&format!("{}@{}", pkg_display, version));
                    print_detail(message);
                    println!();

                    // Ask for confirmation
                    print!("     Continue anyway? (y/N) ");
                    std::io::Write::flush(&mut std::io::stdout()).ok();

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).ok();

                    if input.trim().to_lowercase() != "y" {
                        print_error("Installation cancelled");
                        return false;
                    }
                }
                _ => {
                    print_info(&format!("{}@{}", pkg_display, version));
                    print_detail(message);
                }
            }

            // Show version anomaly info (Pro feature)
            if let Some(prev_version) = &response.previous_version {
                println!("     {} Compared with: {}", "|-".cyan(), prev_version);
            }

            // Show anomaly reasons
            if let Some(reasons) = &response.anomaly_reasons {
                for reason in reasons {
                    println!("     {} {}", "|-".magenta(), reason);
                }
            }

            // Handle sandbox results
            let pro_scanned = response.scanned.unwrap_or(false);

            if !pro_scanned {
                // Show Pro hint for free users
                println!();
                println!(
                    "  {}",
                    "+-------------------------------------------------+".bright_black()
                );
                println!(
                    "  {}",
                    "| PRO features not scanned:                       |".bright_black()
                );
                println!(
                    "  {}",
                    "|    * Version Anomaly Detection                  |".bright_black()
                );
                println!(
                    "  {}",
                    "|    * Script Analysis                            |".bright_black()
                );
                println!(
                    "  {}",
                    "|    * Behavioral Sandbox                         |".bright_black()
                );
                println!(
                    "  {}",
                    "|    * Quarantine Alert                           |".bright_black()
                );
                println!(
                    "  {}",
                    "|    Upgrade: sapo upgrade                        |".bright_black()
                );
                println!(
                    "  {}",
                    "+-------------------------------------------------+".bright_black()
                );
            } else {
                // Pro user - check sandbox
                if response.sandbox_cached.unwrap_or(false) {
                    // Show cached sandbox results
                    print_info("Sandbox: Cache-Hit");
                    let sandbox_risk = response.sandbox_risk_level.as_deref().unwrap_or("safe");
                    let sandbox_score = response.sandbox_score.unwrap_or(0);

                    match sandbox_risk {
                        "dangerous" => {
                            println!(
                                "  {} Sandbox: DANGEROUS (score: {})",
                                "[!]".red(),
                                sandbox_score
                            );
                        }
                        "warning" => {
                            println!(
                                "  {} Sandbox: Warning (score: {})",
                                "[!]".yellow(),
                                sandbox_score
                            );
                        }
                        _ => {
                            print_ok(&format!("Sandbox: Clean (score: {})", sandbox_score));
                        }
                    }

                    // Show cached flags
                    if let Some(flags) = &response.sandbox_flags {
                        for flag in flags {
                            if let Some(detail) = &flag.detail {
                                print_detail(detail);
                            }
                        }
                    }
                } else {
                    // No cache - run local sandbox
                    sandbox::run_local_sandbox(pkg_name, version);
                }
            }

            println!();
            true
        }
        Err(e) => {
            print_warning(&format!("Could not scan {} - API unreachable", package));
            if verbose {
                print_detail(&e);
            }
            true // Don't block on API failure
        }
    }
}
