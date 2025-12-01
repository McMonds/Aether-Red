pub mod engine;
pub mod ledger;
pub mod config;
pub mod crypto;
pub mod worker;

use anyhow::Result;

pub async fn init_core() -> Result<()> {
    tracing::info!("Initializing Core System...");
    Ok(())
}
