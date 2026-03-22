//! Rate Limiter for RustClaw
//! 
//! Implements RPM (Requests Per Minute) control with support for:
//! - Fixed interval based on RPM
//! - Random interval within a configurable range
//! - Token bucket algorithm for smooth rate limiting

use crate::error::{Error, Result};
use crate::types::RateLimitConfig;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

/// Rate limiter state
#[derive(Debug)]
struct RateLimiterState {
    /// Timestamps of recent requests
    request_timestamps: VecDeque<Instant>,
    /// Last request time
    last_request: Option<Instant>,
}

/// Rate limiter for API calls
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    state: Arc<Mutex<RateLimiterState>>,
    /// Semaphore for concurrent request limiting
    request_semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig, max_concurrent: usize) -> Self {
        let state = RateLimiterState {
            request_timestamps: VecDeque::with_capacity(1000),
            last_request: None,
        };

        Self {
            config,
            state: Arc::new(Mutex::new(state)),
            request_semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Create a rate limiter with default configuration
    pub fn default_with_concurrency(max_concurrent: usize) -> Self {
        Self::new(RateLimitConfig::default(), max_concurrent)
    }

    /// Calculate the wait time before the next request can be made
    pub fn calculate_wait_time(&self) -> Duration {
        let now = Instant::now();
        let mut state = self.state.lock();

        // Clean up old timestamps (older than 1 minute)
        let one_minute_ago = now - Duration::from_secs(60);
        while let Some(&timestamp) = state.request_timestamps.front() {
            if timestamp < one_minute_ago {
                state.request_timestamps.pop_front();
            } else {
                break;
            }
        }

        // Check if we need to wait based on RPM
        if let Some(rpm) = self.config.rpm {
            let requests_in_last_minute = state.request_timestamps.len() as u32;
            
            if requests_in_last_minute >= rpm {
                // Need to wait until the oldest request is more than a minute old
                if let Some(&oldest) = state.request_timestamps.front() {
                    let wait_until = oldest + Duration::from_secs(60);
                    if let Some(wait_duration) = wait_until.checked_duration_since(now) {
                        return wait_duration;
                    }
                }
            }
        }

        // Check minimum interval
        if let Some(last_request) = state.last_request {
            let interval = self.calculate_interval();
            let elapsed = now.duration_since(last_request);
            
            if elapsed < interval {
                return interval - elapsed;
            }
        }

        Duration::ZERO
    }

    /// Calculate the interval between requests
    fn calculate_interval(&self) -> Duration {
        if self.config.use_random_interval {
            if let (Some(min), Some(max)) = 
                (self.config.min_interval_ms, self.config.max_interval_ms) {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let interval_ms = rng.gen_range(min..=max);
                return Duration::from_millis(interval_ms);
            }
        }

        if let Some(rpm) = self.config.rpm {
            Duration::from_millis(60000 / rpm as u64)
        } else if let Some(min) = self.config.min_interval_ms {
            Duration::from_millis(min)
        } else {
            Duration::from_millis(1000) // Default 1 second
        }
    }

    /// Acquire a permit to make a request
    /// This will block until a request can be made according to the rate limit
    pub async fn acquire(&self) -> Result<RateLimitPermit> {
        // First, wait for rate limit
        let wait_time = self.calculate_wait_time();
        if !wait_time.is_zero() {
            tracing::debug!("Rate limiter waiting for {:?}", wait_time);
            sleep(wait_time).await;
        }

        // Then acquire semaphore permit for concurrency
        let permit = self.request_semaphore
            .acquire()
            .await
            .map_err(|e| Error::ConcurrencyLimitExceeded(e.to_string()))?;

        // Record this request
        {
            let mut state = self.state.lock();
            state.request_timestamps.push_back(Instant::now());
            state.last_request = Some(Instant::now());
        }

        Ok(RateLimitPermit {
            permit: Some(permit),
        })
    }

    /// Try to acquire a permit without blocking
    /// Returns None if the rate limit would be exceeded
    pub fn try_acquire(&self) -> Option<RateLimitPermit> {
        let wait_time = self.calculate_wait_time();
        if !wait_time.is_zero() {
            return None;
        }

        let permit = self.request_semaphore.try_acquire().ok()?;
        
        // Record this request
        {
            let mut state = self.state.lock();
            state.request_timestamps.push_back(Instant::now());
            state.last_request = Some(Instant::now());
        }

        Some(RateLimitPermit {
            permit: Some(permit),
        })
    }

    /// Get the current number of requests in the last minute
    pub fn current_rpm(&self) -> u32 {
        let now = Instant::now();
        let mut state = self.state.lock();
        
        // Clean up old timestamps
        let one_minute_ago = now - Duration::from_secs(60);
        while let Some(&timestamp) = state.request_timestamps.front() {
            if timestamp < one_minute_ago {
                state.request_timestamps.pop_front();
            } else {
                break;
            }
        }

        state.request_timestamps.len() as u32
    }

    /// Get the number of available request slots
    pub fn available_slots(&self) -> usize {
        self.request_semaphore.available_permits()
    }

    /// Update the rate limit configuration
    pub fn update_config(&mut self, config: RateLimitConfig) {
        self.config = config;
    }

    /// Update the maximum concurrent requests
    pub fn update_max_concurrent(&mut self, max: usize) {
        // Note: Semaphore doesn't support dynamic resizing, so we create a new one
        self.request_semaphore = Arc::new(Semaphore::new(max));
    }
}

/// A permit that allows making a rate-limited request
pub struct RateLimitPermit {
    permit: Option<tokio::sync::SemaphorePermit<'static>>,
}

impl RateLimitPermit {
    /// Create a dummy permit (for testing)
    pub fn dummy() -> Self {
        Self { permit: None }
    }
}

impl Drop for RateLimitPermit {
    fn drop(&mut self) {
        // Permit is automatically released when dropped
    }
}

/// Builder for creating rate limiters
pub struct RateLimiterBuilder {
    rpm: Option<u32>,
    min_interval_ms: Option<u64>,
    max_interval_ms: Option<u64>,
    use_random_interval: bool,
    max_concurrent: usize,
}

impl RateLimiterBuilder {
    pub fn new() -> Self {
        Self {
            rpm: None,
            min_interval_ms: None,
            max_interval_ms: None,
            use_random_interval: false,
            max_concurrent: 10,
        }
    }

    /// Set the requests per minute
    pub fn rpm(mut self, rpm: u32) -> Self {
        self.rpm = Some(rpm);
        self
    }

    /// Set the minimum interval between requests in milliseconds
    pub fn min_interval_ms(mut self, ms: u64) -> Self {
        self.min_interval_ms = Some(ms);
        self
    }

    /// Set the maximum interval between requests in milliseconds
    pub fn max_interval_ms(mut self, ms: u64) -> Self {
        self.max_interval_ms = Some(ms);
        self
    }

    /// Enable random interval within the min/max range
    pub fn random_interval(mut self, enabled: bool) -> Self {
        self.use_random_interval = enabled;
        self
    }

    /// Set the maximum concurrent requests
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Build the rate limiter
    pub fn build(self) -> RateLimiter {
        let config = RateLimitConfig {
            rpm: self.rpm,
            min_interval_ms: self.min_interval_ms,
            max_interval_ms: self.max_interval_ms,
            use_random_interval: self.use_random_interval,
        };
        RateLimiter::new(config, self.max_concurrent)
    }
}

impl Default for RateLimiterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiterBuilder::new()
            .rpm(60)
            .max_concurrent(5)
            .build();

        // Should be able to acquire immediately
        let permit = limiter.acquire().await.unwrap();
        drop(permit);

        // Check RPM
        assert!(limiter.current_rpm() > 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_concurrency() {
        let limiter = RateLimiterBuilder::new()
            .rpm(1000) // High RPM to not block
            .max_concurrent(2)
            .build();

        // Acquire two permits
        let p1 = limiter.acquire().await.unwrap();
        let p2 = limiter.acquire().await.unwrap();

        // Should have no available slots
        assert_eq!(limiter.available_slots(), 0);

        drop(p1);
        drop(p2);

        // Should have slots available again
        assert!(limiter.available_slots() > 0);
    }

    #[test]
    fn test_rate_limiter_builder() {
        let limiter = RateLimiterBuilder::new()
            .rpm(100)
            .min_interval_ms(100)
            .max_interval_ms(500)
            .random_interval(true)
            .max_concurrent(20)
            .build();

        assert_eq!(limiter.config.rpm, Some(100));
        assert_eq!(limiter.config.min_interval_ms, Some(100));
        assert_eq!(limiter.config.max_interval_ms, Some(500));
        assert!(limiter.config.use_random_interval);
    }
}
