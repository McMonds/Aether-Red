use tokio::sync::mpsc;
use anyhow::Result;
use tracing::info;
use crate::worker::{Worker, TaskPackage};

pub struct EngineCore {
    // Command bus for system-wide commands
    command_tx: mpsc::Sender<String>,
    command_rx: mpsc::Receiver<String>,
    // Worker pool
    worker_txs: Vec<mpsc::Sender<TaskPackage>>,
}

impl EngineCore {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        Self {
            command_tx,
            command_rx,
            worker_txs: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Engine Core started. Initializing Swarm...");

        // Spawn Workers (e.g., 5 workers)
        for id in 0..5 {
            let (tx, rx) = mpsc::channel(100);
            self.worker_txs.push(tx);
            
            // Spawn worker in a separate task
            tokio::spawn(async move {
                let mut worker = Worker::new(id, rx, "smooth");
                worker.run().await;
            });
        }
        
        info!("Swarm initialized with 5 workers.");

        // Main event loop
        loop {
            tokio::select! {
                Some(cmd) = self.command_rx.recv() => {
                    info!("Received command: {}", cmd);
                    if cmd == "SHUTDOWN" {
                        break;
                    } else if cmd.starts_with("DISPATCH") {
                        // Mock dispatch logic
                        // In real impl, this comes from the Scheduler/Ledger
                        let task = TaskPackage {
                            target_email: "target@example.com".to_string(),
                            sender_email: "sender@example.com".to_string(),
                        };
                        
                        // Round-robin dispatch
                        let worker_idx = 0; // Simplified
                        if let Some(tx) = self.worker_txs.get(worker_idx) {
                            let _ = tx.send(task).await;
                        }
                    }
                }
            }
        }
        
        info!("Engine Core shutting down.");
        Ok(())
    }
}
