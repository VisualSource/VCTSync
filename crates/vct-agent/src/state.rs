use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::{RwLock, broadcast};
use vct_core::build::LocalBuild;
use vct_core::config::AgentConfig;
use vct_core::protocol::Event;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub cfg: AgentConfig,
    /// Current scan result, refreshed by the filesystem watcher.
    pub builds: RwLock<Vec<LocalBuild>>,
    /// Fan-out for the SSE stream.
    pub events: broadcast::Sender<Event>,
    /// Number of connected `/events` subscribers.
    pub guis: AtomicUsize,
}

impl AppState {
    pub fn new(cfg: AgentConfig) -> SharedState {
        let (events, _) = broadcast::channel(256);
        Arc::new(AppState {
            cfg,
            builds: RwLock::new(Vec::new()),
            events,
            guis: AtomicUsize::new(0),
        })
    }

    pub fn gui_count(&self) -> usize {
        self.guis.load(Ordering::Relaxed)
    }
}
