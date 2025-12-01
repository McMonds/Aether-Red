use serde::Deserialize;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use notify::{Watcher, RecursiveMode, Event};
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub max_workers: usize,
    pub max_daily_per_sender: i64,
    pub proxy_check_interval_secs: u64,
    pub traffic_strategy: String, // "smooth" or "stealth"
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_workers: 5,
            max_daily_per_sender: 500,
            proxy_check_interval_secs: 30,
            traffic_strategy: "smooth".to_string(),
        }
    }
}

pub struct ConfigWatcher {
    config: Arc<RwLock<Config>>,
}

impl ConfigWatcher {
    pub fn new(config_path: &str) -> Result<Self> {
        let config = if Path::new(config_path).exists() {
            let content = std::fs::read_to_string(config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            info!("Config file not found, using defaults");
            Config::default()
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }

    pub async fn start_watching(&self, config_path: String) {
        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            }).expect("Failed to create watcher");

            watcher.watch(Path::new(&config_path), RecursiveMode::NonRecursive)
                .expect("Failed to watch config file");

            info!("ConfigWatcher started for {}", config_path);

            for event in rx {
                if event.kind.is_modify() {
                    info!("Config file changed, reloading...");
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        if let Ok(new_config) = serde_json::from_str::<Config>(&content) {
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                let mut cfg = config.write().await;
                                *cfg = new_config;
                                info!("Config reloaded successfully");
                            });
                        } else {
                            warn!("Failed to parse config file");
                        }
                    }
                }
            }
        });
    }

    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }
}
