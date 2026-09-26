use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub mode: String,
    pub log_level: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mode: "local".to_string(),
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub default: Option<String>,
    pub brave_api_key: Option<String>,
    pub tavily_api_key: Option<String>,
    pub github_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LadderConfig {
    pub cache_enabled: bool,
    pub cache_ttl_hours: u32,
    pub enable_browser_fallback: bool,
    pub max_browser_sessions: u32,
}

impl Default for LadderConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_ttl_hours: 48,
            enable_browser_fallback: true,
            max_browser_sessions: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfig {
    pub max_requests: usize,
    pub max_runtime_sec: u64,
    pub max_depth: usize,
    pub block_private_networks: bool,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            max_requests: 25,
            max_runtime_sec: 120,
            max_depth: 3,
            block_private_networks: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenIssaConfig {
    pub openissa: GeneralConfig,
    pub providers: ProvidersConfig,
    pub ladder: LadderConfig,
    pub research: ResearchConfig,
}

impl OpenIssaConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a simple INI/TOML configuration file natively
    pub fn from_toml_str(content: &str) -> Self {
        let mut config = Self::default();
        let mut current_section = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].trim().to_lowercase();
                continue;
            }

            if let Some((raw_key, raw_val)) = trimmed.split_once('=') {
                let key = raw_key.trim().to_lowercase();
                let val_str = raw_val.split('#').next().unwrap_or("").trim();
                let clean_val = val_str.trim_matches('"').trim_matches('\'').trim();

                match current_section.as_str() {
                    "openissa" => match key.as_str() {
                        "mode" => config.openissa.mode = clean_val.to_string(),
                        "log_level" => config.openissa.log_level = clean_val.to_string(),
                        _ => {}
                    },
                    "providers" => match key.as_str() {
                        "default" => config.providers.default = Some(clean_val.to_string()),
                        "brave_api_key" if !clean_val.is_empty() => {
                            config.providers.brave_api_key = Some(clean_val.to_string());
                        }
                        "tavily_api_key" if !clean_val.is_empty() => {
                            config.providers.tavily_api_key = Some(clean_val.to_string());
                        }
                        "github_token" if !clean_val.is_empty() => {
                            config.providers.github_token = Some(clean_val.to_string());
                        }
                        _ => {}
                    },
                    "ladder" => match key.as_str() {
                        "cache_enabled" => {
                            if let Ok(b) = clean_val.parse::<bool>() {
                                config.ladder.cache_enabled = b;
                            }
                        }
                        "cache_ttl_hours" => {
                            if let Ok(n) = clean_val.parse::<u32>() {
                                config.ladder.cache_ttl_hours = n;
                            }
                        }
                        "enable_browser_fallback" => {
                            if let Ok(b) = clean_val.parse::<bool>() {
                                config.ladder.enable_browser_fallback = b;
                            }
                        }
                        "max_browser_sessions" => {
                            if let Ok(n) = clean_val.parse::<u32>() {
                                config.ladder.max_browser_sessions = n;
                            }
                        }
                        _ => {}
                    },
                    "research" => match key.as_str() {
                        "max_requests" => {
                            if let Ok(n) = clean_val.parse::<usize>() {
                                config.research.max_requests = n;
                            }
                        }
                        "max_runtime_sec" => {
                            if let Ok(n) = clean_val.parse::<u64>() {
                                config.research.max_runtime_sec = n;
                            }
                        }
                        "max_depth" => {
                            if let Ok(n) = clean_val.parse::<usize>() {
                                config.research.max_depth = n;
                            }
                        }
                        "block_private_networks" => {
                            if let Ok(b) = clean_val.parse::<bool>() {
                                config.research.block_private_networks = b;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        config
    }

    /// Default configuration file path (~/.openissa/config.toml)
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".openissa").join("config.toml")
    }

    /// Load configuration from standard locations (OPENISSA_CONFIG, ~/.openissa/config.toml, ./config.toml)
    pub fn load() -> Self {
        if let Ok(env_path) = std::env::var("OPENISSA_CONFIG") {
            let p = Path::new(&env_path);
            if p.exists() {
                if let Ok(content) = std::fs::read_to_string(p) {
                    return Self::from_toml_str(&content);
                }
            }
        }

        let def_path = Self::default_path();
        if def_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&def_path) {
                return Self::from_toml_str(&content);
            }
        }

        let local_path = Path::new("config.toml");
        if local_path.exists() {
            if let Ok(content) = std::fs::read_to_string(local_path) {
                return Self::from_toml_str(&content);
            }
        }

        Self::default()
    }

    /// Apply configured API keys into process environment if not already set by user
    pub fn apply_to_env(&self) {
        if let Some(ref key) = self.providers.brave_api_key {
            if std::env::var("BRAVE_API_KEY").is_err() {
                std::env::set_var("BRAVE_API_KEY", key);
            }
        }
        if let Some(ref key) = self.providers.tavily_api_key {
            if std::env::var("TAVILY_API_KEY").is_err() {
                std::env::set_var("TAVILY_API_KEY", key);
            }
        }
        if let Some(ref token) = self.providers.github_token {
            if std::env::var("GITHUB_TOKEN").is_err() && std::env::var("GH_TOKEN").is_err() {
                std::env::set_var("GITHUB_TOKEN", token);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_toml_config() {
        let sample = r#"
            [openissa]
            mode = "local"
            log_level = "debug"

            [providers]
            brave_api_key = "test_brave_key"
            tavily_api_key = "test_tavily_key"
            github_token = "ghp_12345"

            [ladder]
            cache_enabled = true
            cache_ttl_hours = 72
            enable_browser_fallback = true

            [research]
            max_requests = 10
            max_runtime_sec = 60
            block_private_networks = true
        "#;

        let config = OpenIssaConfig::from_toml_str(sample);
        assert_eq!(config.openissa.log_level, "debug");
        assert_eq!(
            config.providers.brave_api_key.as_deref(),
            Some("test_brave_key")
        );
        assert_eq!(
            config.providers.tavily_api_key.as_deref(),
            Some("test_tavily_key")
        );
        assert_eq!(config.providers.github_token.as_deref(), Some("ghp_12345"));
        assert_eq!(config.ladder.cache_ttl_hours, 72);
        assert_eq!(config.research.max_requests, 10);
        assert!(config.research.block_private_networks);
    }

    #[test]
    fn test_empty_config_defaults() {
        let config = OpenIssaConfig::from_toml_str("");
        assert_eq!(config.openissa.mode, "local");
        assert_eq!(config.ladder.cache_ttl_hours, 48);
        assert_eq!(config.research.max_requests, 25);
    }
}
