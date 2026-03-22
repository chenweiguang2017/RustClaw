//! Session management for RustClaw

use crate::types::SessionId;
use crate::message::Message;
use crate::agent::{Agent, AgentState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is active
    Active,
    /// Session is paused
    Paused,
    /// Session has ended
    Ended,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity time
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Total message count
    pub message_count: usize,
    /// Total token count (approximate)
    pub token_count: usize,
    /// Custom metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            last_activity: now,
            message_count: 0,
            token_count: 0,
            extra: HashMap::new(),
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum session duration in seconds
    pub max_duration_secs: Option<u64>,
    /// Maximum messages in session
    pub max_messages: Option<usize>,
    /// Auto-save session state
    pub auto_save: bool,
    /// Session timeout in seconds
    pub timeout_secs: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_duration_secs: Some(3600), // 1 hour
            max_messages: Some(1000),
            auto_save: true,
            timeout_secs: 300, // 5 minutes
        }
    }
}

/// Session instance
#[derive(Debug, Clone)]
pub struct Session {
    /// Session ID
    pub id: SessionId,
    /// Session status
    pub status: SessionStatus,
    /// Session configuration
    pub config: SessionConfig,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Session start time
    pub started_at: Instant,
    /// Last activity time
    pub last_activity: Instant,
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        Self {
            id: SessionId::default(),
            status: SessionStatus::Active,
            config: SessionConfig::default(),
            metadata: SessionMetadata::default(),
            messages: Vec::new(),
            started_at: Instant::now(),
            last_activity: Instant::now(),
        }
    }

    /// Create a session with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Create a session with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: SessionId(id.into()),
            ..Self::new()
        }
    }

    /// Add a message to the session
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.metadata.message_count = self.messages.len();
        self.touch();
    }

    /// Update last activity time
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
        self.metadata.last_activity = chrono::Utc::now();
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        let elapsed = self.last_activity.elapsed();
        elapsed > Duration::from_secs(self.config.timeout_secs)
    }

    /// Check if session has reached message limit
    pub fn is_message_limit_reached(&self) -> bool {
        if let Some(max) = self.config.max_messages {
            self.messages.len() >= max
        } else {
            false
        }
    }

    /// Check if session has reached duration limit
    pub fn is_duration_limit_reached(&self) -> bool {
        if let Some(max_secs) = self.config.max_duration_secs {
            self.started_at.elapsed() > Duration::from_secs(max_secs)
        } else {
            false
        }
    }

    /// Pause the session
    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
    }

    /// Resume the session
    pub fn resume(&mut self) {
        self.status = SessionStatus::Active;
        self.touch();
    }

    /// End the session
    pub fn end(&mut self) {
        self.status = SessionStatus::Ended;
    }

    /// Clear session messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.metadata.message_count = 0;
        self.touch();
    }

    /// Get messages for context (limited by token count)
    pub fn get_context_messages(&self, max_tokens: usize) -> Vec<Message> {
        // Simple implementation: return all messages if under limit
        // TODO: Implement proper token counting and truncation
        if self.estimate_tokens() <= max_tokens {
            return self.messages.clone();
        }

        // Return last N messages that fit within token limit
        let mut result = Vec::new();
        let mut token_count = 0;

        for msg in self.messages.iter().rev() {
            let msg_tokens = msg.estimate_tokens();
            if token_count + msg_tokens > max_tokens {
                break;
            }
            token_count += msg_tokens;
            result.push(msg.clone());
        }

        result.reverse();
        result
    }

    /// Estimate total tokens in session
    pub fn estimate_tokens(&self) -> usize {
        self.messages.iter().map(|m| m.estimate_tokens()).sum()
    }

    /// Export session to JSON
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id.to_string(),
            "status": self.status,
            "config": self.config,
            "metadata": self.metadata,
            "messages": self.messages,
        })
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager for handling multiple sessions
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create and register a new session
    pub fn create_session(&mut self) -> SessionId {
        let session = Session::new();
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        id
    }

    /// Create a session with custom configuration
    pub fn create_session_with_config(&mut self, config: SessionConfig) -> SessionId {
        let session = Session::with_config(config);
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        id
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &SessionId) -> Option<&Session> {
        self.sessions.get(id)
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, id: &SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    /// Remove a session
    pub fn remove_session(&mut self, id: &SessionId) -> Option<Session> {
        self.sessions.remove(id)
    }

    /// Get all active sessions
    pub fn active_sessions(&self) -> Vec<&Session> {
        self.sessions
            .values()
            .filter(|s| s.status == SessionStatus::Active)
            .collect()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&mut self) -> Vec<SessionId> {
        let expired: Vec<SessionId> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        for id in &expired {
            self.sessions.remove(id);
        }

        expired
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new();
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_session_messages() {
        let mut session = Session::new();
        session.add_message(Message::user("Hello"));
        session.add_message(Message::assistant("Hi!"));

        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.metadata.message_count, 2);
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();
        let id = manager.create_session();

        assert!(manager.get_session(&id).is_some());
        assert_eq!(manager.session_count(), 1);

        manager.remove_session(&id);
        assert_eq!(manager.session_count(), 0);
    }
}
