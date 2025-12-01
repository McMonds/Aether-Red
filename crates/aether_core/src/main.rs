use aether_core::init_core;
use aether_net::init_network;
use aether_tui::run_tui;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (File based, since TUI takes stdout)
    let file_appender = tracing_appender::rolling::daily("logs", "aether.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();

    tracing::info!("Booting Project ÆTHER...");

    // Initialize subsystems
    init_network().await?;
    init_core().await?;

    // Run TUI (Blocking)
    run_tui().await?;

    tracing::info!("Shutdown complete.");
    Ok(())
}
