use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

pub struct Proxy {
    pub address: String,
    pub is_alive: bool,
}

pub struct ProxyManager {
    proxies: Vec<String>,
    current_index: Arc<Mutex<usize>>,
}

impl ProxyManager {
    pub fn new(proxies: Vec<String>) -> Self {
        Self {
            proxies,
            current_index: Arc::new(Mutex::new(0)),
        }
    }

    /// Implements a Round Robin strategy for proxy selection.
    /// This ensures traffic is distributed across all available identities.
    pub async fn get_next_proxy(&self) -> Result<String> {
        if self.proxies.is_empty() {
            return Err(anyhow::anyhow!("Proxy pool is exhausted"));
        }

        let mut idx = self.current_index.lock().await;
        let proxy = self.proxies[*idx].clone();
        
        *idx = (*idx + 1) % self.proxies.len();
        
        Ok(proxy)
    }
}
