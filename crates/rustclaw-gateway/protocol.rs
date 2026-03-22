//! Gateway Protocol for RustClaw
//! 
//! Defines the message format for WebSocket communication
//! Compatible with OpenClaw gateway protocol

use serde::{Deserialize, Serialize};
use rustclaw_core::message::Message;

/// Gateway message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMessage {
    /// Message command/type
    pub command: GatewayCommand,
    /// Message ID for request/response correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    /// Additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl GatewayMessage {
    /// Create a new gateway message
    pub fn new(command: GatewayCommand) -> Self {
        Self {
            command,
            id: Some(uuid::Uuid::new_v4().to_string()),
            timestamp: Some(chrono::Utc::now().timestamp_millis()),
            data: None,
        }
    }

    /// Create a ping message
    pub fn ping() -> Self {
        Self::new(GatewayCommand::Ping)
    }

    /// Create a pong message
    pub fn pong() -> Self {
        Self::new(GatewayCommand::Pong)
    }

    /// Create an error message
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(GatewayCommand::Error {
            message: message.into(),
        })
    }

    /// Create a chat response
    pub fn chat_response(content: impl Into<String>) -> Self {
        Self::new(GatewayCommand::ChatResponse {
            message: Message::assistant(content),
        })
    }

    /// Create a stream chunk
    pub fn stream_chunk(content: impl Into<String>, done: bool) -> Self {
        Self::new(GatewayCommand::StreamChunk {
            content: content.into(),
            done,
        })
    }
}

/// Gateway command types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayCommand {
    /// Ping
    Ping,
    /// Pong
    Pong,
    /// Error
    Error {
        message: String,
    },
    /// Chat request
    Chat {
        messages: Vec<Message>,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
    /// Chat response
    ChatResponse {
        message: Message,
    },
    /// Stream request
    Stream {
        messages: Vec<Message>,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
    /// Stream chunk
    StreamChunk {
        content: String,
        done: bool,
    },
    /// Tool invoke
    ToolInvoke {
        name: String,
        arguments: serde_json::Value,
    },
    /// Tool result
    ToolResult {
        tool_call_id: String,
        result: String,
        is_error: bool,
    },
    /// Session create
    SessionCreate {
        #[serde(skip_serializing_if = "Option::is_none")]
        config: Option<serde_json::Value>,
    },
    /// Session created
    SessionCreated {
        session_id: String,
    },
    /// Session list
    SessionList,
    /// Session list response
    SessionListResponse {
        sessions: Vec<SessionInfo>,
    },
    /// Session end
    SessionEnd {
        session_id: String,
    },
    /// Agent create
    AgentCreate {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        config: Option<serde_json::Value>,
    },
    /// Agent created
    AgentCreated {
        agent_id: String,
    },
    /// Agent list
    AgentList,
    /// Agent list response
    AgentListResponse {
        agents: Vec<AgentInfo>,
    },
    /// Config get
    ConfigGet,
    /// Config response
    ConfigResponse {
        config: serde_json::Value,
    },
    /// Config update
    ConfigUpdate {
        config: serde_json::Value,
    },
    /// Presence update
    PresenceUpdate {
        agent_id: String,
        status: String,
    },
    /// Webhook trigger
    WebhookTrigger {
        event: String,
        payload: serde_json::Value,
    },
}

/// Session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub status: String,
    pub created_at: i64,
    pub message_count: usize,
}

/// Agent info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_message_serialization() {
        let msg = GatewayMessage::ping();
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("ping"));
    }

    #[test]
    fn test_gateway_message_deserialization() {
        let json = r#"{"command": {"type": "ping"}, "id": "test-123"}"#;
        let msg: GatewayMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg.command, GatewayCommand::Ping));
    }
}
