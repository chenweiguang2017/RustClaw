//! Core types for RustClaw

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl Default for AgentId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl Default for SessionId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tool identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(pub String);

impl From<&str> for ToolId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// LLM Provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Azure,
    Local,
    Custom(String),
}

impl Default for LlmProvider {
    fn default() -> Self {
        Self::OpenAI
    }
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAI => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
            Self::Azure => write!(f, "azure"),
            Self::Local => write!(f, "local"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// LLM Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: LlmProvider,
    pub model_name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model_name: "gpt-4".to_string(),
            api_key: None,
            base_url: None,
            temperature: 0.7,
            max_tokens: Some(4096),
            top_p: Some(1.0),
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute (RPM)
    pub rpm: Option<u32>,
    /// Minimum interval between requests in milliseconds
    pub min_interval_ms: Option<u64>,
    /// Maximum interval between requests in milliseconds (for random range)
    pub max_interval_ms: Option<u64>,
    /// Whether to use random interval within range
    pub use_random_interval: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            rpm: Some(60), // Default 60 RPM
            min_interval_ms: None,
            max_interval_ms: None,
            use_random_interval: false,
        }
    }
}

impl RateLimitConfig {
    /// Calculate the interval between requests based on RPM
    pub fn interval_from_rpm(&self) -> u64 {
        if let Some(rpm) = self.rpm {
            60000 / rpm as u64 // milliseconds per request
        } else if let Some(min) = self.min_interval_ms {
            if self.use_random_interval {
                if let Some(max) = self.max_interval_ms {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    rng.gen_range(min..=max)
                } else {
                    min
                }
            } else {
                min
            }
        } else {
            1000 // Default 1 second
        }
    }
}

/// Concurrency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Maximum concurrent API requests
    pub max_concurrent_requests: usize,
    /// Maximum concurrent tool executions
    pub max_concurrent_tools: usize,
    /// Maximum concurrent sessions
    pub max_concurrent_sessions: usize,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            max_concurrent_tools: 5,
            max_concurrent_sessions: 100,
        }
    }
}

/// Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub websocket_path: String,
    pub http_path: String,
    pub auth: AuthConfig,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            websocket_path: "/ws".to_string(),
            http_path: "/api".to_string(),
            auth: AuthConfig::default(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub mode: AuthMode,
    pub token: Option<String>,
    pub password: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            mode: AuthMode::None,
            token: None,
            password: None,
        }
    }
}

/// Authentication mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    None,
    Token,
    Password,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub path: Option<String>,
    pub config: HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "1.0.0".to_string(),
            enabled: true,
            path: None,
            config: HashMap::new(),
        }
    }
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub workspace_path: String,
    pub soul_file: String,
    pub agents_file: String,
    pub max_context_tokens: usize,
    pub enable_long_term_memory: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            workspace_path: ".rustclaw".to_string(),
            soul_file: "SOUL.md".to_string(),
            agents_file: "AGENTS.md".to_string(),
            max_context_tokens: 128000,
            enable_long_term_memory: true,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Json,
            file: None,
        }
    }
}

/// Log format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_default() {
        let id = AgentId::default();
        assert!(!id.0.is_empty());
    }

    #[test]
    fn test_rate_limit_config_interval() {
        let config = RateLimitConfig {
            rpm: Some(60),
            ..Default::default()
        };
        assert_eq!(config.interval_from_rpm(), 1000);
    }

    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();
        assert_eq!(config.provider, LlmProvider::OpenAI);
        assert_eq!(config.model_name, "gpt-4");
    }
}
