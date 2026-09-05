//! GitHub Releases — **published releases only, unauthenticated**. Anonymous API
//! is 60 req/hr, so callers should cache and refresh on demand rather than poll.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::http;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

impl Release {
    /// The mod package zip: prefer an asset whose name contains the mod name,
    /// else the first `.zip`. GitHub's auto-generated "Source code" archives are
    /// not in `assets`, so they are skipped for free.
    pub fn mod_asset(&self) -> Option<&ReleaseAsset> {
        let is_zip = |a: &&ReleaseAsset| a.name.to_lowercase().ends_with(".zip");
        self.assets
            .iter()
            .filter(is_zip)
            .find(|a| {
                a.name
                    .to_lowercase()
                    .contains(&crate::MOD_NAME.to_lowercase())
            })
            .or_else(|| self.assets.iter().find(is_zip))
    }

    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.tag_name)
    }
}

/// Non-draft releases that ship a zip asset. GitHub returns newest-first.
pub async fn fetch_releases() -> Result<Vec<Release>> {
    let url = format!(
        "https://api.github.com/repos/{}/releases",
        crate::GITHUB_REPO
    );
    let releases: Vec<Release> = http::client()
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("GitHub releases request failed")?
        .error_for_status()
        .context("GitHub releases request returned an error status")?
        .json()
        .await
        .context("decoding GitHub releases response")?;

    Ok(releases
        .into_iter()
        .filter(|r| !r.draft && r.mod_asset().is_some())
        .collect())
}

pub async fn download_asset(url: &str) -> Result<Vec<u8>> {
    let bytes = http::client()
        .get(url)
        .send()
        .await
        .context("asset download request failed")?
        .error_for_status()
        .context("asset download returned an error status")?
        .bytes()
        .await
        .context("reading asset bytes")?;
    Ok(bytes.to_vec())
}
