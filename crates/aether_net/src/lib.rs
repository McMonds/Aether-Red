pub mod proxy;
pub mod tls;
pub mod doh;

use anyhow::Result;

pub async fn init_network() -> Result<()> {
    tracing::info!("Initializing Network Layer...");
    // TODO: Initialize DoH client and Proxy Manager
    Ok(())
}
