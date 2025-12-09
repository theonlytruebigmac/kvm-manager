//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub log_level: String,
    pub log_file: Option<String>,
    pub channel_name: String,
    pub max_message_size_kb: usize,
    pub connection_retry_seconds: u64,
    pub request_timeout_seconds: u64,
    pub max_concurrent_requests: usize,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allowed_read_paths: Vec<String>,
    pub allowed_write_paths: Vec<String>,
    pub command_whitelist: Option<Vec<String>>,
    pub max_file_size_mb: usize,
    pub max_command_timeout_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            log_level: "info".to_string(),
            log_file: Some("/var/log/kvmmanager-agent.log".to_string()),
            channel_name: "org.kvmmanager.agent.0".to_string(),
            max_message_size_kb: 64,
            connection_retry_seconds: 5,
            request_timeout_seconds: 30,
            max_concurrent_requests: 10,
            security: SecurityConfig::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_read_paths: vec![
                "/tmp".to_string(),
                "/var/log".to_string(),
                "/etc/hostname".to_string(),
                "/etc/os-release".to_string(),
            ],
            allowed_write_paths: vec![
                "/tmp/kvmmanager".to_string(),
            ],
            command_whitelist: None, // None = all commands allowed
            max_file_size_mb: 10,
            max_command_timeout_seconds: 300,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Check if a path is allowed for reading
    pub fn is_read_allowed(&self, path: &str) -> bool {
        self.security.allowed_read_paths.iter().any(|allowed| {
            path.starts_with(allowed)
        })
    }

    /// Check if a path is allowed for writing
    pub fn is_write_allowed(&self, path: &str) -> bool {
        self.security.allowed_write_paths.iter().any(|allowed| {
            path.starts_with(allowed)
        })
    }

    /// Check if a command is allowed to execute
    pub fn is_command_allowed(&self, command: &str) -> bool {
        match &self.security.command_whitelist {
            None => true, // No whitelist = all commands allowed
            Some(whitelist) => whitelist.contains(&command.to_string()),
        }
    }
}
