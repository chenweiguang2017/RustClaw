//! Workspace management for RustClaw
//! 
//! Handles workspace directory structure and file management

use std::path::{Path, PathBuf};

use rustclaw_core::error::{Error, Result};

use crate::soul::SoulFile;

/// Workspace directory structure
pub struct Workspace {
    /// Root path
    root: PathBuf,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
        }
    }

    /// Initialize the workspace
    pub fn init(&self) -> Result<()> {
        // Create directory structure
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.plugins_path())?;
        std::fs::create_dir_all(self.sessions_path())?;
        std::fs::create_dir_all(self.logs_path())?;

        // Create default files if they don't exist
        if !self.soul_file_path().exists() {
            let soul = SoulFile::default();
            soul.save_to_file(self.soul_file_path())?;
        }

        if !self.config_file_path().exists() {
            let default_config = "# RustClaw Configuration\n\
model:\n\
  provider: openai\n\
  model_name: gpt-4\n\
rate_limit:\n\
  rpm: 60\n\
concurrency:\n\
  max_concurrent_requests: 10\n";
            std::fs::write(self.config_file_path(), default_config)?;
        }

        Ok(())
    }

    /// Get the root path
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get SOUL.md file path
    pub fn soul_file_path(&self) -> PathBuf {
        self.root.join("SOUL.md")
    }

    /// Get AGENTS.md file path
    pub fn agents_file_path(&self) -> PathBuf {
        self.root.join("AGENTS.md")
    }

    /// Get config file path
    pub fn config_file_path(&self) -> PathBuf {
        self.root.join("config.yaml")
    }

    /// Get plugins directory path
    pub fn plugins_path(&self) -> PathBuf {
        self.root.join("plugins")
    }

    /// Get sessions directory path
    pub fn sessions_path(&self) -> PathBuf {
        self.root.join("sessions")
    }

    /// Get logs directory path
    pub fn logs_path(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// Load SOUL.md
    pub fn load_soul(&self) -> Result<SoulFile> {
        SoulFile::from_file(self.soul_file_path())
    }

    /// Save SOUL.md
    pub fn save_soul(&self, soul: &SoulFile) -> Result<()> {
        soul.save_to_file(self.soul_file_path())
    }

    /// Check if workspace exists
    pub fn exists(&self) -> bool {
        self.root.exists() && self.root.is_dir()
    }

    /// List all files in workspace
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, files)?;
                } else {
                    files.push(path);
                }
            }
            Ok(())
        }

        if self.exists() {
            walk_dir(&self.root, &mut files)?;
        }

        Ok(files)
    }

    /// Clean up workspace
    pub fn clean(&self) -> Result<()> {
        if self.exists() {
            std::fs::remove_dir_all(&self.root)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_workspace_init() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path().join(".rustclaw"));
        
        workspace.init().unwrap();
        
        assert!(workspace.exists());
        assert!(workspace.soul_file_path().exists());
        assert!(workspace.config_file_path().exists());
    }
}
