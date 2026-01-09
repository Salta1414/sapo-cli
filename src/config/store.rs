use std::collections::HashMap;
use std::fs;

const DEFAULT_API_URL: &str = "https://admired-chickadee-733.convex.site";

/// Read all config values as a HashMap
pub fn get_config() -> HashMap<String, String> {
    let config_path = super::get_config_path();
    let mut config = HashMap::new();

    if let Ok(content) = fs::read_to_string(&config_path) {
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                config.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
    }

    config
}

/// Get a specific config value
pub fn get_config_value(key: &str) -> Option<String> {
    get_config().get(key).cloned()
}

/// Get a config value with default
pub fn get_config_value_or(key: &str, default: &str) -> String {
    get_config_value(key).unwrap_or_else(|| default.to_string())
}

/// Set a config value
pub fn set_config_value(key: &str, value: &str) {
    let config_path = super::get_config_path();
    let mut config = get_config();
    config.insert(key.to_string(), value.to_string());

    let content: String = config
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&config_path, content).ok();
}

/// Create default config file
pub fn create_default_config(device_id: &str) {
    let config_path = super::get_config_path();
    let content = format!(
        "device_id={}\napi_key=\nplan=free\napi_url={}\ntrusted=",
        device_id, DEFAULT_API_URL
    );
    fs::write(&config_path, content).ok();
}

/// Get the API URL
pub fn get_api_url() -> String {
    get_config_value_or("api_url", DEFAULT_API_URL)
}

/// Get the device ID
pub fn get_device_id() -> String {
    get_config_value("device_id").unwrap_or_else(|| {
        let id = super::generate_device_id();
        set_config_value("device_id", &id);
        id
    })
}

/// Get the API key (if set)
pub fn get_api_key() -> Option<String> {
    get_config_value("api_key").filter(|s| !s.is_empty())
}

/// Get the plan (free/pro/enterprise)
pub fn get_plan() -> String {
    get_config_value_or("plan", "free")
}

/// Check if user is Pro
pub fn is_pro() -> bool {
    let plan = get_plan();
    plan == "pro" || plan == "enterprise"
}

/// Get trusted packages as a Vec
pub fn get_trusted() -> Vec<String> {
    get_config_value("trusted")
        .map(|s| {
            s.split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Check if a package is trusted
pub fn is_trusted(package: &str) -> bool {
    get_trusted().contains(&package.to_string())
}

/// Add a package to trusted list
pub fn add_trusted(package: &str) {
    let mut trusted = get_trusted();
    if !trusted.contains(&package.to_string()) {
        trusted.push(package.to_string());
        set_config_value("trusted", &trusted.join(","));
    }
}

/// Remove a package from trusted list
pub fn remove_trusted(package: &str) {
    let trusted: Vec<String> = get_trusted().into_iter().filter(|p| p != package).collect();
    set_config_value("trusted", &trusted.join(","));
}
