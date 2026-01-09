use crate::config;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const TIMEOUT_SECS: u64 = 15;

#[derive(Debug, Deserialize)]
pub struct ScanResponse {
    pub package: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "riskLevel")]
    pub risk_level: Option<String>,
    pub message: Option<String>,
    pub scanned: Option<bool>,
    #[serde(rename = "sandboxCached")]
    pub sandbox_cached: Option<bool>,
    #[serde(rename = "sandboxScore")]
    pub sandbox_score: Option<i32>,
    #[serde(rename = "sandboxRiskLevel")]
    pub sandbox_risk_level: Option<String>,
    #[serde(rename = "sandboxFlags")]
    pub sandbox_flags: Option<Vec<SandboxFlag>>,
    #[serde(rename = "previousVersion")]
    pub previous_version: Option<String>,
    #[serde(rename = "anomalyReasons")]
    pub anomaly_reasons: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SandboxFlag {
    pub flag: Option<String>,
    pub detail: Option<String>,
    pub score: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ProStatusResponse {
    #[serde(rename = "isPro")]
    pub is_pro: bool,
    pub plan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SandboxAnalyzeRequest {
    pub package: String,
    pub version: String,
    pub behavior: SandboxBehavior,
    pub source: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
}

#[derive(Debug, Serialize, Default)]
pub struct SandboxBehavior {
    #[serde(rename = "filesRead")]
    pub files_read: Vec<String>,
    #[serde(rename = "filesWritten")]
    pub files_written: Vec<String>,
    #[serde(rename = "networkConnections")]
    pub network_connections: Vec<NetworkConnection>,
    #[serde(rename = "processesSpawned")]
    pub processes_spawned: Vec<ProcessSpawned>,
    #[serde(rename = "envVarsAccessed")]
    pub env_vars_accessed: Vec<String>,
    #[serde(rename = "sensitiveAccess")]
    pub sensitive_access: Vec<SensitiveAccess>,
    #[serde(rename = "exitCode")]
    pub exit_code: i32,
}

#[derive(Debug, Serialize)]
pub struct NetworkConnection {
    pub host: Option<String>,
    pub ip: Option<String>,
    pub port: i32,
}

#[derive(Debug, Serialize)]
pub struct ProcessSpawned {
    pub executable: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SensitiveAccess {
    #[serde(rename = "type")]
    pub access_type: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct SandboxAnalyzeResponse {
    pub score: Option<i32>,
    #[serde(rename = "riskLevel")]
    pub risk_level: Option<String>,
    pub flags: Option<Vec<SandboxFlag>>,
    pub cached: Option<bool>,
}

/// Create HTTP client
fn create_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()
}

/// Build URL with auth params
fn build_url(endpoint: &str) -> String {
    let api_url = config::get_api_url();
    let device_id = config::get_device_id();
    let mut url = format!("{}/{}?device={}", api_url, endpoint, device_id);
    
    if let Some(api_key) = config::get_api_key() {
        url.push_str(&format!("&key={}", api_key));
    }
    
    url
}

/// Scan a package
pub fn scan_package(package: &str) -> Result<ScanResponse, String> {
    let client = create_client().map_err(|e| e.to_string())?;
    let url = build_url(&format!("scan?package={}", package));
    
    client
        .get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())
}

/// Check Pro status
pub fn check_pro_status() -> Result<ProStatusResponse, String> {
    let client = create_client().map_err(|e| e.to_string())?;
    let url = build_url("pro/status");
    
    client
        .get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())
}

/// Send sandbox behavior for analysis
pub fn analyze_sandbox(request: &SandboxAnalyzeRequest) -> Result<SandboxAnalyzeResponse, String> {
    let client = create_client().map_err(|e| e.to_string())?;
    let api_url = config::get_api_url();
    let mut url = format!("{}/sandbox/analyze", api_url);
    
    if let Some(api_key) = config::get_api_key() {
        url.push_str(&format!("?key={}", api_key));
    }
    
    client
        .post(&url)
        .json(request)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())
}

/// Download a Pro module (e.g., monitor.js)
pub fn download_pro_module(module_name: &str) -> Result<String, String> {
    let client = create_client().map_err(|e| e.to_string())?;
    let url = build_url(&format!("pro/module/{}", module_name));
    
    let response = client
        .get(&url)
        .send()
        .map_err(|e| e.to_string())?;
    
    if response.status() == 403 {
        return Err("Pro subscription required".to_string());
    }
    
    if response.status() == 404 {
        return Err("Module not found".to_string());
    }
    
    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()));
    }
    
    response.text().map_err(|e| e.to_string())
}
