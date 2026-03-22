//! Tool Executor for RustClaw
//! 
//! Executes tools with concurrency control

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use rustclaw_core::{
    error::{Error, Result},
    tool::{Tool, ToolCall, ToolResult},
};

/// Tool executor trait
#[async_trait]
pub trait ToolExecutorTrait: Send + Sync {
    /// Execute a tool
    async fn execute(&self, call: ToolCall) -> Result<ToolResult>;
    
    /// List available tools
    fn list_tools(&self) -> Vec<&Tool>;
}

/// Built-in tool executor
pub struct ToolExecutor {
    tools: HashMap<String, Tool>,
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new() -> Self {
        let mut executor = Self {
            tools: HashMap::new(),
            handlers: HashMap::new(),
        };
        
        // Register built-in tools
        executor.register_builtins();
        executor
    }

    /// Register a tool
    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Register a tool with handler
    pub fn register_with_handler(&mut self, tool: Tool, handler: Arc<dyn ToolHandler>) {
        self.handlers.insert(tool.name.clone(), handler);
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Register built-in tools
    fn register_builtins(&mut self) {
        // Shell command tool
        self.register(Tool::new("shell", "Execute shell commands")
            .with_parameter("command", rustclaw_core::tool::ParameterType::string()
                .with_description("The shell command to execute"))
            .with_required("command")
            .dangerous());

        // HTTP request tool
        self.register(Tool::new("http_request", "Make HTTP requests")
            .with_parameter("method", rustclaw_core::tool::ParameterType::string()
                .with_description("HTTP method (GET, POST, PUT, DELETE)"))
            .with_parameter("url", rustclaw_core::tool::ParameterType::string()
                .with_description("The URL to request"))
            .with_parameter("headers", rustclaw_core::tool::ParameterType::object()
                .with_description("HTTP headers"))
            .with_parameter("body", rustclaw_core::tool::ParameterType::string()
                .with_description("Request body"))
            .with_required("method")
            .with_required("url"));

        // File read tool
        self.register(Tool::new("file_read", "Read file contents")
            .with_parameter("path", rustclaw_core::tool::ParameterType::string()
                .with_description("File path to read"))
            .with_required("path"));

        // File write tool
        self.register(Tool::new("file_write", "Write content to file")
            .with_parameter("path", rustclaw_core::tool::ParameterType::string()
                .with_description("File path to write"))
            .with_parameter("content", rustclaw_core::tool::ParameterType::string()
                .with_description("Content to write"))
            .with_required("path")
            .with_required("content")
            .dangerous());

        // Web search tool
        self.register(Tool::new("web_search", "Search the web")
            .with_parameter("query", rustclaw_core::tool::ParameterType::string()
                .with_description("Search query"))
            .with_parameter("num_results", rustclaw_core::tool::ParameterType::integer()
                .with_description("Number of results to return"))
            .with_required("query"));
    }

    /// Execute a tool call
    pub async fn execute(&self, call: ToolCall) -> Result<ToolResult> {
        // Check if tool exists
        let tool = self.tools.get(&call.function.name)
            .ok_or_else(|| Error::Tool(format!("Unknown tool: {}", call.function.name)))?;

        // Check for custom handler
        if let Some(handler) = self.handlers.get(&call.function.name) {
            return handler.execute(call).await;
        }

        // Execute built-in tool
        match call.function.name.as_str() {
            "shell" => self.execute_shell(&call).await,
            "http_request" => self.execute_http(&call).await,
            "file_read" => self.execute_file_read(&call).await,
            "file_write" => self.execute_file_write(&call).await,
            "web_search" => self.execute_web_search(&call).await,
            _ => Err(Error::Tool(format!("No handler for tool: {}", call.function.name))),
        }
    }

    async fn execute_shell(&self, call: &ToolCall) -> Result<ToolResult> {
        #[derive(serde::Deserialize)]
        struct ShellArgs {
            command: String,
        }

        let args: ShellArgs = serde_json::from_str(&call.function.arguments)
            .map_err(|e| Error::Tool(format!("Invalid arguments: {}", e)))?;

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .output()
            .await
            .map_err(|e| Error::Tool(format!("Shell execution failed: {}", e)))?;

        let result = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        Ok(ToolResult::success(&call.id, result))
    }

    async fn execute_http(&self, call: &ToolCall) -> Result<ToolResult> {
        // TODO: Implement HTTP request execution
        Ok(ToolResult::success(&call.id, "HTTP request executed"))
    }

    async fn execute_file_read(&self, call: &ToolCall) -> Result<ToolResult> {
        #[derive(serde::Deserialize)]
        struct FileReadArgs {
            path: String,
        }

        let args: FileReadArgs = serde_json::from_str(&call.function.arguments)
            .map_err(|e| Error::Tool(format!("Invalid arguments: {}", e)))?;

        let content = tokio::fs::read_to_string(&args.path)
            .await
            .map_err(|e| Error::Tool(format!("Failed to read file: {}", e)))?;

        Ok(ToolResult::success(&call.id, content))
    }

    async fn execute_file_write(&self, call: &ToolCall) -> Result<ToolResult> {
        #[derive(serde::Deserialize)]
        struct FileWriteArgs {
            path: String,
            content: String,
        }

        let args: FileWriteArgs = serde_json::from_str(&call.function.arguments)
            .map_err(|e| Error::Tool(format!("Invalid arguments: {}", e)))?;

        tokio::fs::write(&args.path, &args.content)
            .await
            .map_err(|e| Error::Tool(format!("Failed to write file: {}", e)))?;

        Ok(ToolResult::success(&call.id, "File written successfully"))
    }

    async fn execute_web_search(&self, call: &ToolCall) -> Result<ToolResult> {
        // TODO: Implement web search
        Ok(ToolResult::success(&call.id, "Web search executed"))
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool handler trait for custom tool implementations
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, call: ToolCall) -> Result<ToolResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_executor_creation() {
        let executor = ToolExecutor::new();
        assert!(executor.tools.contains_key("shell"));
        assert!(executor.tools.contains_key("http_request"));
    }
}
