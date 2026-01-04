pub mod engine;
pub mod ledger;
pub mod config;
pub mod crypto;
pub mod worker;

use anyhow::Result;
use engine::EngineCore;
use aether_traits::SharedState;
use std::sync::Arc;

/// Initializes the Aether-Red core engine and returns a reference to the shared state.
/// This shared state is the lock-free backbone for high-velocity telemetry visualization.
pub async fn init_core() -> Result<Arc<SharedState>> {
    tracing::info!("Initializing Core System...");
    
    // [Directive: Dynamic Swarm Scaling]
    let num_workers = 5; 
    
    // Initialize Engine with DB path and worker count
    let mut engine = EngineCore::new("sqlite:data/aether.db", num_workers).await?;
    let shared_state = engine.get_shared_state();
    
    // Run engine in background (Fire-and-forget orchestration)
    tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            tracing::error!("Engine crashed: {}", e);
        }
    });

    Ok(shared_state)
}
