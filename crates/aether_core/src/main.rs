use aether_core::init_core;
use aether_net::init_network;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("Booting Project ÆTHER...");

    // Initialize subsystems
    init_network().await?;
    init_core().await?;

    tracing::info!("System Ready. Waiting for commands...");
    
    // Keep main thread alive
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received.");

    Ok(())
}
