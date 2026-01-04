use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;
use bytes::BytesMut;

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
    
    /// Optional synchronization point for race-condition triggering.
    async fn wait(&self) {}

    /// Returns the semantic identifier of the strategy.
    fn name(&self) -> &str;
}

/// Handles the generation of adversarial payloads for stress testing.
/// 
/// CRITICAL FIX: "Bring Your Own Buffer" pattern to eliminate heap thrashing.
/// The caller (Worker) owns the buffer and passes a mutable reference.
/// The fuzzer clears and reuses the same memory allocation for the entire simulation.
#[async_trait]
pub trait PayloadFuzzer: Send + Sync {
    /// Produces a fuzzed payload by writing into the provided buffer.
    /// 
    /// The buffer is cleared at the start and reused across calls.
    /// This eliminates 500,000+ allocations/sec under high load.
    fn generate_into(&self, buffer: &mut BytesMut, template: &str);
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
    async fn execute(&self, target: &Target, payload: Option<bytes::Bytes>) -> anyhow::Result<AttackResult>;
}
