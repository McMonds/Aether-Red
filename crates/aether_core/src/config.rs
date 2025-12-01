use serde::Deserialize;
use tokio::sync::RwLock;
use std::sync::Arc;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub max_threads: usize,
    pub target_latency_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_threads: 50,
            target_latency_ms: 300,
        }
    }
}

pub struct ConfigWatcher {
    config: Arc<RwLock<Config>>,
}

impl ConfigWatcher {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::default())),
        }
    }

    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn start_watching(&self) -> Result<()> {
        // Placeholder for file watching logic (inotify)
        // In a real impl, this would update self.config when config.toml changes
        Ok(())
    }
}
