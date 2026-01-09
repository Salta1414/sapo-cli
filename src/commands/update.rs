use crate::utils::*;

pub fn run() {
    println!();
    print_info("Updating Sapo...");

    #[cfg(target_os = "windows")]
    {
        let result = std::process::Command::new("powershell")
            .args([
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "irm https://sapo.salta.world/install.ps1 | iex",
            ])
            .status();

        match result {
            Ok(status) if status.success() => {
                print_ok("Update complete! Restart your terminal.");
            }
            _ => {
                print_error("Update failed");
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let result = std::process::Command::new("bash")
            .args([
                "-c",
                "curl -fsSL https://sapo.salta.world/install.sh | bash",
            ])
            .status();

        match result {
            Ok(status) if status.success() => {
                print_ok("Update complete! Restart your terminal.");
            }
            _ => {
                print_error("Update failed");
            }
        }
    }
}
