use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Proxy {
    pub address: String,
    pub protocol: String, // "socks5"
    pub latency: u64,
    pub is_alive: bool,
}

pub struct ProxyManager {
    proxies: Arc<RwLock<Vec<Proxy>>>,
}

impl ProxyManager {
    pub fn new(proxy_list: Vec<String>) -> Self {
        let proxies = proxy_list.into_iter().map(|addr| Proxy {
            address: addr,
            protocol: "socks5".to_string(),
            latency: 0,
            is_alive: true,
        }).collect();

        Self {
            proxies: Arc::new(RwLock::new(proxies)),
        }
    }

    /// The "Sidecar" thread that continuously checks proxy health
    pub async fn start_health_check(&self) {
        let proxies = self.proxies.clone();
        tokio::spawn(async move {
            loop {
                info!("Running Proxy Health Check...");
                let mut list = proxies.write().await;
                for proxy in list.iter_mut() {
                    // Simulating a check. In real impl, connect to Google/Cloudflare.
                    // If timeout or error -> proxy.is_alive = false;
                    proxy.is_alive = true; 
                }
                drop(list);
                sleep(Duration::from_secs(60)).await;
            }
        });
    }

    pub async fn checkout_proxy(&self) -> Option<Proxy> {
        let list = self.proxies.read().await;
        list.iter().find(|p| p.is_alive).cloned()
    }
}
