//! Config lives in `config.ron` **next to the executable** (resolved via
//! `current_exe()`, not the working directory), overridable with `--config`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use nanoserde::{DeRon, SerRon};

/// Default TCP port the agent binds to.
pub const DEFAULT_PORT: u16 = 9787;

/// `config.ron` beside the running executable.
pub fn default_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("cannot resolve current executable path")?;
    let dir = exe
        .parent()
        .context("executable path has no parent directory")?;
    Ok(dir.join("config.ron"))
}

pub fn load(path: &Path) -> Result<AgentConfig> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading config {}", path.display()))?;
    AgentConfig::deserialize_ron(&text)
        .map_err(|err| anyhow::anyhow!("parsing config {}: {err}", path.display()))
}

/// 32 random bytes, hex-encoded. The vcl Settings page's dev agent token must match.
pub fn generate_token() -> String {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf).expect("os rng unavailable");
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

/// Paths are strings because nanoserde has no `PathBuf` support; use the accessors.
#[derive(Debug, Clone, DeRon, SerRon)]
pub struct AgentConfig {
    /// Address to bind the HTTP server to.
    #[nserde(default_with = "default_bind")]
    pub bind: String,
    /// Shared bearer token. `Authorization: Bearer <token>` on every request.
    pub token: String,
    /// Directories scanned for local build zips (`VoidCrewTerminus-<ver>.zip`).
    #[nserde(default)]
    pub scan_dirs: Vec<String>,
    /// Where Windows→Linux uploads (`POST /files`) land.
    pub incoming_dir: String,
    /// Where `vct-agent send` stages Linux→Windows files.
    pub outbox_dir: String,
    /// Reject uploads larger than this.
    #[nserde(default_with = "default_max_upload")]
    pub max_upload_bytes: u64,
}

fn default_bind() -> String {
    format!("0.0.0.0:{DEFAULT_PORT}")
}

fn default_max_upload() -> u64 {
    512 * 1024 * 1024
}

impl AgentConfig {
    /// A fresh config with a generated token and paths rooted at `base`.
    pub fn scaffold(base: &Path) -> Self {
        let path = |p: &str| base.join(p).to_string_lossy().into_owned();
        Self {
            bind: default_bind(),
            token: generate_token(),
            scan_dirs: vec![
                path("VoidCrewTerminus/bin/Debug/net472/Releases"),
                path("VoidCrewTerminus/bin/Release/net472/Releases"),
            ],
            incoming_dir: path("vct-sync-incoming"),
            outbox_dir: path("vct-sync-outbox"),
            max_upload_bytes: default_max_upload(),
        }
    }

    /// Fields that must be filled before `run` can serve.
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(!self.token.trim().is_empty(), "config: `token` is empty");
        anyhow::ensure!(
            self.bind.parse::<std::net::SocketAddr>().is_ok(),
            "config: `bind` is not a valid host:port ({:?})",
            self.bind
        );
        Ok(())
    }

    pub fn scan_dirs(&self) -> Vec<PathBuf> {
        self.scan_dirs.iter().map(PathBuf::from).collect()
    }

    pub fn incoming_dir(&self) -> PathBuf {
        PathBuf::from(&self.incoming_dir)
    }

    pub fn outbox_dir(&self) -> PathBuf {
        PathBuf::from(&self.outbox_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ron_round_trip_with_comments_and_defaults() {
        let cfg = AgentConfig::scaffold(Path::new("/home/dev"));
        assert_eq!(cfg.token.len(), 64);
        let back = AgentConfig::deserialize_ron(&format!("// header comment\n{}", cfg.serialize_ron())).unwrap();
        assert_eq!(back.token, cfg.token);
        assert_eq!(back.scan_dirs.len(), 2);

        // Only the required fields: bind and max_upload_bytes fall back to defaults.
        let minimal = AgentConfig::deserialize_ron(r#"(token: "t", incoming_dir: "in", outbox_dir: "out")"#).unwrap();
        assert_eq!(minimal.bind, "0.0.0.0:9787");
        assert_eq!(minimal.max_upload_bytes, 512 * 1024 * 1024);
        assert!(minimal.scan_dirs.is_empty());
        minimal.validate().unwrap();
    }
}
