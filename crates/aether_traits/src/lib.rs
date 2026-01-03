use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;

/// Represents a specific endpoint for security auditing or load testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: "GET".to_string(),
            headers: HashMap::new(),
        }
    }
}

/// Telemetry metrics for traffic generation and load analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrafficMetrics {
    pub latency_us: u128,
    pub error_count: u64,
}

#[async_trait]
pub trait TrafficStrategy: Send + Sync {
    /// Determines the next execution delay (jitter/backoff) based on current metrics.
    async fn next_delay(&self, metrics: &TrafficMetrics) -> Duration;
    
    /// Returns the semantic identifier of the strategy.
    fn name(&self) -> &str;
}

/// Handles the generation of adversarial payloads for stress testing.
#[async_trait]
pub trait PayloadFuzzer: Send + Sync {
    /// Produces a fuzzed payload derived from a base template or sample.
    async fn generate(&self, base_template: &str) -> String;
}

/// Detailed performance telemetry for a single execution cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackResult {
    pub status_code: u16,
    pub latency_us: u128,
    pub size_bytes: usize,
}

#[async_trait]
pub trait NetworkClient: Send + Sync {
    /// Core execution logic for a single adversarial request.
    /// Returns microsecond-resolution telemetry upon completion or failure.
    async fn execute(&self, target: &Target, payload: Option<String>) -> anyhow::Result<AttackResult>;
}
