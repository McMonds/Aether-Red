pub mod proxy;
pub mod tls;
pub mod doh;

use anyhow::Result;
use proxy::ProxyManager;
use std::sync::Arc;
use tokio::sync::OnceCell;
use reqwest::Client;

// Global singleton for the Network Layer
pub static PROXY_MANAGER: OnceCell<Arc<ProxyManager>> = OnceCell::const_new();

pub async fn init_network() -> Result<()> {
    tracing::info!("Initializing Adversarial Network Layer...");
    
    // In a real scenario, these would be loaded from a cryptographically secured config
    let proxies = vec![
        "socks5://127.0.0.1:9050".to_string(), // Tor node
        "socks5://127.0.0.1:1080".to_string(),
    ];

    let manager = Arc::new(ProxyManager::new(proxies));
    
    PROXY_MANAGER.set(manager).map_err(|_| anyhow::anyhow!("Network already initialized"))?;

    Ok(())
}

/// Creates a new HTTP client with a faked browser fingerprint and routed through the proxy pool.
pub async fn create_adversarial_client() -> Result<Client> {
    let manager = PROXY_MANAGER.get().ok_or_else(|| anyhow::anyhow!("Network not initialized"))?;
    let proxy_addr = manager.get_next_proxy().await?;
    
    // Build a client that mimics a modern browser (Chrome 120+ signature)
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .proxy(reqwest::Proxy::all(proxy_addr)?)
        .danger_accept_invalid_certs(true) // Required for some security auditing environments
        .build()?;

    Ok(client)
}
