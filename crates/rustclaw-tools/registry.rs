//! Tool Registry for RustClaw

use std::collections::HashMap;
use std::sync::Arc;

use rustclaw_core::{
    error::{Error, Result},
    tool::{Tool, ToolCall, ToolResult},
};

use crate::builtin::BuiltinTools;

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            handlers: HashMap::new(),
        };

        // Register built-in tools
        for tool in BuiltinTools::all() {
            registry.register(tool);
        }

        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Register a tool with a handler
    pub fn register_with_handler<H: ToolHandler + 'static>(&mut self, tool: Tool, handler: H) {
        self.handlers.insert(tool.name.clone(), Arc::new(handler));
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// List all tools
    pub fn list(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// List tools by category
    pub fn list_by_category(&self, category: &str) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|t| t.category.as_deref() == Some(category))
            .collect()
    }

    /// Check if a tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get tool count
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Convert to OpenAI tools format
    pub fn to_openai_tools(&self) -> Vec<serde_json::Value> {
        self.tools.values().map(|t| t.to_openai_function()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool handler trait
#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool
    async fn execute(&self, call: ToolCall) -> Result<ToolResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        assert!(registry.count() > 0);
        assert!(registry.contains("shell"));
    }
}
