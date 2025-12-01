pub mod engine;
pub mod ledger;
pub mod config;
pub mod crypto;
pub mod worker;

use anyhow::Result;
use engine::EngineCore;

pub async fn init_core() -> Result<()> {
    tracing::info!("Initializing Core System...");
    // Initialize Engine with DB path
    // In Docker, this maps to /app/data/aether.db
    let mut engine = EngineCore::new("sqlite:data/aether.db").await?;
    
    // Run engine in background
    tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            tracing::error!("Engine crashed: {}", e);
        }
    });

    Ok(())
}
