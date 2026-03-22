//! Plugin Loader for RustClaw

use std::path::{Path, PathBuf};
use std::collections::HashMap;

use rustclaw_core::error::{Error, Result};

use crate::types::{Plugin, PluginInfo, PluginType};

/// Plugin loader
pub struct PluginLoader {
    /// Plugin directories to search
    plugin_dirs: Vec<PathBuf>,
    /// Loaded plugins
    plugins: HashMap<String, Plugin>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            plugin_dirs: vec![
                PathBuf::from(".rustclaw/plugins"),
                PathBuf::from("~/.rustclaw/plugins"),
            ],
            plugins: HashMap::new(),
        }
    }

    /// Add a plugin directory
    pub fn add_plugin_dir(&mut self, dir: impl Into<PathBuf>) {
        self.plugin_dirs.push(dir.into());
    }

    /// Discover plugins in all directories
    pub fn discover(&mut self) -> Result<Vec<PluginInfo>> {
        let mut discovered = Vec::new();

        for dir in &self.plugin_dirs {
            if dir.exists() {
                discovered.extend(self.discover_in_dir(dir)?);
            }
        }

        Ok(discovered)
    }

    /// Discover plugins in a specific directory
    fn discover_in_dir(&self, dir: &Path) -> Result<Vec<PluginInfo>> {
        let mut plugins = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check for plugin manifest
                let manifest_path = path.join("plugin.yaml");
                if manifest_path.exists() {
                    if let Ok(info) = self.load_manifest(&manifest_path) {
                        plugins.push(info);
                    }
                }

                // Check for package.json (TypeScript plugins)
                let package_path = path.join("package.json");
                if package_path.exists() {
                    if let Ok(info) = self.load_package_json(&package_path) {
                        plugins.push(info);
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Load plugin manifest
    fn load_manifest(&self, path: &Path) -> Result<PluginInfo> {
        let content = std::fs::read_to_string(path)?;
        let info: PluginInfo = serde_yaml::from_str(&content)?;
        Ok(info)
    }

    /// Load package.json for TypeScript plugins
    fn load_package_json(&self, path: &Path) -> Result<PluginInfo> {
        let content = std::fs::read_to_string(path)?;
        let package: serde_json::Value = serde_json::from_str(&content)?;

        let name = package["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let version = package["version"]
            .as_str()
            .unwrap_or("0.0.0")
            .to_string();

        let description = package["description"]
            .as_str()
            .map(|s| s.to_string());

        // Check for OpenClaw plugin metadata
        let openclaw = package.get("openclaw");

        Ok(PluginInfo {
            name,
            version,
            description,
            author: None,
            entry: "index.ts".to_string(),
            plugin_type: PluginType::TypeScript,
            dependencies: vec![],
            config_schema: None,
        })
    }

    /// Load a plugin by name
    pub fn load(&mut self, name: &str) -> Result<()> {
        // Find plugin in discovered plugins
        let info = self.discover()?
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| Error::Plugin(format!("Plugin not found: {}", name)))?;

        let mut plugin = Plugin::new(info);
        plugin.load().map_err(|e| Error::Plugin(e))?;

        self.plugins.insert(name.to_string(), plugin);

        Ok(())
    }

    /// Unload a plugin
    pub fn unload(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.unload().map_err(|e| Error::Plugin(e))?;
            self.plugins.remove(name);
        }
        Ok(())
    }

    /// Get a loaded plugin
    pub fn get(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name)
    }

    /// List all loaded plugins
    pub fn list(&self) -> Vec<&Plugin> {
        self.plugins.values().collect()
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new();
        assert!(loader.plugin_dirs.len() >= 1);
    }
}
