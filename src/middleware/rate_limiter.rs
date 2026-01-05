use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;

/// Rate limiter using sliding window algorithm
#[derive(Clone)]
pub struct RateLimiter {
    /// Maximum number of requests allowed in the time window
    max_requests: u32,
    /// Time window in seconds
    window_seconds: i64,
    /// Storage for tracking requests per IP (IP -> list of timestamps)
    storage: Arc<DashMap<String, Vec<i64>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed in the window
    /// * `window_seconds` - Time window in seconds
    pub fn new(max_requests: u32, window_seconds: i64) -> Self {
        Self {
            max_requests,
            window_seconds,
            storage: Arc::new(DashMap::new()),
        }
    }

    /// Check if an IP address has exceeded the rate limit
    ///
    /// # Arguments
    /// * `ip` - IP address to check
    ///
    /// # Returns
    /// * `Ok(())` - Request is allowed
    /// * `Err((StatusCode, String))` - Rate limit exceeded with error message
    pub fn check_rate_limit(&self, ip: &str) -> Result<(), (StatusCode, String)> {
        let now = Utc::now().timestamp();
        let cutoff = now - self.window_seconds;

        // Get or create the entry for this IP
        let mut entry = self.storage.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove timestamps older than the window (sliding window cleanup)
        entry.retain(|&timestamp| timestamp > cutoff);

        // Check if limit is exceeded
        if entry.len() >= self.max_requests as usize {
            let retry_after = entry
                .first()
                .map(|&ts| ts + self.window_seconds - now)
                .unwrap_or(self.window_seconds);

            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                format!(
                    "Rate limit exceeded. Maximum {} requests per {} seconds allowed. Try again in {} seconds.",
                    self.max_requests, self.window_seconds, retry_after
                ),
            ));
        }

        // Add current timestamp
        entry.push(now);

        Ok(())
    }

    /// Middleware function to apply rate limiting to requests
    pub async fn middleware(
        limiter: Arc<RateLimiter>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        req: Request<Body>,
        next: Next,
    ) -> Result<Response, (StatusCode, String)> {
        // LINE 85: Extract IP address from connection info
        let ip = addr.ip().to_string();

        // Get current timestamp for logging
        let now = Utc::now();
        let timestamp = now.timestamp();

        // Console log: IP address and timestamp
        println!(
            "🌐 Request from IP: {} | Time: {} (Unix: {})",
            ip,
            now.format("%Y-%m-%d %H:%M:%S UTC"),
            timestamp
        );

        // Check rate limit
        match limiter.check_rate_limit(&ip) {
            Ok(_) => {
                println!("✅ Rate limit OK for {}", ip);
            }
            Err(ref e) => {
                println!("❌ Rate limit EXCEEDED for {} - {}", ip, e.1);
                return Err(e.clone());
            }
        }

        // If allowed, proceed with the request
        Ok(next.run(req).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(3, 60);

        // First 3 requests should succeed
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(3, 60);

        // First 3 requests should succeed
        for _ in 0..3 {
            assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        }

        // 4th request should fail
        assert!(limiter.check_rate_limit("127.0.0.1").is_err());
    }

    #[test]
    fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(3, 60);

        // Different IPs should have separate limits
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("192.168.1.1").is_ok());
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("192.168.1.1").is_ok());
    }
}
