use tokio::sync::mpsc;
use anyhow::Result;
use tracing::{info, warn, debug};
use crate::worker::{Worker, ExecutionTask};
use crate::ledger::LedgerManager;
use aether_traits::{Target, AttackResult};
use std::sync::Arc;
use std::collections::HashMap;

pub struct EngineCore {
    // Command bus for system-wide control signals
    _command_tx: mpsc::Sender<String>,
    command_rx: mpsc::Receiver<String>,
    // Telemetry pipeline: Decoupled ring buffer
    metrics_tx: flume::Sender<AttackResult>,
    // Worker pool: Channels for tasking the swarm
    worker_txs: Vec<mpsc::Sender<ExecutionTask>>,
    // Persistence layer for audit logging and limit enforcement
    _ledger: Arc<LedgerManager>,
}

impl EngineCore {
    pub async fn new(db_path: &str) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::channel(100);
        let ledger = Arc::new(LedgerManager::new(db_path, "secret_key").await?);
        
        // [Directive 4] Bounded ring buffer for telemetry
        // Capacity of 10k ensures we handle bursts while try_send prevents backpressure.
        let (metrics_tx, metrics_rx) = flume::bounded::<AttackResult>(10000);

        // Spawn Dedicated Telemetry Actor (OS Thread)
        std::thread::Builder::new()
            .name("TelemetryTx".into())
            .spawn(move || {
                info!("TelemetryTx Thread Online. Processing swarm intelligence...");
                while let Ok(metrics) = metrics_rx.recv() {
                    // Logic for batching and reporting to C2/Dashboard
                    // For now, we log to file (non-blocking tracing already configured)
                    debug!("TELEMETRY_RX: Code: {} | Lat: {}us", metrics.status_code, metrics.latency_us);
                }
            })?;

        Ok(Self {
            _command_tx: command_tx,
            command_rx,
            metrics_tx,
            worker_txs: Vec::new(),
            _ledger: ledger,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Aether-Red Engine started. Initializing Swarm...");

        // Spawn Workers (Scalable adversarial swarm)
        for id in 0..5 {
            let (tx, rx) = mpsc::channel(100);
            let metrics_tx = self.metrics_tx.clone();
            self.worker_txs.push(tx);
            
            // Spawn each worker in an isolated async task
            tokio::spawn(async move {
                match Worker::new(id, rx, metrics_tx, "smooth") {
                    Ok(mut worker) => {
                        worker.run().await;
                    }
                    Err(e) => {
                        warn!("Failed to initialize Worker {}: {}", id, e);
                    }
                }
            });
        }
        
        info!("Aether-Red Swarm initialized with 5 workers.");

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

                        let task = ExecutionTask {
                            target,
                            payload_template: "{\"data\": \"base_buffer\"}".to_string(),
                            // CRITICAL FIX #3: Force HTTP/1.1 for text-based attacks (JSON, smuggling)
                            force_http1: true,
                        };
                        
                        // Round-robin worker tasking
                        if let Some(tx) = self.worker_txs.get(0) {
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
