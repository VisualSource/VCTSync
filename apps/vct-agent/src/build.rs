//! Scanning the Linux dev box's build-output directories for `VoidCrewTerminus-*.zip`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use nanoserde::{DeJson, SerJson};
use sha2::{Digest, Sha256};

/// The Thunderstore `manifest.json` that `prebuild.py` writes into every build zip.
#[derive(Debug, Clone, Default, DeJson)]
pub struct PackageManifest {
    #[nserde(default)]
    pub version_number: Option<String>,
    #[nserde(default)]
    pub dependencies: Vec<String>,
}

/// One local build zip, as described to the GUI. `path` is agent-local and not
/// serialised.
#[derive(Debug, Clone, SerJson, DeJson)]
pub struct LocalBuild {
    /// Stable id — the content hash. Also the download key and the SSE change key.
    pub id: String,
    pub file_name: String,
    pub version: String,
    #[nserde(default)]
    pub dependencies: Vec<String>,
    /// `"Debug"` / `"Release"` / `""` — inferred from the path.
    pub config: String,
    pub size: u64,
    /// Modification time, unix seconds.
    pub mtime: i64,
    pub sha256: String,
    #[nserde(skip)]
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

    let sha256 = format!("{:x}", Sha256::digest(std::fs::read(path)?));

    let manifest = read_manifest(path).unwrap_or_default();
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
pub fn read_manifest(zip: &Path) -> Result<PackageManifest> {
    let mut reader = s_zip::StreamingZipReader::open(zip).context("not a valid zip archive")?;
    let entry = reader
        .entries()
        .iter()
        .find(|e| {
            e.name
                .rsplit(['/', '\\'])
                .next()
                .is_some_and(|base| base.eq_ignore_ascii_case("manifest.json"))
        })
        .cloned()
        .context("no manifest.json in zip")?;

    let bytes = reader.read_entry(&entry)?;
    let text = String::from_utf8_lossy(&bytes);
    // Thunderstore manifests often start with a UTF-8 BOM.
    PackageManifest::deserialize_json(text.trim_start_matches('\u{feff}'))
        .map_err(|err| anyhow::anyhow!("parsing manifest.json: {err}"))
}

/// `VoidCrewTerminus-0.0.17.zip` → `0.0.17`.
fn version_from_name(name: &str) -> Option<String> {
    name.strip_suffix(".zip")?
        .rsplit_once('-')
        .map(|(_, v)| v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_build_zip() {
        let dir = std::env::temp_dir().join(format!("vct-agent-build-test-{}", std::process::id()));
        let release = dir.join("Release");
        std::fs::create_dir_all(&release).unwrap();

        let with_manifest = release.join("VoidCrewTerminus-0.0.1.zip");
        let mut writer = s_zip::StreamingZipWriter::new(&with_manifest).unwrap();
        writer
            .add_entry("pkg/Manifest.JSON", "\u{feff}{\"version_number\":\"0.0.22\",\"dependencies\":[\"BepInEx-BepInExPack-5.4.2100\"],\"extra\":1}".as_bytes())
            .unwrap();
        writer.finish().unwrap();

        let without_manifest = release.join("VoidCrewTerminus-0.0.9.zip");
        let mut writer = s_zip::StreamingZipWriter::new(&without_manifest).unwrap();
        writer.add_entry("VoidCrewTerminus.dll", b"dll").unwrap();
        writer.finish().unwrap();

        let builds = scan(&[release.clone()]);
        assert_eq!(builds.len(), 2);
        let b = builds.iter().find(|b| b.path == with_manifest).unwrap();
        assert_eq!(b.version, "0.0.22", "manifest wins over the file name");
        assert_eq!(b.dependencies, ["BepInEx-BepInExPack-5.4.2100"]);
        assert_eq!(b.config, "Release");
        assert_eq!(b.id.len(), 64);
        assert!(!b.serialize_json().contains("path"), "path stays agent-local");

        let fallback = builds.iter().find(|b| b.path == without_manifest).unwrap();
        assert_eq!(fallback.version, "0.0.9", "falls back to the file name");

        std::fs::remove_dir_all(dir).unwrap();
    }
}
