use tokio::sync::mpsc;
use anyhow::Result;
use tracing::{info, warn, debug};
use crate::worker::{Worker, ExecutionTask, TelemetryPayload};
use crate::ledger::LedgerManager;
use aether_traits::{Target, AttackResult, SharedState};
use aether_net::tls::AttackProfile;
use std::sync::Arc;
use std::collections::HashMap;
use hdrhistogram::Histogram;

pub struct EngineCore {
    // Command bus for system-wide control signals
    _command_tx: mpsc::Sender<String>,
    command_rx: mpsc::Receiver<String>,
    // Telemetry pipeline: Decoupled ring buffer
    telemetry_tx: flume::Sender<TelemetryPayload>,
    // Shared State for TUI Snapshots
    shared_state: Arc<SharedState>,
    // Worker pool: Channels for tasking the swarm
    worker_txs: Vec<mpsc::Sender<ExecutionTask>>,
    // Persistence layer for audit logging and limit enforcement
    _ledger: Arc<LedgerManager>,
    // [Fix 1] Round-Robin dispatch state
    dispatch_index: std::sync::atomic::AtomicUsize,
}

impl EngineCore {
    pub async fn new(db_path: &str, num_workers: usize) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::channel(100);
        let ledger = Arc::new(LedgerManager::new(db_path, "secret_key").await?);
        
        // [Directive: Shared Atomic State]
        let shared_state = SharedState::new(num_workers);

        // [Directive 4] Bounded ring buffer for telemetry
        let (telemetry_tx, telemetry_rx) = flume::bounded::<TelemetryPayload>(10000);

        // [Directive: Telemetry Aggregation] Global Histogram
        let mut global_histogram = Histogram::<u64>::new_with_bounds(1, 10_000_000, 3)?;

        // Spawn Dedicated Telemetry Actor (OS Thread)
        // [Flaw 1 Fix: Merging logic happens here]
        std::thread::Builder::new()
            .name("TelemetryTx".into())
            .spawn(move || {
                info!("TelemetryTx Thread Online. Processing swarm intelligence...");
                while let Ok(payload) = telemetry_rx.recv() {
                    match payload {
                        TelemetryPayload::Histogram(local_hist) => {
                            // Merge local samples into global view
                            let _ = global_histogram.add(&local_hist);
                            debug!("METRICS_SYNC: Latency P99: {}us", global_histogram.value_at_quantile(0.99));
                        }
                        TelemetryPayload::Attack(result) => {
                            // [Flaw 3 Fix: Boundary Sanitizer]
                            // Prevents binary payloads/control chars from corrupting the TUI.
                            let sanitized_msg = sanitize_log(result);
                            debug!("TELEMETRY_LOG: {}", sanitized_msg);
                        }
                    }
                }
            })?;

        Ok(Self {
            _command_tx: command_tx,
            command_rx,
            telemetry_tx,
            shared_state,
            worker_txs: Vec::new(),
            _ledger: ledger,
            dispatch_index: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    pub fn get_shared_state(&self) -> Arc<SharedState> {
        self.shared_state.clone()
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Aether-Red Engine started. Initializing Swarm...");

        let num_workers = self.shared_state.worker_statuses.len();

        // Spawn Workers (Scalable adversarial swarm)
        for id in 0..num_workers {
            let (tx, rx) = mpsc::channel(100);
            let telemetry_tx = self.telemetry_tx.clone();
            let shared_state = self.shared_state.clone();
            self.worker_txs.push(tx);
            
            // Spawn each worker in an isolated async task
            tokio::spawn(async move {
                match Worker::new(id, rx, telemetry_tx, shared_state, "smooth") {
                    Ok(mut worker) => {
                        worker.run().await;
                    }
                    Err(e) => {
                        warn!("Failed to initialize Worker {}: {}", id, e);
                    }
                }
            });
        }
        
        info!("Aether-Red Swarm initialized with {} workers.", num_workers);

        // Central Orchestration Loop
        loop {
            tokio::select! {
                Some(cmd) = self.command_rx.recv() => {
                    info!("Orchestrator received command: {}", cmd);
                    if cmd == "SHUTDOWN" {
                        break;
                    } else if cmd.starts_with("DISPATCH") {
                        // High-performance dispatch logic
                        let target = Arc::new(Target {
                            url: "http://target-system.internal/api/v1".to_string(),
                            method: "POST".to_string(),
                            headers: HashMap::new(),
                        });

                        // [Category C] Orchestrating Advanced Evasion
                        let profile = if cmd.contains("STEALTH") {
                             AttackProfile {
                                 force_http1: true,
                                 use_0rtt: true,
                                 fragment_handshake: true,
                                 ..Default::default()
                             }
                        } else if cmd.contains("LEGACY") {
                             AttackProfile {
                                 force_http1: true,
                                 force_tls11: true,
                                 force_http10: true,
                                 ..Default::default()
                             }
                        } else {
                             AttackProfile {
                                 force_http1: true, // Default to HTTP/1.1 for text fuzzing
                                 ..Default::default()
                             }
                        };

                        let task = ExecutionTask {
                            target,
                            payload_template: "{\"data\": \"base_buffer\"}".to_string(),
                            profile,
                        };
                        
                        // [Fix 1] Round-robin worker tasking
                        let i = self.dispatch_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.worker_txs.len();
                        if let Some(tx) = self.worker_txs.get(i) {
                            let _ = tx.send(task).await;
                        }
                    }
                }
            }
        }
        
        info!("Aether-Red Engine shutting down.");
        Ok(())
    }
}

/// [Flaw 3 Fix: Boundary Sanitizer]
fn sanitize_log(result: AttackResult) -> String {
    let base = format!("Code: {} | Lat: {}us | Size: {}b", 
        result.status_code, result.latency_us, result.size_bytes);
        
    let base_bytes = base.as_bytes();
    let stripped = strip_ansi_escapes::strip(base_bytes);
    let lossy = String::from_utf8_lossy(&stripped);
        
    let mut sanitized = String::with_capacity(lossy.len());
    for c in lossy.chars() {
        if c == '\n' || !c.is_control() {
            sanitized.push(c);
        } else {
            for ec in c.escape_default() {
                sanitized.push(ec);
            }
        }
    }
    
    if sanitized.len() > 128 {
        sanitized.truncate(125);
        sanitized.push_str("...");
    }
    
    sanitized
}
