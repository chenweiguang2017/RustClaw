//! Concurrency Controller for RustClaw
//! 
//! Provides control over maximum concurrent operations:
//! - API requests
//! - Tool executions
//! - Sessions

use crate::error::{Error, Result};
use crate::types::ConcurrencyConfig;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};

/// Concurrency controller for managing concurrent operations
#[derive(Debug)]
pub struct ConcurrencyController {
    /// Semaphore for API requests
    request_semaphore: Arc<Semaphore>,
    /// Semaphore for tool executions
    tool_semaphore: Arc<Semaphore>,
    /// Semaphore for sessions
    session_semaphore: Arc<Semaphore>,
    /// Current active requests count
    active_requests: AtomicUsize,
    /// Current active tools count
    active_tools: AtomicUsize,
    /// Current active sessions count
    active_sessions: AtomicUsize,
    /// Configuration
    config: RwLock<ConcurrencyConfig>,
}

impl ConcurrencyController {
    /// Create a new concurrency controller with the given configuration
    pub fn new(config: ConcurrencyConfig) -> Self {
        Self {
            request_semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            tool_semaphore: Arc::new(Semaphore::new(config.max_concurrent_tools)),
            session_semaphore: Arc::new(Semaphore::new(config.max_concurrent_sessions)),
            active_requests: AtomicUsize::new(0),
            active_tools: AtomicUsize::new(0),
            active_sessions: AtomicUsize::new(0),
            config: RwLock::new(config),
        }
    }

    /// Create a controller with default configuration
    pub fn default() -> Self {
        Self::new(ConcurrencyConfig::default())
    }

    /// Acquire a permit for an API request
    pub async fn acquire_request(&self) -> Result<RequestPermit> {
        let permit = self.request_semaphore
            .acquire()
            .await
            .map_err(|e| Error::ConcurrencyLimitExceeded(format!(
                "Failed to acquire request permit: {}", e
            )))?;
        
        self.active_requests.fetch_add(1, Ordering::SeqCst);
        
        Ok(RequestPermit {
            permit: Some(permit),
            controller: self,
        })
    }

    /// Try to acquire a request permit without blocking
    pub fn try_acquire_request(&self) -> Option<RequestPermit> {
        let permit = self.request_semaphore.try_acquire().ok()?;
        self.active_requests.fetch_add(1, Ordering::SeqCst);
        
        Some(RequestPermit {
            permit: Some(permit),
            controller: self,
        })
    }

    /// Acquire a permit for tool execution
    pub async fn acquire_tool(&self) -> Result<ToolPermit> {
        let permit = self.tool_semaphore
            .acquire()
            .await
            .map_err(|e| Error::ConcurrencyLimitExceeded(format!(
                "Failed to acquire tool permit: {}", e
            )))?;
        
        self.active_tools.fetch_add(1, Ordering::SeqCst);
        
        Ok(ToolPermit {
            permit: Some(permit),
            controller: self,
        })
    }

    /// Try to acquire a tool permit without blocking
    pub fn try_acquire_tool(&self) -> Option<ToolPermit> {
        let permit = self.tool_semaphore.try_acquire().ok()?;
        self.active_tools.fetch_add(1, Ordering::SeqCst);
        
        Some(ToolPermit {
            permit: Some(permit),
            controller: self,
        })
    }

    /// Acquire a permit for a session
    pub async fn acquire_session(&self) -> Result<SessionPermit> {
        let permit = self.session_semaphore
            .acquire()
            .await
            .map_err(|e| Error::ConcurrencyLimitExceeded(format!(
                "Failed to acquire session permit: {}", e
            )))?;
        
        self.active_sessions.fetch_add(1, Ordering::SeqCst);
        
        Ok(SessionPermit {
            permit: Some(permit),
            controller: self,
        })
    }

    /// Try to acquire a session permit without blocking
    pub fn try_acquire_session(&self) -> Option<SessionPermit> {
        let permit = self.session_semaphore.try_acquire().ok()?;
        self.active_sessions.fetch_add(1, Ordering::SeqCst);
        
        Some(SessionPermit {
            permit: Some(permit),
            controller: self,
        })
    }

    /// Get the number of available request slots
    pub fn available_requests(&self) -> usize {
        self.request_semaphore.available_permits()
    }

    /// Get the number of available tool slots
    pub fn available_tools(&self) -> usize {
        self.tool_semaphore.available_permits()
    }

    /// Get the number of available session slots
    pub fn available_sessions(&self) -> usize {
        self.session_semaphore.available_permits()
    }

    /// Get the number of active requests
    pub fn active_requests(&self) -> usize {
        self.active_requests.load(Ordering::SeqCst)
    }

    /// Get the number of active tools
    pub fn active_tools(&self) -> usize {
        self.active_tools.load(Ordering::SeqCst)
    }

    /// Get the number of active sessions
    pub fn active_sessions(&self) -> usize {
        self.active_sessions.load(Ordering::SeqCst)
    }

    /// Get the current configuration
    pub fn config(&self) -> ConcurrencyConfig {
        self.config.read().clone()
    }

    /// Update the configuration (note: this creates new semaphores)
    pub fn update_config(&self, config: ConcurrencyConfig) {
        let mut current = self.config.write();
        *current = config;
    }

    /// Get statistics about current concurrency
    pub fn stats(&self) -> ConcurrencyStats {
        let config = self.config.read();
        ConcurrencyStats {
            active_requests: self.active_requests.load(Ordering::SeqCst),
            max_requests: config.max_concurrent_requests,
            available_requests: self.request_semaphore.available_permits(),
            active_tools: self.active_tools.load(Ordering::SeqCst),
            max_tools: config.max_concurrent_tools,
            available_tools: self.tool_semaphore.available_permits(),
            active_sessions: self.active_sessions.load(Ordering::SeqCst),
            max_sessions: config.max_concurrent_sessions,
            available_sessions: self.session_semaphore.available_permits(),
        }
    }
}

/// Statistics about current concurrency
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConcurrencyStats {
    pub active_requests: usize,
    pub max_requests: usize,
    pub available_requests: usize,
    pub active_tools: usize,
    pub max_tools: usize,
    pub available_tools: usize,
    pub active_sessions: usize,
    pub max_sessions: usize,
    pub available_sessions: usize,
}

/// Permit for an API request
pub struct RequestPermit<'a> {
    permit: Option<SemaphorePermit<'a>>,
    controller: &'a ConcurrencyController,
}

impl<'a> Drop for RequestPermit<'a> {
    fn drop(&mut self) {
        self.controller.active_requests.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Permit for tool execution
pub struct ToolPermit<'a> {
    permit: Option<SemaphorePermit<'a>>,
    controller: &'a ConcurrencyController,
}

impl<'a> Drop for ToolPermit<'a> {
    fn drop(&mut self) {
        self.controller.active_tools.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Permit for a session
pub struct SessionPermit<'a> {
    permit: Option<SemaphorePermit<'a>>,
    controller: &'a ConcurrencyController,
}

impl<'a> Drop for SessionPermit<'a> {
    fn drop(&mut self) {
        self.controller.active_sessions.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Builder for creating concurrency controllers
pub struct ConcurrencyControllerBuilder {
    max_concurrent_requests: usize,
    max_concurrent_tools: usize,
    max_concurrent_sessions: usize,
}

impl ConcurrencyControllerBuilder {
    pub fn new() -> Self {
        Self {
            max_concurrent_requests: 10,
            max_concurrent_tools: 5,
            max_concurrent_sessions: 100,
        }
    }

    pub fn max_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = max;
        self
    }

    pub fn max_tools(mut self, max: usize) -> Self {
        self.max_concurrent_tools = max;
        self
    }

    pub fn max_sessions(mut self, max: usize) -> Self {
        self.max_concurrent_sessions = max;
        self
    }

    pub fn build(self) -> ConcurrencyController {
        let config = ConcurrencyConfig {
            max_concurrent_requests: self.max_concurrent_requests,
            max_concurrent_tools: self.max_concurrent_tools,
            max_concurrent_sessions: self.max_concurrent_sessions,
        };
        ConcurrencyController::new(config)
    }
}

impl Default for ConcurrencyControllerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrency_controller_basic() {
        let controller = ConcurrencyControllerBuilder::new()
            .max_requests(5)
            .max_tools(3)
            .max_sessions(10)
            .build();

        // Acquire request permit
        let permit = controller.acquire_request().await.unwrap();
        assert_eq!(controller.active_requests(), 1);
        assert_eq!(controller.available_requests(), 4);
        
        drop(permit);
        assert_eq!(controller.active_requests(), 0);
    }

    #[tokio::test]
    async fn test_concurrency_controller_limit() {
        let controller = ConcurrencyControllerBuilder::new()
            .max_requests(2)
            .build();

        // Acquire two permits
        let p1 = controller.acquire_request().await.unwrap();
        let p2 = controller.acquire_request().await.unwrap();

        // Should have no available slots
        assert_eq!(controller.available_requests(), 0);

        // Try acquire should fail
        assert!(controller.try_acquire_request().is_none());

        drop(p1);
        drop(p2);

        // Should have slots available again
        assert_eq!(controller.available_requests(), 2);
    }

    #[test]
    fn test_concurrency_stats() {
        let controller = ConcurrencyController::default();
        let stats = controller.stats();
        
        assert_eq!(stats.active_requests, 0);
        assert!(stats.max_requests > 0);
    }
}
