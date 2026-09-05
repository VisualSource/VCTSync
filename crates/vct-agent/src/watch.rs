//! Filesystem watcher: keeps `state.builds` fresh and emits `build` / `file`
//! SSE events when the scan dirs or the outbox change.

use std::collections::HashSet;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use vct_core::build;
use vct_core::protocol::Event;

use crate::server::outbox;
use crate::state::SharedState;

/// Spawn the initial scan, the OS watcher, and the rescan loop. Never returns a
/// handle — it lives for the process.
pub fn spawn(state: SharedState) {
    tokio::spawn(async move {
        refresh_builds(&state).await;

        // seed the outbox "already announced" set from what's on disk now
        let mut announced: HashSet<String> = outbox::list(&state.cfg.outbox_dir)
            .into_iter()
            .map(|i| i.name)
            .collect();

        let (raw_tx, raw_rx) = std::sync::mpsc::channel::<()>();
        let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if res.is_ok() {
                let _ = raw_tx.send(());
            }
        });
        let mut watcher = match watcher {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("could not create filesystem watcher: {e}");
                return;
            }
        };
        for dir in state
            .cfg
            .scan_dirs
            .iter()
            .chain(std::iter::once(&state.cfg.outbox_dir))
        {
            if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
                tracing::debug!("not watching {} ({e})", dir.display());
            }
        }

        let (wake_tx, mut wake_rx) = tokio::sync::mpsc::channel::<()>(1);
        // Bridge notify's own thread → async, coalescing bursts.
        std::thread::spawn(move || {
            let _watcher = watcher; // keep alive
            while raw_rx.recv().is_ok() {
                std::thread::sleep(Duration::from_millis(300));
                while raw_rx.try_recv().is_ok() {}
                let _ = wake_tx.blocking_send(());
            }
        });

        while wake_rx.recv().await.is_some() {
            refresh_builds(&state).await;
            for item in outbox::list(&state.cfg.outbox_dir) {
                if announced.insert(item.name.clone()) {
                    let _ = state.events.send(Event::File { item });
                }
            }
        }
    });
}

async fn refresh_builds(state: &SharedState) {
    let dirs = state.cfg.scan_dirs.clone();
    let found = match tokio::task::spawn_blocking(move || build::scan(&dirs)).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("build scan task panicked: {e}");
            return;
        }
    };

    let mut guard = state.builds.write().await;
    let old: HashSet<String> = guard.iter().map(|b| b.id.clone()).collect();
    for b in &found {
        if !old.contains(&b.id) {
            tracing::info!("new build: {} ({}, {})", b.version, b.config, b.file_name);
            let _ = state.events.send(Event::Build { build: b.clone() });
        }
    }
    *guard = found;
}
