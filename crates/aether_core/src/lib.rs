pub mod engine;
pub mod ledger;
pub mod config;
pub mod crypto;

use anyhow::Result;

pub async fn init_core() -> Result<()> {
    tracing::info!("Initializing Core System...");
    Ok(())
}
