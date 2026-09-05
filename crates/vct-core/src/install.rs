//! Port of `VoidCrewTerminus/scripts/installer/install.ts` — install/update the
//! mod into a Thunderstore Mod Manager profile and keep `mods.yml` in sync.
//!
//! TMM lays its data out under `%APPDATA%` on Windows:
//! `Thunderstore Mod Manager/DataFolder/VoidCrew/profiles/<profile>/{mods.yml, BepInEx/plugins/...}`.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::manifest::{FALLBACK_DEPENDENCIES, parse_version};
use crate::{MOD_AUTHOR, MOD_NAME, PACKAGE_NAME};

const WEBSITE: &str = "https://github.com/VisualSource/VoidCrewTerminus";
const TMM_RELATIVE: &str = "Thunderstore Mod Manager/DataFolder/VoidCrew";

// ---- locate the data folder + profiles -------------------------------------

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// Where the TMM data folder (the one containing `profiles/`) might be.
pub fn candidate_data_folders() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        v.push(PathBuf::from(appdata).join(TMM_RELATIVE));
    }
    if let Some(home) = home_dir() {
        v.push(home.join(".config").join(TMM_RELATIVE));
    }
    v
}

/// Resolve the TMM data folder, honouring an explicit override from config.
pub fn resolve_data_folder(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        anyhow::ensure!(
            p.join("profiles").is_dir(),
            "{} has no 'profiles' subfolder",
            p.display()
        );
        return Ok(p.to_path_buf());
    }
    candidate_data_folders()
        .into_iter()
        .find(|p| p.join("profiles").is_dir())
        .context(
            "couldn't find the Thunderstore Mod Manager data folder \
             (usually %APPDATA%\\Thunderstore Mod Manager\\DataFolder\\VoidCrew); \
             set `tmm_data_folder` in config",
        )
}

/// Profile directory names under `<data_folder>/profiles`.
pub fn list_profiles(data_folder: &Path) -> Result<Vec<String>> {
    let profiles_dir = data_folder.join("profiles");
    let mut names: Vec<String> = std::fs::read_dir(&profiles_dir)
        .with_context(|| format!("reading {}", profiles_dir.display()))?
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    Ok(names)
}

pub fn profile_dir(data_folder: &Path, profile: &str) -> PathBuf {
    data_folder.join("profiles").join(profile)
}

// ---- install --------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallReport {
    pub version: String,
    pub files_written: u32,
    pub mod_dir: PathBuf,
    pub mods_yml_updated: bool,
    pub sha256: String,
}

/// Provenance record dropped into the installed mod folder as
/// [`MARKER_FILE`]. Local builds reuse version numbers, so the `sha256` and
/// `installed_at` are what actually identify what is on disk.
pub const MARKER_FILE: &str = "vct-sync-install.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMarker {
    /// e.g. `github:0.0.17` or `local:Debug`.
    pub source: String,
    pub version: String,
    pub sha256: String,
    /// Unix seconds.
    pub installed_at: i64,
}

/// Wipe + extract `zip_bytes` into `<profile_dir>/BepInEx/plugins/<PACKAGE_NAME>/`,
/// then upsert the `mods.yml` entry and drop a [`MARKER_FILE`]. `BepInEx/config`
/// is left untouched. `fallback_version` (e.g. a GitHub tag) is used only when the
/// zip's `manifest.json` has no version. `source` is a short provenance label.
pub fn install(
    zip_bytes: &[u8],
    profile_dir: &Path,
    fallback_version: Option<&str>,
    source: &str,
) -> Result<InstallReport> {
    let manifest = crate::build::read_manifest(zip_bytes).unwrap_or_default();

    let mod_dir = profile_dir
        .join("BepInEx")
        .join("plugins")
        .join(PACKAGE_NAME);
    if mod_dir.exists() {
        std::fs::remove_dir_all(&mod_dir).context("removing previous install")?;
    }
    std::fs::create_dir_all(&mod_dir)?;
    let files_written = extract_into(zip_bytes, &mod_dir)?;

    let version = manifest
        .version_number
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| fallback_version.map(|s| s.trim_start_matches(['v', 'V']).to_string()))
        .unwrap_or_else(|| "0.0.0".into());
    let dependencies: Vec<String> = if manifest.dependencies.is_empty() {
        FALLBACK_DEPENDENCIES.iter().map(|s| s.to_string()).collect()
    } else {
        manifest.dependencies.clone()
    };
    let website = manifest
        .website_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(WEBSITE);

    let mods_yml_updated = update_mods_yml(
        profile_dir,
        &version,
        manifest.description.as_deref().unwrap_or(""),
        website,
        &dependencies,
    )
    .context("updating mods.yml")?;

    let sha256 = hex::encode(Sha256::digest(zip_bytes));
    let marker = InstalledMarker {
        source: source.to_string(),
        version: version.clone(),
        sha256: sha256.clone(),
        installed_at: now_millis() / 1000,
    };
    if let Ok(json) = serde_json::to_vec_pretty(&marker) {
        let _ = std::fs::write(mod_dir.join(MARKER_FILE), json);
    }

    Ok(InstallReport {
        version,
        files_written,
        mod_dir,
        mods_yml_updated,
        sha256,
    })
}

/// The provenance marker for the currently-installed mod, if present.
pub fn installed_marker(profile_dir: &Path) -> Option<InstalledMarker> {
    let path = profile_dir
        .join("BepInEx")
        .join("plugins")
        .join(PACKAGE_NAME)
        .join(MARKER_FILE);
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// Extract with a zip-slip guard; skip directory and zero-length entries
/// (matches install.ts).
fn extract_into(zip_bytes: &[u8], dest: &Path) -> Result<u32> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))
        .context("the download couldn't be read as a zip")?;
    let mut written = 0u32;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i)?;
        let Some(rel) = f.enclosed_name() else {
            tracing::warn!("skipping unsafe zip path: {}", f.name());
            continue;
        };
        if f.is_dir() {
            continue;
        }
        let mut buf = Vec::with_capacity(f.size() as usize);
        f.read_to_end(&mut buf)?;
        if buf.is_empty() {
            continue;
        }
        let out = dest.join(rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out, &buf)?;
        written += 1;
    }
    Ok(written)
}

// ---- mods.yml -----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionNumber {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModEntry {
    manifest_version: u32,
    name: String,
    author_name: String,
    website_url: String,
    display_name: String,
    description: Option<String>,
    game_version: String,
    network_mode: String,
    package_type: String,
    install_mode: String,
    installed_at_time: i64,
    loaders: Vec<serde_yaml_ng::Value>,
    dependencies: Vec<String>,
    incompatibilities: Vec<serde_yaml_ng::Value>,
    optional_dependencies: Vec<serde_yaml_ng::Value>,
    version_number: VersionNumber,
    enabled: bool,
    online_source: bool,
}

fn now_millis() -> i64 {
    (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64
}

/// Upsert this mod's entry. Foreign entries are round-tripped as opaque YAML
/// values so nothing they carry is lost. Returns `true` if an existing entry was
/// updated, `false` if a new one was appended.
fn update_mods_yml(
    profile_dir: &Path,
    version: &str,
    description: &str,
    website: &str,
    dependencies: &[String],
) -> Result<bool> {
    let mods_yml = profile_dir.join("mods.yml");

    let mut items: Vec<serde_yaml_ng::Value> = if mods_yml.exists() {
        let text = std::fs::read_to_string(&mods_yml)?;
        serde_yaml_ng::from_str(&text).unwrap_or_else(|e| {
            tracing::warn!("mods.yml couldn't be parsed, rewriting fresh: {e}");
            Vec::new()
        })
    } else {
        Vec::new()
    };

    let idx = items.iter().position(|v| {
        v.get("name").and_then(|x| x.as_str()) == Some(PACKAGE_NAME)
            || v.get("displayName").and_then(|x| x.as_str()) == Some(MOD_NAME)
    });

    let (prev_installed_at, prev_enabled) = idx
        .map(|i| {
            let v = &items[i];
            (
                v.get("installedAtTime").and_then(|x| x.as_i64()),
                v.get("enabled").and_then(|x| x.as_bool()),
            )
        })
        .unwrap_or((None, None));

    let (major, minor, patch) = parse_version(version);
    let entry = ModEntry {
        manifest_version: 1,
        name: PACKAGE_NAME.to_string(),
        author_name: MOD_AUTHOR.to_string(),
        website_url: website.to_string(),
        display_name: MOD_NAME.to_string(),
        description: (!description.is_empty()).then(|| description.to_string()),
        game_version: "0".to_string(),
        network_mode: "both".to_string(),
        package_type: "other".to_string(),
        install_mode: "managed".to_string(),
        installed_at_time: prev_installed_at.unwrap_or_else(now_millis),
        loaders: Vec::new(),
        dependencies: dependencies.to_vec(),
        incompatibilities: Vec::new(),
        optional_dependencies: Vec::new(),
        version_number: VersionNumber {
            major,
            minor,
            patch,
        },
        enabled: prev_enabled.unwrap_or(true),
        online_source: false,
    };
    let value = serde_yaml_ng::to_value(&entry)?;

    let updated = match idx {
        Some(i) => {
            items[i] = value;
            true
        }
        None => {
            items.push(value);
            false
        }
    };

    std::fs::write(&mods_yml, serde_yaml_ng::to_string(&items)?)?;
    Ok(updated)
}

/// The mod's `(version, enabled)` as recorded in a profile's `mods.yml`, if present.
pub fn installed_version(profile_dir: &Path) -> Option<(String, bool)> {
    let text = std::fs::read_to_string(profile_dir.join("mods.yml")).ok()?;
    let items: Vec<serde_yaml_ng::Value> = serde_yaml_ng::from_str(&text).ok()?;
    let entry = items.iter().find(|v| {
        v.get("name").and_then(|x| x.as_str()) == Some(PACKAGE_NAME)
            || v.get("displayName").and_then(|x| x.as_str()) == Some(MOD_NAME)
    })?;
    let vn = entry.get("versionNumber")?;
    let g = |k| vn.get(k).and_then(|x| x.as_u64()).unwrap_or(0);
    let version = format!("{}.{}.{}", g("major"), g("minor"), g("patch"));
    let enabled = entry.get("enabled").and_then(|x| x.as_bool()).unwrap_or(true);
    Some((version, enabled))
}

// ---- log helpers ------------------------------------------------------------

/// `<profile_dir>/BepInEx/LogOutput.log`.
pub fn log_path(profile_dir: &Path) -> PathBuf {
    profile_dir.join("BepInEx").join("LogOutput.log")
}
