//! Gateway Server for RustClaw
//! 
//! Provides HTTP and WebSocket endpoints compatible with OpenClaw

use axum::{
    extract::{ws::WebSocketUpgrade, State, Query, Path},
    http::{StatusCode, header},
    response::{IntoResponse, Response, Json},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tokio::sync::broadcast;

use rustclaw_core::{
    config::RustClawConfig,
    error::Result,
    types::*,
    message::Message,
    session::SessionId,
};

use crate::websocket::WebSocketHandler;
use crate::protocol::{GatewayMessage, GatewayCommand};

/// Gateway server state
#[derive(Debug)]
pub struct GatewayState {
    pub config: RustClawConfig,
    pub message_tx: broadcast::Sender<GatewayMessage>,
}

/// Gateway server
pub struct GatewayServer {
    config: RustClawConfig,
    state: Arc<GatewayState>,
}

impl GatewayServer {
    /// Create a new gateway server
    pub fn new(config: RustClawConfig) -> Self {
        let (message_tx, _) = broadcast::channel(1000);
        let state = Arc::new(GatewayState {
            config: config.clone(),
            message_tx,
        });

        Self { config, state }
    }

    /// Build the router
    pub fn build_router(&self) -> Router {
        Router::new()
            // WebSocket endpoint
            .route("/ws", get(ws_handler))
            // HTTP API endpoints (OpenClaw compatible)
            .route("/api/v1/chat", post(chat_handler))
            .route("/api/v1/chat/stream", post(chat_stream_handler))
            .route("/api/v1/tools", get(tools_list_handler))
            .route("/api/v1/tools/invoke", post(tools_invoke_handler))
            .route("/api/v1/sessions", get(sessions_list_handler))
            .route("/api/v1/sessions/:id", get(session_get_handler))
            .route("/api/v1/sessions/:id/messages", post(session_message_handler))
            .route("/api/v1/agents", get(agents_list_handler))
            .route("/api/v1/agents/:id", get(agent_get_handler))
            .route("/api/v1/health", get(health_handler))
            .route("/api/v1/config", get(config_handler))
            .route("/api/v1/config", put(config_update_handler))
            // Add CORS and tracing layers
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(self.state.clone())
    }

    /// Start the server
    pub async fn serve(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.gateway.host, self.config.gateway.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        
        tracing::info!("RustClaw Gateway listening on {}", addr);
        
        let router = self.build_router();
        axum::serve(listener, router).await?;
        
        Ok(())
    }

    /// Get the state
    pub fn state(&self) -> Arc<GatewayState> {
        self.state.clone()
    }
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<GatewayState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        let handler = WebSocketHandler::new(socket, state);
        async move {
            if let Err(e) = handler.handle().await {
                tracing::error!("WebSocket error: {}", e);
            }
        }
    })
}

/// Chat completion handler
async fn chat_handler(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    // TODO: Implement chat completion with rate limiting
    Json(ChatResponse {
        id: uuid::Uuid::new_v4().to_string(),
        choices: vec![ChatChoice {
            message: Message::assistant("Hello from RustClaw!"),
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
    })
}

/// Streaming chat handler
async fn chat_stream_handler(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    // TODO: Implement streaming with SSE
    StatusCode::NOT_IMPLEMENTED
}

/// List available tools
async fn tools_list_handler(
    State(state): State<Arc<GatewayState>>,
) -> impl IntoResponse {
    Json(ToolsListResponse {
        tools: vec![],
    })
}

/// Invoke a tool
async fn tools_invoke_handler(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<ToolInvokeRequest>,
) -> impl IntoResponse {
    // TODO: Implement tool invocation
    Json(ToolInvokeResponse {
        result: "Tool executed".to_string(),
        is_error: false,
    })
}

/// List sessions
async fn sessions_list_handler(
    State(state): State<Arc<GatewayState>>,
) -> impl IntoResponse {
    Json(SessionsListResponse {
        sessions: vec![],
    })
}

/// Get a session
async fn session_get_handler(
    State(state): State<Arc<GatewayState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(SessionResponse {
        id,
        status: "active".to_string(),
        messages: vec![],
    })
}

/// Add message to session
async fn session_message_handler(
    State(state): State<Arc<GatewayState>>,
    Path(id): Path<String>,
    Json(message): Json<Message>,
) -> impl IntoResponse {
    StatusCode::OK
}

/// List agents
async fn agents_list_handler(
    State(state): State<Arc<GatewayState>>,
) -> impl IntoResponse {
    Json(AgentsListResponse {
        agents: vec![],
    })
}

/// Get an agent
async fn agent_get_handler(
    State(state): State<Arc<GatewayState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(AgentResponse {
        id,
        name: "default".to_string(),
        state: "idle".to_string(),
    })
}

/// Health check
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get configuration
async fn config_handler(
    State(state): State<Arc<GatewayState>>,
) -> impl IntoResponse {
    Json(&state.config)
}

/// Update configuration
async fn config_update_handler(
    State(state): State<Arc<GatewayState>>,
    Json(config): Json<RustClawConfig>,
) -> impl IntoResponse {
    // TODO: Implement config update
    StatusCode::OK
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct ChatRequest {
    messages: Vec<Message>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    id: String,
    choices: Vec<ChatChoice>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct ChatChoice {
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ToolsListResponse {
    tools: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ToolInvokeRequest {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ToolInvokeResponse {
    result: String,
    is_error: bool,
}

#[derive(Debug, Serialize)]
struct SessionsListResponse {
    sessions: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    id: String,
    status: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct AgentsListResponse {
    agents: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct AgentResponse {
    id: String,
    name: String,
    state: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}
