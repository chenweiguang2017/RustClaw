//! Plugin types for RustClaw

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: Option<String>,
    /// Plugin author
    pub author: Option<String>,
    /// Plugin entry point
    pub entry: String,
    /// Plugin type
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Plugin configuration schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
}

/// Plugin type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    /// Native Rust plugin
    Native,
    /// TypeScript plugin (OpenClaw compatible)
    TypeScript,
    /// Python plugin
    Python,
}

/// Plugin instance
#[derive(Debug)]
pub struct Plugin {
    /// Plugin info
    pub info: PluginInfo,
    /// Plugin state
    pub state: PluginState,
    /// Plugin configuration
    pub config: HashMap<String, serde_json::Value>,
}

impl Plugin {
    /// Create a new plugin
    pub fn new(info: PluginInfo) -> Self {
        Self {
            info,
            state: PluginState::Unloaded,
            config: HashMap::new(),
        }
    }

    /// Load the plugin
    pub fn load(&mut self) -> Result<(), String> {
        self.state = PluginState::Loaded;
        Ok(())
    }

    /// Unload the plugin
    pub fn unload(&mut self) -> Result<(), String> {
        self.state = PluginState::Unloaded;
        Ok(())
    }

    /// Enable the plugin
    pub fn enable(&mut self) -> Result<(), String> {
        self.state = PluginState::Enabled;
        Ok(())
    }

    /// Disable the plugin
    pub fn disable(&mut self) -> Result<(), String> {
        self.state = PluginState::Disabled;
        Ok(())
    }
}

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginState {
    /// Plugin is unloaded
    Unloaded,
    /// Plugin is loaded but not active
    Loaded,
    /// Plugin is enabled and active
    Enabled,
    /// Plugin is disabled
    Disabled,
    /// Plugin has an error
    Error,
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unloaded => write!(f, "unloaded"),
            Self::Loaded => write!(f, "loaded"),
            Self::Enabled => write!(f, "enabled"),
            Self::Disabled => write!(f, "disabled"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let info = PluginInfo {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test plugin".to_string()),
            author: None,
            entry: "index.ts".to_string(),
            plugin_type: PluginType::TypeScript,
            dependencies: vec![],
            config_schema: None,
        };

        let plugin = Plugin::new(info);
        assert_eq!(plugin.state, PluginState::Unloaded);
    }
}
