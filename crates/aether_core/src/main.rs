use aether_core::init_core;
use aether_net::init_network;
use aether_tui::run_tui;
use tracing::{info, warn};
use rlimit::{setrlimit, Resource};

fn main() -> anyhow::Result<()> {
    // [Directive 1] Resource Limit Elevation (FD Management)
    // Before elevating the runtime, ensure we have enough file descriptors for the swarm.
    let (soft, hard) = rlimit::getrlimit(Resource::NOFILE)?;
    if let Err(e) = setrlimit(Resource::NOFILE, hard, hard) {
        // Fallback or warning if not running as root/privileged
        eprintln!("Warning: Failed to set RLIMIT_NOFILE to hard limit: {}. Current: {}/{}", e, soft, hard);
    }

    // Initialize the specialized Aether-Red Reactor
    // [Directive 2] Explicit Runtime Orchestration
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(num_cpus::get())
        .thread_name("aether-reactor")
        .build()?;

    runtime.block_on(async {
        // Initialize logging (File based, since TUI takes stdout)
        let file_appender = tracing_appender::rolling::daily("logs", "aether.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .init();

        info!("Aether-Red Reactor Online. FD Limit: {}/{}", hard, hard);
        info!("Initializing Adversarial Swarm...");

        // Initialize subsystems
        if let Err(e) = init_network().await {
            warn!("Network subsystem failed to initialize: {}", e);
            return Err(e);
        }

        if let Err(e) = init_core().await {
            warn!("Core engine failed to initialize: {}", e);
            return Err(e);
        }

        // Run TUI (Blocking)
        if let Err(e) = run_tui().await {
            warn!("TUI crashed: {}", e);
            return Err(e);
        }

        info!("Aether-Red Engine shutting down.");
        Ok(())
    })
}
