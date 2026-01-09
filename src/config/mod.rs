mod store;
#[cfg(test)]
mod tests;

pub use store::*;

use std::path::PathBuf;

/// Get the Sapo config directory (~/.sapo)
pub fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".sapo")
}

/// Get the config file path (~/.sapo/config)
pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config")
}

/// Get the bin directory (~/.sapo/bin)
pub fn get_bin_dir() -> PathBuf {
    get_config_dir().join("bin")
}

/// Initialize config on first run
pub fn init() {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).ok();
    }

    let config_path = get_config_path();
    if !config_path.exists() {
        let device_id = generate_device_id();
        create_default_config(&device_id);
    }
}

/// Generate a unique device ID
pub fn generate_device_id() -> String {
    let uuid = uuid::Uuid::new_v4();

    #[cfg(target_os = "windows")]
    let prefix = "win";

    #[cfg(target_os = "macos")]
    let prefix = "mac";

    #[cfg(target_os = "linux")]
    let prefix = "linux";

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let prefix = "unknown";

    format!("{}_{}", prefix, uuid)
}
