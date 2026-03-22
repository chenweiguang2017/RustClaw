//! Tool types for RustClaw

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool parameters schema (JSON Schema)
    pub parameters: ToolParameters,
    /// Whether the tool is dangerous
    #[serde(default)]
    pub dangerous: bool,
    /// Tool category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

impl Tool {
    /// Create a new tool
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: ToolParameters::default(),
            dangerous: false,
            category: None,
        }
    }

    /// Add a parameter to the tool
    pub fn with_parameter(mut self, name: impl Into<String>, param_type: ParameterType) -> Self {
        self.parameters.properties.insert(name.into(), param_type);
        self
    }

    /// Mark a parameter as required
    pub fn with_required(mut self, name: impl Into<String>) -> Self {
        self.parameters.required.push(name.into());
        self
    }

    /// Mark the tool as dangerous
    pub fn dangerous(mut self) -> Self {
        self.dangerous = true;
        self
    }

    /// Set the tool category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Convert to OpenAI function format
    pub fn to_openai_function(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": {
                    "type": "object",
                    "properties": self.parameters.properties.iter().map(|(k, v)| {
                        (k.clone(), v.to_json())
                    }).collect::<HashMap<String, serde_json::Value>>(),
                    "required": self.parameters.required,
                }
            }
        })
    }
}

/// Tool parameters schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    /// Parameter type (always "object" for tools)
    #[serde(rename = "type")]
    pub type_name: String,
    /// Parameter properties
    pub properties: HashMap<String, ParameterType>,
    /// Required parameters
    #[serde(default)]
    pub required: Vec<String>,
}

impl Default for ToolParameters {
    fn default() -> Self {
        Self {
            type_name: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }
}

/// Parameter type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterType {
    /// Parameter type
    #[serde(rename = "type")]
    pub type_name: String,
    /// Parameter description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Enum values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Items type for arrays
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ParameterType>>,
}

impl ParameterType {
    /// Create a string parameter
    pub fn string() -> Self {
        Self {
            type_name: "string".to_string(),
            description: None,
            r#enum: None,
            default: None,
            items: None,
        }
    }

    /// Create an integer parameter
    pub fn integer() -> Self {
        Self {
            type_name: "integer".to_string(),
            description: None,
            r#enum: None,
            default: None,
            items: None,
        }
    }

    /// Create a number parameter
    pub fn number() -> Self {
        Self {
            type_name: "number".to_string(),
            description: None,
            r#enum: None,
            default: None,
            items: None,
        }
    }

    /// Create a boolean parameter
    pub fn boolean() -> Self {
        Self {
            type_name: "boolean".to_string(),
            description: None,
            r#enum: None,
            default: None,
            items: None,
        }
    }

    /// Create an array parameter
    pub fn array(items: ParameterType) -> Self {
        Self {
            type_name: "array".to_string(),
            description: None,
            r#enum: None,
            default: None,
            items: Some(Box::new(items)),
        }
    }

    /// Create an object parameter
    pub fn object() -> Self {
        Self {
            type_name: "object".to_string(),
            description: None,
            r#enum: None,
            default: None,
            items: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add enum values
    pub fn with_enum(mut self, values: Vec<String>) -> Self {
        self.r#enum = Some(values);
        self
    }

    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool type (usually "function")
    pub r#type: String,
    /// Function call details
    pub function: FunctionCall,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            r#type: "function".to_string(),
            function: FunctionCall {
                name: name.into(),
                arguments: arguments.into(),
            },
        }
    }

    /// Parse arguments as a specific type
    pub fn parse_arguments<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.function.arguments)
    }
}

/// Function call details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments (JSON string)
    pub arguments: String,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID
    pub tool_call_id: String,
    /// Result content
    pub content: String,
    /// Whether the tool execution failed
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// Create an error result
    pub fn error(tool_call_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: error.into(),
            is_error: true,
        }
    }

    /// Convert to OpenAI tool message format
    pub fn to_openai_message(&self) -> serde_json::Value {
        serde_json::json!({
            "tool_call_id": self.tool_call_id,
            "role": "tool",
            "content": self.content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new("get_weather", "Get the current weather")
            .with_parameter("location", ParameterType::string().with_description("City name"))
            .with_required("location");

        assert_eq!(tool.name, "get_weather");
        assert!(tool.parameters.properties.contains_key("location"));
        assert!(tool.parameters.required.contains(&"location".to_string()));
    }

    #[test]
    fn test_tool_call_parsing() {
        let tool_call = ToolCall::new(
            "call_123",
            "get_weather",
            r#"{"location": "Beijing"}"#,
        );

        let args: HashMap<String, String> = tool_call.parse_arguments().unwrap();
        assert_eq!(args.get("location"), Some(&"Beijing".to_string()));
    }

    #[test]
    fn test_tool_result() {
        let result = ToolResult::success("call_123", "Sunny, 25°C");
        assert!(!result.is_error);

        let error = ToolResult::error("call_456", "City not found");
        assert!(error.is_error);
    }
}
