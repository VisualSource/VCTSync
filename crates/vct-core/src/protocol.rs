//! DTOs for the agent's HTTP API and its `/events` SSE stream. Kept free of
//! `axum`/`iced` so both ends share one definition.

use serde::{Deserialize, Serialize};

use crate::build::LocalBuild;

pub const AUTH_SCHEME: &str = "Bearer";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildsResponse {
    pub builds: Vec<LocalBuild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxItem {
    pub name: String,
    pub size: u64,
    /// Unix seconds.
    pub staged_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxResponse {
    pub items: Vec<OutboxItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub bind: String,
    pub connected_guis: usize,
    pub pending_outbox: usize,
    pub watched_dirs: Vec<String>,
    pub builds: usize,
}

/// Server-sent events on `GET /events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// A build zip appeared or changed in a watched directory.
    Build { build: LocalBuild },
    /// A file was staged into the outbox (`vct-agent send`).
    File { item: OutboxItem },
    /// Keep-alive; carries the current outbox depth.
    Ping { pending_outbox: usize },
}
