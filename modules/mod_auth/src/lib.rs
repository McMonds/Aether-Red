use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::{self, Duration};
use tracing::info;

/// [Directive 8] Sharded-State Identity Manager with OOM Protection.
/// Manages session tokens and adversarial identities using DashMap for lock-free concurrency.
pub struct IdentityManager {
    // Sharded map for high-frequency session access
    sessions: Arc<DashMap<String, String>>,
}

impl IdentityManager {
    pub fn new(max_capacity: usize) -> Arc<Self> {
        let sessions = Arc::new(DashMap::new());
        let manager = Arc::new(Self {
            sessions: sessions.clone(),
        });

        // Spawn background cleanup tick to prevent memory leaks during long simulations
        let cleanup_sessions = sessions.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(300)); // 5-minute cleanup tick
            loop {
                interval.tick().await;
                if cleanup_sessions.len() > max_capacity {
                    warn_capacity_reached(cleanup_sessions.len(), max_capacity);
                    cleanup_sessions.clear(); // Abortive clear to prevent OOM
                    info!("IdentityManager: Cache cleared to protect system memory.");
                }
            }
        });

        manager
    }

    pub fn store_session(&self, id: String, params: String) {
        self.sessions.insert(id, params);
    }

    pub fn get_session(&self, id: &str) -> Option<String> {
        self.sessions.get(id).map(|val| val.value().clone())
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

fn warn_capacity_reached(current: usize, max: usize) {
    tracing::warn!(
        "IdentityManager: OOM Protection Triggered. Current sessions: {} exceeds MAX: {}",
        current, max
    );
}
