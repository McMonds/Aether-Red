use aether_core::init_core;
use aether_net::init_network;
use aether_tui::run_tui;
use tracing::{info, warn};
use rlimit::{setrlimit, Resource};
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;
use std::io;
use std::panic;

/// [Directive: Ghost Terminal Fix]
/// Custom panic hook to ensure the terminal is restored to a usable state 
/// before the panic info is printed to stderr.
fn setup_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Force cleanup of TUI/Raw mode
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        
        // Pass control to default hook to print stack trace
        default_hook(panic_info);
    }));
}

fn main() -> anyhow::Result<()> {
    // Initialize panic safety first
    setup_panic_hook();

    // [Directive 1] Resource Limit Elevation (FD Management)
    let (soft, hard) = rlimit::getrlimit(Resource::NOFILE)?;
    if let Err(e) = setrlimit(Resource::NOFILE, hard, hard) {
        eprintln!("Warning: Failed to set RLIMIT_NOFILE to hard limit: {}. Current: {}/{}", e, soft, hard);
    }

    // Initialize the specialized Aether-Red Reactor
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

        // Initialize subsystems
        if let Err(e) = init_network().await {
            warn!("Network subsystem failed to initialize: {}", e);
            return Err(e);
        }

        // Initialize core and retrieve SHARED STATE
        let shared_state = match init_core().await {
            Ok(ss) => ss,
            Err(e) => {
                warn!("Core engine failed to initialize: {}", e);
                return Err(e);
            }
        };

        // Run TUI with Shared State (Blocking)
        // [Directive: Atomic Snapshots]
        if let Err(e) = run_tui(shared_state).await {
            warn!("TUI crashed: {}", e);
            return Err(e);
        }

        info!("Aether-Red Engine shutting down.");
        Ok(())
    })
}
