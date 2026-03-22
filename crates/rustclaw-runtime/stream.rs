//! Stream Handler for RustClaw
//! 
//! Handles streaming responses from LLM

use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

use rustclaw_core::error::Result;

/// Stream handler for processing LLM streaming responses
pub struct StreamHandler {
    buffer: String,
    done: bool,
}

impl StreamHandler {
    /// Create a new stream handler
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            done: false,
        }
    }

    /// Process a chunk of the stream
    pub fn process_chunk(&mut self, chunk: &str) -> Option<String> {
        if self.done {
            return None;
        }

        // Parse SSE format
        for line in chunk.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                
                if data == "[DONE]" {
                    self.done = true;
                    return None;
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                        for choice in choices {
                            if let Some(delta) = choice.get("delta").and_then(|d| d.get("content")) {
                                if let Some(content) = delta.as_str() {
                                    self.buffer.push_str(content);
                                    return Some(content.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Get the complete buffer
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    /// Check if stream is done
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Reset the handler
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.done = false;
    }
}

impl Default for StreamHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// SSE event parser
pub struct SseParser {
    event_type: Option<String>,
    data: String,
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            event_type: None,
            data: String::new(),
        }
    }

    pub fn parse(&mut self, chunk: &str) -> Vec<SseEvent> {
        let mut events = Vec::new();

        for line in chunk.lines() {
            if line.is_empty() {
                // Empty line signals end of event
                if !self.data.is_empty() {
                    events.push(SseEvent {
                        event_type: self.event_type.take().unwrap_or_else(|| "message".to_string()),
                        data: std::mem::take(&mut self.data),
                    });
                }
            } else if line.starts_with(':') {
                // Comment, ignore
            } else if let Some(event_type) = line.strip_prefix("event:") {
                self.event_type = Some(event_type.trim().to_string());
            } else if let Some(data) = line.strip_prefix("data:") {
                if !self.data.is_empty() {
                    self.data.push('\n');
                }
                self.data.push_str(data.trim());
            }
        }

        events
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

/// SSE event
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: String,
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_handler() {
        let mut handler = StreamHandler::new();
        
        let chunk = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
        let result = handler.process_chunk(chunk);
        
        assert_eq!(result, Some("Hello".to_string()));
        assert_eq!(handler.get_buffer(), "Hello");
    }

    #[test]
    fn test_sse_parser() {
        let mut parser = SseParser::new();
        
        let chunk = "event: message\ndata: {\"test\": true}\n\n";
        let events = parser.parse(chunk);
        
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "message");
        assert_eq!(events[0].data, "{\"test\": true}");
    }
}
