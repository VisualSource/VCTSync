//! Scanning the Linux dev box's build-output directories for `VoidCrewTerminus-*.zip`.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::manifest::PackageManifest;

/// One local build zip, as described to the GUI. `path` is agent-local and not
/// serialised.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalBuild {
    /// Stable id — the content hash. Also the download key and the SSE change key.
    pub id: String,
    pub file_name: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// `"Debug"` / `"Release"` / `""` — inferred from the path.
    pub config: String,
    pub size: u64,
    /// Modification time, unix seconds.
    pub mtime: i64,
    pub sha256: String,
    #[serde(skip)]
    pub path: PathBuf,
}

/// Scan every directory in `dirs` for `*.zip`, newest first. Unreadable dirs and
/// unreadable zips are logged and skipped, never fatal. Identical zips (same
/// content hash) are de-duplicated across directories.
pub fn scan(dirs: &[PathBuf]) -> Vec<LocalBuild> {
    let mut out: Vec<LocalBuild> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for dir in dirs {
        let rd = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(e) => {
                tracing::debug!("scan dir {} skipped: {e}", dir.display());
                continue;
            }
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("zip") {
                continue;
            }
            match describe(&path) {
                Ok(b) => {
                    if seen.insert(b.id.clone()) {
                        out.push(b);
                    }
                }
                Err(e) => tracing::warn!("skipping {}: {e:#}", path.display()),
            }
        }
    }

    out.sort_by(|a, b| b.mtime.cmp(&a.mtime));
    out
}

fn describe(path: &Path) -> Result<LocalBuild> {
    let meta = std::fs::metadata(path)?;
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let bytes = std::fs::read(path)?;
    let sha256 = hex::encode(Sha256::digest(&bytes));

    let manifest = read_manifest(&bytes).unwrap_or_default();
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let version = manifest
        .version_number
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| version_from_name(&file_name))
        .unwrap_or_else(|| "unknown".into());
    let config = path
        .components()
        .rev()
        .find_map(|c| {
            let s = c.as_os_str().to_string_lossy();
            (s == "Debug" || s == "Release").then(|| s.into_owned())
        })
        .unwrap_or_default();

    Ok(LocalBuild {
        id: sha256.clone(),
        file_name,
        version,
        dependencies: manifest.dependencies,
        config,
        size,
        mtime,
        sha256,
        path: path.to_path_buf(),
    })
}

/// Read `manifest.json` out of a build zip (matched case-insensitively, at any depth).
pub fn read_manifest(zip_bytes: &[u8]) -> Result<PackageManifest> {
    let mut zip =
        zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).context("not a valid zip archive")?;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i)?;
        let base = f.name().rsplit(['/', '\\']).next().unwrap_or("").to_owned();
        if base.eq_ignore_ascii_case("manifest.json") {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            return serde_json::from_str(&s).context("parsing manifest.json");
        }
    }
    anyhow::bail!("no manifest.json in zip")
}

/// `VoidCrewTerminus-0.0.17.zip` → `0.0.17`.
fn version_from_name(name: &str) -> Option<String> {
    name.strip_suffix(".zip")?
        .rsplit_once('-')
        .map(|(_, v)| v.to_string())
}
