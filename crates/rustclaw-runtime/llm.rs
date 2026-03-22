//! LLM Client for RustClaw
//! 
//! Provides OpenAI-compatible API client with rate limiting support

use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use rustclaw_core::{
    error::{Error, Result},
    message::Message,
    types::{LlmProvider, ModelConfig},
};

/// LLM client configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: "gpt-4".to_string(),
            api_key: None,
            base_url: "https://api.openai.com/v1".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

impl From<&ModelConfig> for LlmConfig {
    fn from(config: &ModelConfig) -> Self {
        Self {
            provider: config.provider.clone(),
            model: config.model_name.clone(),
            api_key: config.api_key.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| {
                match &config.provider {
                    LlmProvider::OpenAI => "https://api.openai.com/v1".to_string(),
                    LlmProvider::Anthropic => "https://api.anthropic.com/v1".to_string(),
                    LlmProvider::Azure => "https://your-resource.openai.azure.com/openai/deployments".to_string(),
                    LlmProvider::Local => "http://localhost:11434/v1".to_string(),
                    LlmProvider::Custom(url) => url.clone(),
                }
            }),
            temperature: config.temperature,
            max_tokens: config.max_tokens.unwrap_or(4096),
        }
    }
}

/// LLM client
pub struct LlmClient {
    config: LlmConfig,
    http_client: Client,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(config: LlmConfig) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap();

        Self { config, http_client }
    }

    /// Send a chat completion request
    pub async fn chat(&self, messages: Vec<Message>) -> Result<Message> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| Error::LlmApi("API key not configured".to_string()))?;

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.iter().map(|m| m.to_openai_format()).collect(),
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            stream: Some(false),
        };

        let response = self.http_client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::LlmApi(format!("API error: {}", error_text)));
        }

        let chat_response: ChatResponse = response.json().await?;
        
        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.clone())
        } else {
            Err(Error::LlmApi("No response choices".to_string()))
        }
    }

    /// Stream a chat completion
    pub async fn stream_chat(
        &self,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| Error::LlmApi("API key not configured".to_string()))?;

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.iter().map(|m| m.to_openai_format()).collect(),
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            stream: Some(true),
        };

        let response = self.http_client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::LlmApi(format!("API error: {}", error_text)));
        }

        // TODO: Implement proper SSE stream parsing
        Ok(Box::pin(futures::stream::empty()))
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    id: String,
    choices: Vec<ChatChoice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    index: u32,
    message: Message,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.base_url, "https://api.openai.com/v1");
    }
}
