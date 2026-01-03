use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use std::sync::Arc;
use std::time::{Duration, Instant};
use reqwest::{Client, Method};
use aether_traits::{Target, TrafficStrategy, TrafficMetrics, AttackResult, PayloadFuzzer};
use mod_traffic::{SmoothFlow, StealthJitter};
use mod_fuzz::PolyglotFuzzer;

/// Instruction set for an adversarial execution cycle.
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    pub target: Arc<Target>,
    pub payload_template: String,
}

pub struct Worker {
    id: usize,
    receiver: mpsc::Receiver<ExecutionTask>,
    client: Client,
    fuzzer: Arc<dyn PayloadFuzzer>,
    traffic_strategy: Arc<dyn TrafficStrategy>,
}

impl Worker {
    pub fn new(
        id: usize, 
        receiver: mpsc::Receiver<ExecutionTask>,
        strategy_type: &str,
    ) -> Result<Self> {
        // Initialize high-performance HTTP client with default 30s timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let traffic_strategy: Arc<dyn TrafficStrategy> = if strategy_type == "stealth" {
            Arc::new(StealthJitter)
        } else {
            Arc::new(SmoothFlow)
        };

        Ok(Self {
            id,
            receiver,
            client,
            fuzzer: Arc::new(PolyglotFuzzer),
            traffic_strategy,
        })
    }

    pub async fn run(&mut self) {
        info!("Worker {} [Adversarial] initialized.", self.id);

        while let Some(task) = self.receiver.recv().await {
            debug!("Worker {} executing attack on {}", self.id, task.target.url);
            
            // Execute and capture telemetry
            match self.execute_attack(task).await {
                Ok(metrics) => {
                    info!(
                        "Worker {} ATTACK SUCCESS | Code: {} | Latency: {}us | Size: {} bytes",
                        self.id, metrics.status_code, metrics.latency_us, metrics.size_bytes
                    );
                }
                Err(e) => {
                    warn!("Worker {} ATTACK FAILED: {}", self.id, e);
                }
            }
        }

        info!("Worker {} shutting down.", self.id);
    }

    async fn execute_attack(&self, task: ExecutionTask) -> Result<AttackResult> {
        // 1. Calculate Jitter/Delay (Adversarial Cadence)
        // Mock metrics for strategy calculation - in production, these are fed back from a central collector
        let current_metrics = TrafficMetrics { latency_us: 0, error_count: 0 }; 
        let delay = self.traffic_strategy.next_delay(&current_metrics).await;
        tokio::time::sleep(delay).await;

        // 2. Payload Fuzzing
        let payload = self.fuzzer.generate(&task.payload_template).await;

        // 3. Network Execution with Telemetry
        let method = Method::from_bytes(task.target.method.as_bytes())?;
        let mut request_builder = self.client.request(method, &task.target.url);

        // Apply adversarial headers
        for (key, value) in &task.target.headers {
            request_builder = request_builder.header(key, value);
        }

        let start = Instant::now();
        let response = request_builder.body(payload).send().await?;
        let latency_us = start.elapsed().as_micros();
        
        let status_code = response.status().as_u16();
        let size_bytes = response.bytes().await?.len();

        Ok(AttackResult {
            status_code,
            latency_us,
            size_bytes,
        })
    }
}
