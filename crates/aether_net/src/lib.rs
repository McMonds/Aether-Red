pub mod proxy;
pub mod tls;
pub mod doh;

use anyhow::Result;
use proxy::ProxyManager;
use std::sync::Arc;
use tokio::sync::OnceCell;

// Global singleton for the Network Layer
pub static PROXY_MANAGER: OnceCell<Arc<ProxyManager>> = OnceCell::const_new();

pub async fn init_network() -> Result<()> {
    tracing::info!("Initializing Network Layer...");
    
    // Load proxies from config/file (Mock list for now)
    let proxies = vec![
        "socks5://127.0.0.1:9050".to_string(), // Tor default
        "socks5://127.0.0.1:1080".to_string(),
    ];

    let manager = Arc::new(ProxyManager::new(proxies));
    
    // Start Sidecar
    manager.start_health_check().await;
    
    PROXY_MANAGER.set(manager).map_err(|_| anyhow::anyhow!("Network already initialized"))?;

    Ok(())
}
