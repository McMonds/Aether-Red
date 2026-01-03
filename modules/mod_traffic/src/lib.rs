use aether_traits::{TrafficStrategy, TrafficMetrics};
use async_trait::async_trait;
use rand::Rng;
use std::time::Duration;

/// Normal operation strategy: Consistent, low-jitter cadence.
pub struct SmoothFlow;

#[async_trait]
impl TrafficStrategy for SmoothFlow {
    async fn next_delay(&self, metrics: &TrafficMetrics) -> Duration {
        let mut rng = rand::thread_rng();
        
        // Base 300ms + small jitter (0-50ms)
        let base_ms = 300;
        let jitter_ms = rng.gen_range(0..50);
        
        // If microsecond latency is high (>500ms), add backoff
        let backoff_ms = if metrics.latency_us > 500_000 {
            (metrics.latency_us / 2_000) as u64
        } else {
            0
        };

        Duration::from_millis(base_ms + jitter_ms + backoff_ms)
    }

    fn name(&self) -> &str {
        "SmoothFlow"
    }
}

/// Defensive strategy: High variance, long delays to mimic human-like or erratic behavior.
pub struct StealthJitter;

#[async_trait]
impl TrafficStrategy for StealthJitter {
    async fn next_delay(&self, metrics: &TrafficMetrics) -> Duration {
        let mut rng = rand::thread_rng();
        
        // Base 2000ms (2s) + massive jitter (0-5000ms)
        let base_ms = 2000;
        let jitter_ms = rng.gen_range(0..5000);
        
        // Aggressive backoff on recorded execution errors
        let error_penalty_ms = if metrics.error_count > 0 {
            5000 // Add 5s penalty if any errors occurred in the last interval
        } else {
            0
        };

        Duration::from_millis(base_ms + jitter_ms + error_penalty_ms)
    }

    fn name(&self) -> &str {
        "StealthJitter"
    }
}
