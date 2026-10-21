//! DTOs for the agent's HTTP API and its `/events` SSE stream.

use nanoserde::{DeJson, SerJson};

use crate::build::LocalBuild;

/// One entry of `GET /versions`: the same shape vcl builds from a GitHub release
/// (view/pages/builds/types.ts `Version`), so both lists install the same way.
#[derive(Debug, Clone, SerJson)]
pub struct Version {
    pub id: String,
    pub version: String,
    /// Absolute URL of `/versions/{id}/download` on this agent (needs the bearer token).
    pub source_url: String,
    /// Always `"local"`; GitHub entries are `"remote"`.
    pub content_type: String,
    /// ISO 8601 UTC, like GitHub's `published_at`.
    pub timestamp: String,
    /// The build config (`Debug` / `Release`); a local zip has no git branch.
    pub branch: String,
    /// `sha256:<hex>`, GitHub's asset digest format.
    pub digest: String,
    pub size: u64,
}

impl Version {
    /// `base_url` is how the client reached this agent, e.g. `http://192.168.1.94:9787`.
    pub fn from_build(build: &LocalBuild, base_url: &str) -> Self {
        Self {
            id: build.id.clone(),
            version: build.version.clone(),
            source_url: format!("{base_url}/versions/{}/download", build.id),
            content_type: "local".into(),
            timestamp: iso_utc(build.mtime),
            branch: build.config.clone(),
            digest: format!("sha256:{}", build.sha256),
            size: build.size,
        }
    }
}

/// Unix seconds → `YYYY-MM-DDTHH:MM:SSZ` (days-to-civil from Howard Hinnant's date algorithms).
pub fn iso_utc(unix_secs: i64) -> String {
    let (days, secs) = (unix_secs.div_euclid(86_400), unix_secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

#[derive(Debug, Clone, SerJson, DeJson)]
pub struct OutboxItem {
    pub name: String,
    pub size: u64,
    /// Unix seconds.
    pub staged_at: i64,
}

#[derive(Debug, Clone, SerJson)]
pub struct OutboxResponse {
    pub items: Vec<OutboxItem>,
}

#[derive(Debug, Clone, SerJson, DeJson)]
pub struct StatusResponse {
    pub bind: String,
    pub connected_guis: usize,
    pub pending_outbox: usize,
    pub watched_dirs: Vec<String>,
    pub builds: usize,
}

/// Server-sent events on `GET /events`.
#[derive(Debug, Clone)]
pub enum Event {
    /// A build zip appeared or changed in a watched directory.
    Build { build: LocalBuild },
    /// A file was staged into the outbox (`vct-agent send`).
    File { item: OutboxItem },
    /// Keep-alive; carries the current outbox depth.
    Ping { pending_outbox: usize },
}

impl Event {
    /// Tagged by a `type` field, e.g. `{"type":"ping","pending_outbox":0}`. Written by hand
    /// because nanoserde has no internally tagged enums.
    pub fn to_json(&self) -> String {
        match self {
            Event::Build { build } => format!(r#"{{"type":"build","build":{}}}"#, build.serialize_json()),
            Event::File { item } => format!(r#"{{"type":"file","item":{}}}"#, item.serialize_json()),
            Event::Ping { pending_outbox } => format!(r#"{{"type":"ping","pending_outbox":{pending_outbox}}}"#),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_utc_matches_reference() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso_utc(1_792_611_494), "2026-10-21T19:38:14Z");
        assert_eq!(iso_utc(4_102_444_799), "2099-12-31T23:59:59Z");
    }

    #[test]
    fn version_matches_vcl_shape() {
        let build = LocalBuild {
            id: "abc".into(),
            file_name: "VoidCrewTerminus-0.0.23.zip".into(),
            version: "0.0.23".into(),
            dependencies: vec![],
            config: "Release".into(),
            size: 169,
            mtime: 1_792_611_494,
            sha256: "abc".into(),
            path: "/x.zip".into(),
        };
        let json = Version::from_build(&build, "http://10.0.0.2:9787").serialize_json();
        assert_eq!(
            json,
            r#"{"id":"abc","version":"0.0.23","source_url":"http://10.0.0.2:9787/versions/abc/download","content_type":"local","timestamp":"2026-10-21T19:38:14Z","branch":"Release","digest":"sha256:abc","size":169}"#
        );
    }
}
