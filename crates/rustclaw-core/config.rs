//! Configuration management for RustClaw

use crate::error::Result;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration for RustClaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustClawConfig {
    /// Model configuration
    pub model: ModelConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Concurrency configuration
    pub concurrency: ConcurrencyConfig,
    /// Gateway configuration
    pub gateway: GatewayConfig,
    /// Memory configuration
    pub memory: MemoryConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Plugins configuration
    pub plugins: Vec<PluginConfig>,
}

impl Default for RustClawConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            rate_limit: RateLimitConfig::default(),
            concurrency: ConcurrencyConfig::default(),
            gateway: GatewayConfig::default(),
            memory: MemoryConfig::default(),
            logging: LoggingConfig::default(),
            plugins: Vec::new(),
        }
    }
}

impl RustClawConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: RustClawConfig = if path.as_ref().extension().map_or(false, |e| e == "json") {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };
        Ok(config)
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Model configuration
        if let Ok(api_key) = std::env::var("RUSTCLAW_API_KEY") {
            config.model.api_key = Some(api_key);
        }
        if let Ok(base_url) = std::env::var("RUSTCLAW_BASE_URL") {
            config.model.base_url = Some(base_url);
        }
        if let Ok(model) = std::env::var("RUSTCLAW_MODEL") {
            config.model.model_name = model;
        }
        if let Ok(provider) = std::env::var("RUSTCLAW_PROVIDER") {
            config.model.provider = match provider.to_lowercase().as_str() {
                "openai" => LlmProvider::OpenAI,
                "anthropic" => LlmProvider::Anthropic,
                "azure" => LlmProvider::Azure,
                "local" => LlmProvider::Local,
                _ => LlmProvider::Custom(provider),
            };
        }

        // Rate limiting configuration
        if let Ok(rpm) = std::env::var("RUSTCLAW_RPM") {
            if let Ok(rpm) = rpm.parse() {
                config.rate_limit.rpm = Some(rpm);
            }
        }
        if let Ok(min_interval) = std::env::var("RUSTCLAW_MIN_INTERVAL_MS") {
            if let Ok(min_interval) = min_interval.parse() {
                config.rate_limit.min_interval_ms = Some(min_interval);
            }
        }
        if let Ok(max_interval) = std::env::var("RUSTCLAW_MAX_INTERVAL_MS") {
            if let Ok(max_interval) = max_interval.parse() {
                config.rate_limit.max_interval_ms = Some(max_interval);
            }
        }
        if let Ok(random) = std::env::var("RUSTCLAW_RANDOM_INTERVAL") {
            config.rate_limit.use_random_interval = random.to_lowercase() == "true";
        }

        // Concurrency configuration
        if let Ok(max_req) = std::env::var("RUSTCLAW_MAX_CONCURRENT_REQUESTS") {
            if let Ok(max_req) = max_req.parse() {
                config.concurrency.max_concurrent_requests = max_req;
            }
        }
        if let Ok(max_tools) = std::env::var("RUSTCLAW_MAX_CONCURRENT_TOOLS") {
            if let Ok(max_tools) = max_tools.parse() {
                config.concurrency.max_concurrent_tools = max_tools;
            }
        }
        if let Ok(max_sessions) = std::env::var("RUSTCLAW_MAX_CONCURRENT_SESSIONS") {
            if let Ok(max_sessions) = max_sessions.parse() {
                config.concurrency.max_concurrent_sessions = max_sessions;
            }
        }

        // Gateway configuration
        if let Ok(host) = std::env::var("RUSTCLAW_HOST") {
            config.gateway.host = host;
        }
        if let Ok(port) = std::env::var("RUSTCLAW_PORT") {
            if let Ok(port) = port.parse() {
                config.gateway.port = port;
            }
        }

        // Auth configuration
        if let Ok(token) = std::env::var("RUSTCLAW_GATEWAY_TOKEN") {
            config.gateway.auth.token = Some(token);
            config.gateway.auth.mode = AuthMode::Token;
        }
        if let Ok(password) = std::env::var("RUSTCLAW_GATEWAY_PASSWORD") {
            config.gateway.auth.password = Some(password);
            config.gateway.auth.mode = AuthMode::Password;
        }

        // Memory configuration
        if let Ok(workspace) = std::env::var("RUSTCLAW_WORKSPACE") {
            config.memory.workspace_path = workspace;
        }

        // Logging configuration
        if let Ok(level) = std::env::var("RUSTCLAW_LOG_LEVEL") {
            config.logging.level = level;
        }

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = if path.as_ref().extension().map_or(false, |e| e == "json") {
            serde_json::to_string_pretty(self)?
        } else {
            serde_yaml::to_string(self)?
        };
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default config file path
    pub fn default_config_path() -> PathBuf {
        PathBuf::from(".rustclaw/config.yaml")
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate rate limit configuration
        if let (Some(min), Some(max)) = 
            (self.rate_limit.min_interval_ms, self.rate_limit.max_interval_ms) {
            if min > max {
                return Err(crate::error::Error::Config(
                    "min_interval_ms cannot be greater than max_interval_ms".to_string()
                ));
            }
        }

        // Validate concurrency configuration
        if self.concurrency.max_concurrent_requests == 0 {
            return Err(crate::error::Error::Config(
                "max_concurrent_requests must be greater than 0".to_string()
            ));
        }

        // Validate gateway port
        if self.gateway.port == 0 {
            return Err(crate::error::Error::Config(
                "gateway port must be greater than 0".to_string()
            ));
        }

        Ok(())
    }
}

/// Configuration builder
pub struct ConfigBuilder {
    config: RustClawConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: RustClawConfig::default(),
        }
    }

    pub fn model(mut self, model: ModelConfig) -> Self {
        self.config.model = model;
        self
    }

    pub fn rate_limit(mut self, rate_limit: RateLimitConfig) -> Self {
        self.config.rate_limit = rate_limit;
        self
    }

    pub fn concurrency(mut self, concurrency: ConcurrencyConfig) -> Self {
        self.config.concurrency = concurrency;
        self
    }

    pub fn gateway(mut self, gateway: GatewayConfig) -> Self {
        self.config.gateway = gateway;
        self
    }

    pub fn memory(mut self, memory: MemoryConfig) -> Self {
        self.config.memory = memory;
        self
    }

    pub fn api_key(mut self, api_key: String) -> Self {
        self.config.model.api_key = Some(api_key);
        self
    }

    pub fn rpm(mut self, rpm: u32) -> Self {
        self.config.rate_limit.rpm = Some(rpm);
        self
    }

    pub fn max_concurrent_requests(mut self, max: usize) -> Self {
        self.config.concurrency.max_concurrent_requests = max;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.gateway.port = port;
        self
    }

    pub fn build(self) -> Result<RustClawConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RustClawConfig::default();
        assert_eq!(config.gateway.port, 3000);
        assert_eq!(config.rate_limit.rpm, Some(60));
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .rpm(100)
            .max_concurrent_requests(20)
            .port(8080)
            .build()
            .unwrap();
        
        assert_eq!(config.rate_limit.rpm, Some(100));
        assert_eq!(config.concurrency.max_concurrent_requests, 20);
        assert_eq!(config.gateway.port, 8080);
    }
}
