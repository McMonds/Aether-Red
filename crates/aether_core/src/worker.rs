use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use std::sync::Arc;
use aether_traits::{ContentGenerator, TrafficStrategy, TrafficMetrics};
use mod_content::SpintaxGenerator;
use mod_traffic::{SmoothFlow, StealthJitter};

#[derive(Debug, Clone)]
pub struct TaskPackage {
    pub target_email: String,
    pub sender_email: String,
    // In real impl, would contain proxy info, SMTP creds, etc.
}

pub struct Worker {
    id: usize,
    receiver: mpsc::Receiver<TaskPackage>,
    // Traits for pluggable logic
    content_gen: Arc<dyn ContentGenerator>,
    traffic_strategy: Arc<dyn TrafficStrategy>,
}

impl Worker {
    pub fn new(
        id: usize, 
        receiver: mpsc::Receiver<TaskPackage>,
        strategy_type: &str
    ) -> Self {
        // Select strategy based on config
        let traffic_strategy: Arc<dyn TrafficStrategy> = if strategy_type == "stealth" {
            Arc::new(StealthJitter)
        } else {
            Arc::new(SmoothFlow)
        };

        Self {
            id,
            receiver,
            content_gen: Arc::new(SpintaxGenerator),
            traffic_strategy,
        }
    }

    pub async fn run(&mut self) {
        info!("Worker {} started.", self.id);

        while let Some(task) = self.receiver.recv().await {
            debug!("Worker {} received task for {}", self.id, task.target_email);
            
            if let Err(e) = self.execute_task(task).await {
                error!("Worker {} failed task: {}", self.id, e);
                // In a real actor model, we might report this back to the supervisor
            }
        }

        info!("Worker {} shutting down.", self.id);
    }

    async fn execute_task(&self, task: TaskPackage) -> Result<()> {
        // 1. Calculate Delay (Traffic Shaping)
        let metrics = TrafficMetrics { latency_ms: 100, error_rate: 0.0 }; // Mock metrics
        let delay = self.traffic_strategy.calculate_delay(&metrics).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

        // 2. Generate Content (Obfuscation)
        let (subject, body) = self.content_gen.generate_content("Exclusive Offer").await?;
        debug!("Generated content: Subject='{}'", subject);

        // 3. Execute Network Action (Mock SMTP)
        // In real impl, this calls aether_net::smtp::send(...)
        // For now, we just log it.
        info!(
            "Worker {} SENT email to {} | Subject: {} | Delay: {}ms", 
            self.id, task.target_email, subject, delay
        );

        Ok(())
    }
}
