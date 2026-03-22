//! SOUL.md file handling for RustClaw
//! 
//! The SOUL.md file defines the agent's personality, values, and behavior

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use rustclaw_core::error::{Error, Result};

/// SOUL.md file representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulFile {
    /// Agent name
    pub name: String,
    /// Agent description
    pub description: Option<String>,
    /// Agent personality traits
    pub personality: Vec<String>,
    /// Agent values
    pub values: Vec<String>,
    /// Agent capabilities
    pub capabilities: Vec<String>,
    /// System prompt template
    pub system_prompt: Option<String>,
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for SoulFile {
    fn default() -> Self {
        Self {
            name: "RustClaw Agent".to_string(),
            description: Some("A helpful AI assistant powered by RustClaw".to_string()),
            personality: vec![
                "Helpful".to_string(),
                "Precise".to_string(),
                "Efficient".to_string(),
            ],
            values: vec![
                "Accuracy".to_string(),
                "Safety".to_string(),
                "Privacy".to_string(),
            ],
            capabilities: vec![
                "Code generation".to_string(),
                "Text analysis".to_string(),
                "Tool execution".to_string(),
            ],
            system_prompt: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl SoulFile {
    /// Create a new SOUL file
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Load from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_markdown(&content)
    }

    /// Parse from markdown content
    pub fn from_markdown(content: &str) -> Result<Self> {
        let mut soul = SoulFile::default();
        let mut current_section = String::new();
        let mut current_content = String::new();

        for line in content.lines() {
            if line.starts_with("# ") {
                // Main heading - agent name
                soul.name = line[2..].trim().to_string();
            } else if line.starts_with("## ") {
                // Save previous section
                if !current_section.is_empty() {
                    soul.process_section(&current_section, &current_content);
                }
                current_section = line[3..].trim().to_string();
                current_content = String::new();
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        // Process last section
        if !current_section.is_empty() {
            soul.process_section(&current_section, &current_content);
        }

        Ok(soul)
    }

    /// Process a section of the SOUL file
    fn process_section(&mut self, section: &str, content: &str) {
        let items: Vec<String> = content
            .lines()
            .filter(|l| l.starts_with("- ") || l.starts_with("* "))
            .map(|l| l[2..].trim().to_string())
            .collect();

        match section.to_lowercase().as_str() {
            "personality" | "traits" => {
                self.personality = items;
            }
            "values" => {
                self.values = items;
            }
            "capabilities" | "skills" => {
                self.capabilities = items;
            }
            "system prompt" | "system" => {
                self.system_prompt = Some(content.trim().to_string());
            }
            "description" => {
                self.description = Some(content.trim().to_string());
            }
            _ => {
                // Store as metadata
                self.metadata.insert(
                    section.to_lowercase(),
                    serde_json::to_value(items).unwrap_or(serde_json::Value::Null),
                );
            }
        }
    }

    /// Convert to markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {}\n\n", self.name));

        if let Some(ref desc) = self.description {
            md.push_str(&format!("## Description\n\n{}\n\n", desc));
        }

        if !self.personality.is_empty() {
            md.push_str("## Personality\n\n");
            for trait_item in &self.personality {
                md.push_str(&format!("- {}\n", trait_item));
            }
            md.push('\n');
        }

        if !self.values.is_empty() {
            md.push_str("## Values\n\n");
            for value in &self.values {
                md.push_str(&format!("- {}\n", value));
            }
            md.push('\n');
        }

        if !self.capabilities.is_empty() {
            md.push_str("## Capabilities\n\n");
            for cap in &self.capabilities {
                md.push_str(&format!("- {}\n", cap));
            }
            md.push('\n');
        }

        if let Some(ref prompt) = self.system_prompt {
            md.push_str("## System Prompt\n\n");
            md.push_str(prompt);
            md.push_str("\n\n");
        }

        md
    }

    /// Save to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = self.to_markdown();
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate system prompt
    pub fn generate_system_prompt(&self) -> String {
        if let Some(ref prompt) = self.system_prompt {
            return prompt.clone();
        }

        let mut prompt = format!("You are {}, ", self.name);

        if let Some(ref desc) = self.description {
            prompt.push_str(&format!("{}, ", desc));
        }

        if !self.personality.is_empty() {
            prompt.push_str("with the following personality traits: ");
            prompt.push_str(&self.personality.join(", "));
            prompt.push_str(". ");
        }

        if !self.values.is_empty() {
            prompt.push_str("You value: ");
            prompt.push_str(&self.values.join(", "));
            prompt.push_str(". ");
        }

        if !self.capabilities.is_empty() {
            prompt.push_str("Your capabilities include: ");
            prompt.push_str(&self.capabilities.join(", "));
            prompt.push_str(". ");
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soul_file_default() {
        let soul = SoulFile::default();
        assert_eq!(soul.name, "RustClaw Agent");
    }

    #[test]
    fn test_soul_file_markdown() {
        let md = r#"# Test Agent

## Personality

- Friendly
- Helpful

## Values

- Accuracy
- Safety

## System Prompt

You are a test agent.
"#;

        let soul = SoulFile::from_markdown(md).unwrap();
        assert_eq!(soul.name, "Test Agent");
        assert_eq!(soul.personality, vec!["Friendly", "Helpful"]);
        assert_eq!(soul.values, vec!["Accuracy", "Safety"]);
    }
}
