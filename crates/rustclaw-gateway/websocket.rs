//! WebSocket handler for RustClaw Gateway

use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use rustclaw_core::error::{Error, Result};

use super::GatewayState;
use crate::protocol::{GatewayMessage, GatewayCommand};

/// WebSocket handler
pub struct WebSocketHandler {
    socket: WebSocket,
    state: Arc<GatewayState>,
}

impl WebSocketHandler {
    /// Create a new WebSocket handler
    pub fn new(socket: WebSocket, state: Arc<GatewayState>) -> Self {
        Self { socket, state }
    }

    /// Handle WebSocket connection
    pub async fn handle(mut self) -> Result<()> {
        tracing::info!("WebSocket connection established");

        // Subscribe to broadcast messages
        let mut rx = self.state.message_tx.subscribe();

        while let Some(msg) = self.socket.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    tracing::debug!("Received WebSocket message: {}", text);
                    
                    // Parse the message
                    match serde_json::from_str::<GatewayMessage>(&text) {
                        Ok(gateway_msg) => {
                            self.handle_message(gateway_msg).await?;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse message: {}", e);
                            self.send_error("Invalid message format").await?;
                        }
                    }
                }
                Ok(WsMessage::Binary(data)) => {
                    tracing::debug!("Received binary message: {} bytes", data.len());
                    // Handle binary messages if needed
                }
                Ok(WsMessage::Ping(data)) => {
                    self.socket.send(WsMessage::Pong(data)).await?;
                }
                Ok(WsMessage::Pong(_)) => {
                    // Ignore pong
                }
                Ok(WsMessage::Close(_)) => {
                    tracing::info!("WebSocket connection closed by client");
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        tracing::info!("WebSocket connection ended");
        Ok(())
    }

    /// Handle a gateway message
    async fn handle_message(&mut self, msg: GatewayMessage) -> Result<()> {
        match msg.command {
            GatewayCommand::Ping => {
                self.send_pong().await?;
            }
            GatewayCommand::Chat { messages, model } => {
                // TODO: Implement chat with rate limiting
                self.send_chat_response("Chat received").await?;
            }
            GatewayCommand::Stream { messages, model } => {
                // TODO: Implement streaming
            }
            GatewayCommand::ToolInvoke { name, arguments } => {
                // TODO: Implement tool invocation
            }
            GatewayCommand::SessionCreate { config } => {
                // TODO: Implement session creation
            }
            GatewayCommand::SessionList => {
                // TODO: Implement session listing
            }
            GatewayCommand::ConfigGet => {
                // TODO: Implement config get
            }
            GatewayCommand::ConfigUpdate { config } => {
                // TODO: Implement config update
            }
            _ => {
                self.send_error("Unknown command").await?;
            }
        }
        Ok(())
    }

    /// Send a pong response
    async fn send_pong(&mut self) -> Result<()> {
        let msg = GatewayMessage::pong();
        self.send_message(&msg).await
    }

    /// Send a chat response
    async fn send_chat_response(&mut self, content: &str) -> Result<()> {
        let msg = GatewayMessage::chat_response(content);
        self.send_message(&msg).await
    }

    /// Send an error message
    async fn send_error(&mut self, error: &str) -> Result<()> {
        let msg = GatewayMessage::error(error);
        self.send_message(&msg).await
    }

    /// Send a gateway message
    async fn send_message(&mut self, msg: &GatewayMessage) -> Result<()> {
        let json = serde_json::to_string(msg)?;
        self.socket.send(WsMessage::Text(json)).await?;
        Ok(())
    }
}
