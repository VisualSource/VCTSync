//! GUI config: `config.yml` next to the executable (override with `--config`).

use std::path::PathBuf;

use vct_core::config::{self, GuiConfig};

pub fn path() -> PathBuf {
    std::env::args()
        .skip_while(|a| a != "--config")
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| config::default_path().ok())
        .unwrap_or_else(|| PathBuf::from("config.yml"))
}

pub fn load() -> (PathBuf, GuiConfig) {
    let p = path();
    let cfg = config::load(&p).unwrap_or_else(|e| {
        tracing::warn!("no GUI config at {} ({e}); using defaults", p.display());
        GuiConfig::default()
    });
    (p, cfg)
}

pub fn save(path: &std::path::Path, cfg: &GuiConfig) -> anyhow::Result<()> {
    config::save(path, cfg)
}
