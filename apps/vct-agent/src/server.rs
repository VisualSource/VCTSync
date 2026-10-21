//! The agent's HTTP surface. Every route requires `Authorization: Bearer <token>`.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::atomic::Ordering;

use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
    routing::{get, post},
};
use nanoserde::SerJson;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::protocol::{Event, OutboxResponse, StatusResponse, Version};
use crate::state::SharedState;

pub fn app(state: SharedState) -> Router {
    let max_upload = state.cfg.max_upload_bytes as usize;
    Router::new()
        .route("/status", get(status))
        .route("/versions", get(builds))
        .route("/versions/{id}/download", get(build_download))
        .route(
            "/files",
            post(file_upload).layer(DefaultBodyLimit::max(max_upload)),
        )
        .route("/outbox", get(outbox_list))
        .route("/outbox/{name}", get(outbox_get).delete(outbox_delete))
        .route("/events", get(events))
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state)
}

/// A JSON response from a nanoserde type (axum's `Json` needs serde).
fn json(body: &impl SerJson) -> Response {
    ([(header::CONTENT_TYPE, "application/json")], body.serialize_json()).into_response()
}

// ---- auth ----------------------------------------------------------------

async fn auth(
    State(state): State<SharedState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let ok = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.trim() == state.cfg.token)
        .unwrap_or(false);

    if ok {
        next.run(request).await
    } else {
        (StatusCode::UNAUTHORIZED, "missing or bad bearer token").into_response()
    }
}

// ---- handlers ----------------------------------------------------------------

async fn status(State(state): State<SharedState>) -> Response {
    let builds = state.builds.read().await.len();
    json(&StatusResponse {
        bind: state.cfg.bind.clone(),
        connected_guis: state.gui_count(),
        pending_outbox: outbox::list(&state.cfg.outbox_dir()).len(),
        watched_dirs: state.cfg.scan_dirs.clone(),
        builds,
    })
}

/// The builds in vcl's `Version` shape, download links pointing back at the host the client used.
async fn builds(State(state): State<SharedState>, headers: HeaderMap) -> Response {
    let host = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or(&state.cfg.bind);
    let base_url = format!("http://{host}");
    let versions: Vec<Version> = state
        .builds
        .read()
        .await
        .iter()
        .map(|b| Version::from_build(b, &base_url))
        .collect();
    json(&versions)
}

async fn build_download(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let path = {
        let guard = state.builds.read().await;
        guard.iter().find(|b| b.id == id).map(|b| b.path.clone())
    };
    let path = path.ok_or(StatusCode::NOT_FOUND)?;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "build.zip".into());
    Ok((
        [
            (header::CONTENT_TYPE, "application/zip".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        bytes,
    )
        .into_response())
}

async fn file_upload(
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let raw = query
        .get("name")
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing ?name=".to_string()))?;
    let name = safe_name(raw)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("unsafe name: {raw:?}")))?;
    let incoming = state.cfg.incoming_dir();
    tokio::fs::create_dir_all(&incoming)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let dest = incoming.join(&name);
    tokio::fs::write(&dest, &body)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    tracing::info!("received {} ({} bytes)", dest.display(), body.len());
    Ok(StatusCode::CREATED)
}

async fn outbox_list(State(state): State<SharedState>) -> Response {
    json(&OutboxResponse {
        items: outbox::list(&state.cfg.outbox_dir()),
    })
}

async fn outbox_get(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    let name = safe_name(&name).ok_or(StatusCode::BAD_REQUEST)?;
    let path = state.cfg.outbox_dir().join(&name);
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/octet-stream".to_string(),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{name}\""),
            ),
        ],
        bytes,
    )
        .into_response())
}

async fn outbox_delete(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> StatusCode {
    let Some(name) = safe_name(&name) else {
        return StatusCode::BAD_REQUEST;
    };
    match tokio::fs::remove_file(state.cfg.outbox_dir().join(&name)).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

async fn events(
    State(state): State<SharedState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<SseEvent, Infallible>>> {
    state.guis.fetch_add(1, Ordering::Relaxed);
    let guard = GuiGuard(state.clone());
    let pending = outbox::list(&state.cfg.outbox_dir()).len();

    let live = BroadcastStream::new(state.events.subscribe()).filter_map(move |msg| {
        let _hold = &guard;
        msg.ok().map(|ev| Ok(SseEvent::default().data(ev.to_json())))
    });
    let hello = tokio_stream::once(Ok(SseEvent::default().data(
        Event::Ping {
            pending_outbox: pending,
        }
        .to_json(),
    )));

    Sse::new(hello.chain(live)).keep_alive(KeepAlive::default())
}

struct GuiGuard(SharedState);
impl Drop for GuiGuard {
    fn drop(&mut self) {
        self.0.guis.fetch_sub(1, Ordering::Relaxed);
    }
}

// ---- helpers ---------------------------------------------------------------

/// Reduce to a single path component and reject anything with separators, `..`,
/// or that is empty / all-dots.
pub fn safe_name(raw: &str) -> Option<String> {
    let name = std::path::Path::new(raw)
        .file_name()
        .and_then(|n| n.to_str())?
        .to_string();
    if name.is_empty()
        || name == ".."
        || name == "."
        || name.contains(['/', '\\'])
        || name != raw
    {
        return None;
    }
    Some(name)
}

pub mod outbox {
    use std::path::Path;

    use crate::protocol::OutboxItem;

    /// Files directly in `dir`, newest first. Missing dir → empty.
    pub fn list(dir: &Path) -> Vec<OutboxItem> {
        let mut items: Vec<OutboxItem> = match std::fs::read_dir(dir) {
            Ok(rd) => rd
                .flatten()
                .filter(|e| e.path().is_file())
                .filter_map(|e| {
                    let meta = e.metadata().ok()?;
                    let staged_at = meta
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    Some(OutboxItem {
                        name: e.file_name().to_string_lossy().into_owned(),
                        size: meta.len(),
                        staged_at,
                    })
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        items.sort_by(|a, b| b.staged_at.cmp(&a.staged_at));
        items
    }
}

#[cfg(test)]
mod tests {
    use super::safe_name;

    #[test]
    fn safe_name_rejects_traversal() {
        assert_eq!(safe_name("build.zip").as_deref(), Some("build.zip"));
        for bad in ["", ".", "..", "../x", "a/b", "a\\b", "/etc/passwd"] {
            assert_eq!(safe_name(bad), None, "{bad:?}");
        }
    }
}
