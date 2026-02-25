use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

struct Bucket {
    tokens: f64,
    max_tokens: f64,
    last_refill: Instant,
    refill_rate: f64, // tokens per second
}

impl Bucket {
    fn new(max_per_minute: u32) -> Self {
        let max = max_per_minute as f64;
        Self {
            tokens: max,
            max_tokens: max,
            last_refill: Instant::now(),
            refill_rate: max / 60.0,
        }
    }

    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    default_max_per_minute: u32,
}

impl RateLimiter {
    pub fn new(default_max_per_minute: u32) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            default_max_per_minute,
        }
    }

    /// Check if a request to `path` is allowed. Returns true if allowed.
    pub fn check(&self, path: &str, max_per_minute: Option<u32>) -> bool {
        let max = max_per_minute.unwrap_or(self.default_max_per_minute);
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        let bucket = buckets
            .entry(path.to_string())
            .or_insert_with(|| Bucket::new(max));
        bucket.try_consume()
    }

    /// Remove a path's bucket (when trigger is disarmed).
    pub fn remove(&self, path: &str) {
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        buckets.remove(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_within_limit() {
        let limiter = RateLimiter::new(60);
        // 60 requests should all pass
        for _ in 0..60 {
            assert!(limiter.check("/test", None));
        }
    }

    #[test]
    fn test_over_limit() {
        let limiter = RateLimiter::new(5);
        for _ in 0..5 {
            assert!(limiter.check("/test", None));
        }
        // 6th should fail
        assert!(!limiter.check("/test", None));
    }

    #[test]
    fn test_custom_limit_per_path() {
        let limiter = RateLimiter::new(60);
        // Path with custom limit of 2
        assert!(limiter.check("/strict", Some(2)));
        assert!(limiter.check("/strict", Some(2)));
        assert!(!limiter.check("/strict", Some(2)));
        // Different path still has capacity
        assert!(limiter.check("/other", None));
    }
}
