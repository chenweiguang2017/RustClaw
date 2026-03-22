//! Context Manager for RustClaw
//! 
//! Manages conversation context and token limits

use std::collections::VecDeque;

use rustclaw_core::{
    error::Result,
    message::Message,
};

/// Context manager for handling conversation history
pub struct ContextManager {
    /// Maximum tokens for context
    max_tokens: usize,
    /// Message history
    messages: VecDeque<Message>,
    /// Estimated token count
    token_count: usize,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            messages: VecDeque::new(),
            token_count: 0,
        }
    }

    /// Add a message to context
    pub fn add_message(&mut self, message: Message) {
        let tokens = message.estimate_tokens();
        self.token_count += tokens;
        self.messages.push_back(message);
        
        // Trim if over limit
        self.trim_to_limit();
    }

    /// Add multiple messages
    pub fn add_messages(&mut self, messages: Vec<Message>) {
        for msg in messages {
            self.add_message(msg);
        }
    }

    /// Get all messages
    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    /// Get messages within token limit
    pub fn get_context_messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.token_count = 0;
    }

    /// Get current token count
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if context is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.token_count >= self.max_tokens
    }

    /// Trim messages to fit within token limit
    fn trim_to_limit(&mut self) {
        while self.token_count > self.max_tokens && self.messages.len() > 1 {
            if let Some(removed) = self.messages.pop_front() {
                self.token_count -= removed.estimate_tokens();
            }
        }
    }

    /// Compact context by summarizing old messages
    pub fn compact(&mut self) -> Result<()> {
        // TODO: Implement summarization
        Ok(())
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(128000) // Default 128k tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager() {
        let mut ctx = ContextManager::new(1000);
        
        ctx.add_message(Message::user("Hello"));
        ctx.add_message(Message::assistant("Hi there!"));
        
        assert_eq!(ctx.message_count(), 2);
        assert!(ctx.token_count() > 0);
    }

    #[test]
    fn test_context_trimming() {
        let mut ctx = ContextManager::new(10); // Very small limit
        
        // Add messages that exceed limit
        for i in 0..10 {
            ctx.add_message(Message::user(format!("Message {}", i)));
        }
        
        // Should have trimmed
        assert!(ctx.token_count() <= 10);
    }
}
