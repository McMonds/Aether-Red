use async_trait::async_trait;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficMetrics {
    pub latency_ms: u64,
    pub error_rate: f64,
}

#[async_trait]
pub trait TrafficStrategy: Send + Sync {
    /// Calculate the delay before the next action based on metrics
    async fn calculate_delay(&self, metrics: &TrafficMetrics) -> u64;
    
    /// Name of the strategy (e.g., "SmoothFlow", "StealthJitter")
    fn name(&self) -> &str;
}

#[async_trait]
pub trait ContentGenerator: Send + Sync {
    /// Generate the subject and body for an email
    async fn generate_content(&self, context: &str) -> Result<(String, String)>;
}

#[async_trait]
pub trait AccountResurrector: Send + Sync {
    /// Attempt to unlock a dead account (e.g., solve captcha)
    async fn attempt_resurrection(&self, email: &str, password: &str) -> Result<bool>;
}
