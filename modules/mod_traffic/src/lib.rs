use aether_traits::{TrafficStrategy, TrafficMetrics};
use async_trait::async_trait;
use rand::Rng;


/// Normal operation strategy: Consistent, low-jitter delays.
pub struct SmoothFlow;

#[async_trait]
impl TrafficStrategy for SmoothFlow {
    async fn calculate_delay(&self, metrics: &TrafficMetrics) -> u64 {
        let mut rng = rand::thread_rng();
        // Base 300ms + small jitter (0-50ms)
        // Adjusts slightly based on latency
        let base = 300;
        let jitter = rng.gen_range(0..50);
        
        // If latency is high (>500ms), add backoff
        let backoff = if metrics.latency_ms > 500 {
            metrics.latency_ms / 2
        } else {
            0
        };

        base + jitter + backoff
    }

    fn name(&self) -> &str {
        "SmoothFlow"
    }
}

/// Defensive strategy: High variance, long delays to mimic human behavior under scrutiny.
pub struct StealthJitter;

#[async_trait]
impl TrafficStrategy for StealthJitter {
    async fn calculate_delay(&self, metrics: &TrafficMetrics) -> u64 {
        let mut rng = rand::thread_rng();
        // Base 2000ms (2s) + massive jitter (0-5000ms)
        let base = 2000;
        let jitter = rng.gen_range(0..5000);
        
        // Aggressive backoff on errors
        let error_penalty = if metrics.error_rate > 0.1 {
            5000 // Add 5s if error rate > 10%
        } else {
            0
        };

        base + jitter + error_penalty
    }

    fn name(&self) -> &str {
        "StealthJitter"
    }
}
