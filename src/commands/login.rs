use crate::config;
use crate::utils::*;

pub fn run() {
    let device_id = config::get_device_id();
    let url = format!("https://sapo.salta.world/login?device={}", device_id);

    print_info("Opening browser for login...");

    // Open URL in default browser
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", &url])
            .spawn()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&url).spawn().ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .ok();
    }

    println!(
        "  {} After login, run 'sapo sync' to activate Pro features",
        "|-".bright_black()
    );
}

use colored::Colorize;
