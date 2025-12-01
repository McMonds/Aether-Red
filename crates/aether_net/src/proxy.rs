use tokio::time::{sleep, Duration};
use tracing::{info, warn, debug};
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::{Client, Proxy as ReqwestProxy};

#[derive(Debug, Clone)]
pub struct Proxy {
    pub address: String, // e.g., "socks5://1.2.3.4:1080"
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
            latency: 0,
            is_alive: true, // Optimistic start
        }).collect();

        Self {
            proxies: Arc::new(RwLock::new(proxies)),
        }
    }

    /// The "Sidecar" thread that continuously checks proxy health
    pub async fn start_health_check(&self) {
        let proxies = self.proxies.clone();
        tokio::spawn(async move {
            info!("Proxy Health Check Sidecar started.");
            loop {
                // Clone the list to avoid holding the lock during network IO
                let targets: Vec<(usize, String)> = {
                    let list = proxies.read().await;
                    list.iter().enumerate()
                        .map(|(i, p)| (i, p.address.clone()))
                        .collect()
                };

                for (idx, addr) in targets {
                    let (alive, latency) = Self::check_proxy(&addr).await;
                    
                    let mut list = proxies.write().await;
                    if let Some(proxy) = list.get_mut(idx) {
                        proxy.is_alive = alive;
                        proxy.latency = latency;
                        if !alive {
                            debug!("Proxy {} is DEAD", addr);
                        }
                    }
                }
                
                sleep(Duration::from_secs(30)).await;
            }
        });
    }

    async fn check_proxy(addr: &str) -> (bool, u64) {
        let start = std::time::Instant::now();
        match ReqwestProxy::all(addr) {
            Ok(proxy) => {
                let client = Client::builder()
                    .proxy(proxy)
                    .timeout(Duration::from_secs(5))
                    .build();

                match client {
                    Ok(c) => {
                        // Try to fetch a reliable target
                        match c.get("https://www.google.com").send().await {
                            Ok(resp) => {
                                if resp.status().is_success() {
                                    (true, start.elapsed().as_millis() as u64)
                                } else {
                                    (false, 0)
                                }
                            }
                            Err(_) => (false, 0),
                        }
                    }
                    Err(_) => (false, 0),
                }
            }
            Err(_) => (false, 0),
        }
    }

    pub async fn checkout_proxy(&self) -> Option<Proxy> {
        let list = self.proxies.read().await;
        // Simple strategy: Return first alive proxy
        // Advanced: Return lowest latency
        list.iter().find(|p| p.is_alive).cloned()
    }
}
