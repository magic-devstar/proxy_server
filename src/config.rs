use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub api: Vec<APIConfig>,
    pub upstream: Vec<UpstreamProvider>,
    
    #[serde(default)]
    pub clickhouse: ClickhouseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub logging: String,
    
    #[serde(rename = "sni-check")]
    pub sni_check: bool,
    
    pub inflation: i32,
    
    #[serde(rename = "update-interval")]
    pub update_interval: u32,
    
    #[serde(rename = "node-name")]
    pub node_name: String,
    
    #[serde(default, rename = "source-ips")]
    pub source_ips: Vec<String>,
    
    #[serde(default)]
    pub retries: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_retries", rename = "max-retries")]
    pub max_retries: u32,
    
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout: 5,
        }
    }
}

fn default_max_retries() -> u32 { 3 }
fn default_timeout() -> u32 { 5 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConfig {
    pub name: String,
    
    #[serde(rename = "base-url")]
    pub base_url: String,
    
    #[serde(rename = "api-key")]
    pub api_key: String,
    
    #[serde(rename = "default-package")]
    pub default_package: String,
    
    #[serde(default)]
    pub legacy: bool,
    
    pub ports: ServerPorts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPorts {
    pub userpass: String,
    
    #[serde(default)]
    pub ipauth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClickhouseConfig {
    pub host: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamProvider {
    pub name: String,
    pub ips: Vec<String>,
    pub user: String,
    pub password: String,
    pub weight: u32,
    
    #[serde(rename = "format-in")]
    pub format_in: String,
    
    pub package: String,
    
    #[serde(default)]
    pub mapping: HashMap<String, String>,
    
    #[serde(default)]
    pub separator: String,
    
    #[serde(default, rename = "allowed-countries")]
    pub allowed_countries: Vec<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}

