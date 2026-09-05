//! The Thunderstore `manifest.json` that `prebuild.py` writes into every build zip.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageManifest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version_number: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// install.ts's fallback when a zip omits its manifest.
pub const FALLBACK_DEPENDENCIES: &[&str] = &[
    "BepInEx-BepInExPack-5.4.2100",
    "NihilityShift-VoidManager-1.2.10",
];

/// `0.0.17` → `(0, 0, 17)`; missing parts default to 0, leading `v` tolerated.
pub fn parse_version(v: &str) -> (u64, u64, u64) {
    let mut it = v.trim_start_matches(['v', 'V']).split('.');
    let major = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}
