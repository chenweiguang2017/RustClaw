//! Runtime Engine for RustClaw
//! 
//! The core agent execution engine with RPM control and concurrency limits

use std::sync::Arc;
use tokio::sync::RwLock;

use rustclaw_core::{
    config::RustClawConfig,
    error::{Error, Result},
    rate_limiter::RateLimiter,
    concurrency::ConcurrencyController,
    message::Message,
    agent::{Agent, AgentState},
    tool::{Tool, ToolCall, ToolResult},
    session::{Session, SessionId, SessionManager},
};

use crate::llm::{LlmClient, LlmConfig};
use crate::executor::ToolExecutor;

/// Runtime engine for agent execution
pub struct RuntimeEngine {
    /// Configuration
    config: Arc<RwLock<RustClawConfig>>,
    /// LLM client
    llm_client: Arc<LlmClient>,
    /// Rate limiter
    rate_limiter: Arc<RateLimiter>,
    /// Concurrency controller
    concurrency: Arc<ConcurrencyController>,
    /// Tool executor
    tool_executor: Arc<ToolExecutor>,
    /// Session manager
    session_manager: Arc<RwLock<SessionManager>>,
}

impl RuntimeEngine {
    /// Create a new runtime engine
    pub fn new(config: RustClawConfig) -> Self {
        let llm_config = LlmConfig::from(&config.model);
        let llm_client = Arc::new(LlmClient::new(llm_config));
        
        let rate_limiter = Arc::new(RateLimiter::new(
            config.rate_limit.clone(),
            config.concurrency.max_concurrent_requests,
        ));
        
        let concurrency = Arc::new(ConcurrencyController::new(config.concurrency.clone()));
        let tool_executor = Arc::new(ToolExecutor::new());
        let session_manager = Arc::new(RwLock::new(SessionManager::new()));
        
        Self {
            config: Arc::new(RwLock::new(config)),
            llm_client,
            rate_limiter,
            concurrency,
            tool_executor,
            session_manager,
        }
    }

    /// Create a new session
    pub async fn create_session(&self) -> SessionId {
        let mut manager = self.session_manager.write().await;
        manager.create_session()
    }

    /// Get a session by ID
    pub async fn get_session(&self, id: &SessionId) -> Option<Session> {
        let manager = self.session_manager.read().await;
        manager.get_session(id).cloned()
    }

    /// Chat with the agent
    pub async fn chat(&self, session_id: &SessionId, message: Message) -> Result<Message> {
        // Acquire rate limit permit
        let _permit = self.rate_limiter.acquire().await?;
        
        // Acquire concurrency permit
        let _concurrency = self.concurrency.acquire_request().await?;
        
        // Get session
        let mut manager = self.session_manager.write().await;
        let session = manager.get_session_mut(session_id)
            .ok_or_else(|| Error::Session("Session not found".to_string()))?;
        
        // Add user message
        session.add_message(message.clone());
        
        // Build messages for LLM
        let messages = session.messages.clone();
        
        // Call LLM
        let response = self.llm_client.chat(messages).await?;
        
        // Add assistant message
        session.add_message(response.clone());
        
        Ok(response)
    }

    /// Stream chat with the agent
    pub async fn stream_chat(
        &self,
        session_id: &SessionId,
        message: Message,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        // Acquire rate limit permit
        let _permit = self.rate_limiter.acquire().await?;
        
        // Get session
        let mut manager = self.session_manager.write().await;
        let session = manager.get_session_mut(session_id)
            .ok_or_else(|| Error::Session("Session not found".to_string()))?;
        
        // Add user message
        session.add_message(message.clone());
        
        // Build messages for LLM
        let messages = session.messages.clone();
        
        // Stream from LLM
        let stream = self.llm_client.stream_chat(messages).await?;
        
        Ok(stream)
    }

    /// Execute a tool
    pub async fn execute_tool(&self, tool_call: ToolCall) -> Result<ToolResult> {
        // Acquire tool concurrency permit
        let _permit = self.concurrency.acquire_tool().await?;
        
        self.tool_executor.execute(tool_call).await
    }

    /// Register a tool
    pub fn register_tool(&self, tool: Tool) {
        self.tool_executor.register(tool);
    }

    /// Update rate limit configuration
    pub async fn update_rate_limit(&self, rpm: Option<u32>, min_interval_ms: Option<u64>, max_interval_ms: Option<u64>) {
        let mut config = self.config.write().await;
        config.rate_limit.rpm = rpm;
        config.rate_limit.min_interval_ms = min_interval_ms;
        config.rate_limit.max_interval_ms = max_interval_ms;
        
        // Update rate limiter
        // Note: In production, we'd want to update the existing rate limiter
    }

    /// Update max concurrent requests
    pub async fn update_max_concurrent(&self, max: usize) {
        let mut config = self.config.write().await;
        config.concurrency.max_concurrent_requests = max;
    }

    /// Get current RPM
    pub fn current_rpm(&self) -> u32 {
        self.rate_limiter.current_rpm()
    }

    /// Get available request slots
    pub fn available_slots(&self) -> usize {
        self.rate_limiter.available_slots()
    }

    /// Get statistics
    pub async fn stats(&self) -> RuntimeStats {
        let manager = self.session_manager.read().await;
        
        RuntimeStats {
            current_rpm: self.rate_limiter.current_rpm(),
            available_requests: self.rate_limiter.available_slots(),
            active_requests: self.concurrency.active_requests(),
            active_tools: self.concurrency.active_tools(),
            active_sessions: manager.session_count(),
        }
    }
}

/// Runtime statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct RuntimeStats {
    pub current_rpm: u32,
    pub available_requests: usize,
    pub active_requests: usize,
    pub active_tools: usize,
    pub active_sessions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_engine_creation() {
        let config = RustClawConfig::default();
        let engine = RuntimeEngine::new(config);
        
        let session_id = engine.create_session().await;
        assert!(!session_id.0.is_empty());
    }
}
