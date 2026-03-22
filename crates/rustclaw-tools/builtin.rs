//! Built-in tools for RustClaw

use rustclaw_core::tool::{Tool, ParameterType};

/// Get all built-in tools
pub fn get_builtin_tools() -> Vec<Tool> {
    vec![
        // Shell command tool
        Tool::new("shell", "Execute shell commands")
            .with_parameter("command", ParameterType::string()
                .with_description("The shell command to execute"))
            .with_required("command")
            .dangerous(),

        // HTTP request tool
        Tool::new("http_request", "Make HTTP requests")
            .with_parameter("method", ParameterType::string()
                .with_description("HTTP method (GET, POST, PUT, DELETE)")
                .with_enum(vec!["GET".into(), "POST".into(), "PUT".into(), "DELETE".into()]))
            .with_parameter("url", ParameterType::string()
                .with_description("The URL to request"))
            .with_parameter("headers", ParameterType::object()
                .with_description("HTTP headers as key-value pairs"))
            .with_parameter("body", ParameterType::string()
                .with_description("Request body (for POST/PUT)"))
            .with_required("method")
            .with_required("url"),

        // File read tool
        Tool::new("file_read", "Read file contents")
            .with_parameter("path", ParameterType::string()
                .with_description("File path to read"))
            .with_required("path"),

        // File write tool
        Tool::new("file_write", "Write content to file")
            .with_parameter("path", ParameterType::string()
                .with_description("File path to write"))
            .with_parameter("content", ParameterType::string()
                .with_description("Content to write"))
            .with_required("path")
            .with_required("content")
            .dangerous(),

        // Web search tool
        Tool::new("web_search", "Search the web")
            .with_parameter("query", ParameterType::string()
                .with_description("Search query"))
            .with_parameter("num_results", ParameterType::integer()
                .with_description("Number of results to return (default: 10)"))
            .with_required("query"),

        // Code execution tool
        Tool::new("code_execute", "Execute code in a sandboxed environment")
            .with_parameter("language", ParameterType::string()
                .with_description("Programming language")
                .with_enum(vec!["python".into(), "javascript".into(), "rust".into()]))
            .with_parameter("code", ParameterType::string()
                .with_description("Code to execute"))
            .with_required("language")
            .with_required("code")
            .dangerous(),

        // Memory tool
        Tool::new("memory_store", "Store information in memory")
            .with_parameter("key", ParameterType::string()
                .with_description("Key to store the value under"))
            .with_parameter("value", ParameterType::string()
                .with_description("Value to store"))
            .with_required("key")
            .with_required("value"),

        // Memory retrieve tool
        Tool::new("memory_retrieve", "Retrieve information from memory")
            .with_parameter("key", ParameterType::string()
                .with_description("Key to retrieve"))
            .with_required("key"),

        // Calculator tool
        Tool::new("calculator", "Perform mathematical calculations")
            .with_parameter("expression", ParameterType::string()
                .with_description("Mathematical expression to evaluate"))
            .with_required("expression"),

        // Date/time tool
        Tool::new("datetime", "Get current date and time")
            .with_parameter("timezone", ParameterType::string()
                .with_description("Timezone (e.g., 'UTC', 'America/New_York')"))
            .with_parameter("format", ParameterType::string()
                .with_description("Date format string")),
    ]
}

/// Built-in tools container
pub struct BuiltinTools;

impl BuiltinTools {
    /// Get all built-in tools
    pub fn all() -> Vec<Tool> {
        get_builtin_tools()
    }

    /// Get tool by name
    pub fn get(name: &str) -> Option<Tool> {
        get_builtin_tools().into_iter().find(|t| t.name == name)
    }

    /// Check if tool is built-in
    pub fn is_builtin(name: &str) -> bool {
        get_builtin_tools().iter().any(|t| t.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_tools() {
        let tools = BuiltinTools::all();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "shell"));
    }

    #[test]
    fn test_builtin_tool_get() {
        let tool = BuiltinTools::get("shell");
        assert!(tool.is_some());
    }
}
