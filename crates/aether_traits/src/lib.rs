use bytes::BytesMut;
use std::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;

/// [Directive: Worker Hive] Status enum for thread heatmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkerStatus {
    Idle = 0,
    Handshaking = 1,
    Sending = 2,
    Blocked = 3,
    Dead = 4,
}

/// [Directive: Atomic Snapshots] Lock-free shared state for the C2 Dashboard.
/// Composition of atomic primitives ensures zero lock contention under high-velocity (50k+ RPS).
pub struct SharedState {
    pub total_requests: AtomicU64,
    pub total_bytes: AtomicU64,
    pub error_count: AtomicU64,
    pub worker_statuses: Vec<AtomicU8>,
    pub worker_heartbeats: Vec<AtomicU64>,
    
    // [Directive: Interactive C2] Runtime Tunables (Atomic parameter injection)
    pub target_rps: AtomicUsize,
    pub jitter_factor: AtomicUsize, // Percentage 0-100
}

impl SharedState {
    pub fn new(num_workers: usize) -> Arc<Self> {
        let mut worker_statuses = Vec::with_capacity(num_workers);
        let mut worker_heartbeats = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            worker_statuses.push(AtomicU8::new(WorkerStatus::Idle as u8));
            worker_heartbeats.push(AtomicU64::new(0));
        }
        
        Arc::new(Self {
            total_requests: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            worker_statuses,
            worker_heartbeats,
            target_rps: AtomicUsize::new(1000),
            jitter_factor: AtomicUsize::new(10),
        })
    }
}

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
