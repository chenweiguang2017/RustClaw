//! Agent types for RustClaw

use crate::types::{AgentId, SessionId};
use crate::message::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    /// Agent is idle
    Idle,
    /// Agent is processing a request
    Processing,
    /// Agent is waiting for tool execution
    WaitingForTool,
    /// Agent is in error state
    Error,
    /// Agent is stopped
    Stopped,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Processing => write!(f, "processing"),
            Self::WaitingForTool => write!(f, "waiting_for_tool"),
            Self::Error => write!(f, "error"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,
    /// Agent description
    pub description: Option<String>,
    /// System prompt
    pub system_prompt: Option<String>,
    /// Model to use
    pub model: Option<String>,
    /// Maximum turns in conversation
    pub max_turns: Option<u32>,
    /// Enable tool use
    pub enable_tools: bool,
    /// Enable memory
    pub enable_memory: bool,
    /// Custom configuration
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            system_prompt: None,
            model: None,
            max_turns: Some(100),
            enable_tools: true,
            enable_memory: true,
            extra: HashMap::new(),
        }
    }
}

/// Agent instance
#[derive(Debug, Clone)]
pub struct Agent {
    /// Agent ID
    pub id: AgentId,
    /// Agent configuration
    pub config: AgentConfig,
    /// Current state
    pub state: AgentState,
    /// Current session ID
    pub current_session: Option<SessionId>,
    /// Conversation history
    pub history: Vec<Message>,
    /// Turn count
    pub turn_count: u32,
}

impl Agent {
    /// Create a new agent with default configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: AgentId::default(),
            config: AgentConfig {
                name: name.into(),
                ..Default::default()
            },
            state: AgentState::Idle,
            current_session: None,
            history: Vec::new(),
            turn_count: 0,
        }
    }

    /// Create an agent with custom configuration
    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            id: AgentId::default(),
            config,
            state: AgentState::Idle,
            current_session: None,
            history: Vec::new(),
            turn_count: 0,
        }
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// Add a message to history
    pub fn add_message(&mut self, message: Message) {
        self.history.push(message);
        if message.role == crate::message::MessageRole::User {
            self.turn_count += 1;
        }
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.turn_count = 0;
    }

    /// Set the agent state
    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
    }

    /// Start a new session
    pub fn start_session(&mut self, session_id: SessionId) {
        self.current_session = Some(session_id);
        self.state = AgentState::Idle;
    }

    /// End the current session
    pub fn end_session(&mut self) {
        self.current_session = None;
        self.state = AgentState::Idle;
    }

    /// Check if agent can accept more messages
    pub fn can_accept_message(&self) -> bool {
        if let Some(max_turns) = self.config.max_turns {
            self.turn_count < max_turns
        } else {
            true
        }
    }

    /// Get the system message if configured
    pub fn system_message(&self) -> Option<Message> {
        self.config.system_prompt.as_ref().map(|prompt| {
            Message::system(prompt.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("test-agent");
        assert_eq!(agent.config.name, "test-agent");
        assert_eq!(agent.state, AgentState::Idle);
    }

    #[test]
    fn test_agent_state_transitions() {
        let mut agent = Agent::new("test");
        assert_eq!(agent.state, AgentState::Idle);
        
        agent.set_state(AgentState::Processing);
        assert_eq!(agent.state, AgentState::Processing);
    }

    #[test]
    fn test_agent_history() {
        let mut agent = Agent::new("test");
        agent.add_message(Message::user("Hello"));
        agent.add_message(Message::assistant("Hi there!"));
        
        assert_eq!(agent.history.len(), 2);
        assert_eq!(agent.turn_count, 1);
    }
}
