//! Config lives in `config.yml` **next to the executable** (resolved via
//! `current_exe()`, not the working directory), overridable with `--config`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// `config.yml` beside the running executable.
pub fn default_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("cannot resolve current executable path")?;
    let dir = exe
        .parent()
        .context("executable path has no parent directory")?;
    Ok(dir.join("config.yml"))
}

pub fn load<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading config {}", path.display()))?;
    serde_yaml_ng::from_str(&text)
        .with_context(|| format!("parsing config {}", path.display()))
}

pub fn save<T: Serialize>(path: &Path, cfg: &T) -> Result<()> {
    let text = serde_yaml_ng::to_string(cfg)?;
    std::fs::write(path, text).with_context(|| format!("writing config {}", path.display()))
}

/// 32 random bytes, hex-encoded. Shared verbatim between agent and GUI config.
pub fn generate_token() -> String {
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf).expect("os rng unavailable");
    hex::encode(buf)
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Address to bind the HTTP server to.
    #[serde(default = "default_bind")]
    pub bind: String,
    /// Shared bearer token. `Authorization: Bearer <token>` on every request.
    pub token: String,
    /// Directories scanned for local build zips (`VoidCrewTerminus-<ver>.zip`).
    #[serde(default)]
    pub scan_dirs: Vec<PathBuf>,
    /// Where Windows→Linux uploads (`POST /files`) land.
    pub incoming_dir: PathBuf,
    /// Where `vct-agent send` stages Linux→Windows files.
    pub outbox_dir: PathBuf,
    /// Reject uploads larger than this.
    #[serde(default = "default_max_upload")]
    pub max_upload_bytes: u64,
}

fn default_bind() -> String {
    format!("0.0.0.0:{}", crate::DEFAULT_PORT)
}

fn default_max_upload() -> u64 {
    512 * 1024 * 1024
}

impl AgentConfig {
    /// A fresh config with a generated token and paths rooted at `base`.
    pub fn scaffold(base: &Path) -> Self {
        Self {
            bind: default_bind(),
            token: generate_token(),
            scan_dirs: vec![
                base.join("VoidCrewTerminus/bin/Debug/net472/Releases"),
                base.join("VoidCrewTerminus/bin/Release/net472/Releases"),
            ],
            incoming_dir: base.join("vct-sync-incoming"),
            outbox_dir: base.join("vct-sync-outbox"),
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
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuiConfig {
    /// Agent address, e.g. `192.168.1.94:9787`.
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub token: String,
    /// Last-used TMM profile directory name.
    #[serde(default)]
    pub profile: Option<String>,
    /// Explicit TMM data folder (the dir containing `profiles/`) when autodetect fails.
    #[serde(default)]
    pub tmm_data_folder: Option<PathBuf>,
    /// Where "Collect log" output and received files are written.
    #[serde(default)]
    pub save_dir: Option<PathBuf>,
    /// Auto-save incoming Linux→Windows files instead of prompting each time.
    #[serde(default)]
    pub auto_save_incoming: bool,
}
