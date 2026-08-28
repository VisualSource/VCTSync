use std::sync::Arc;

use iced::task::{Straw, sipper};
use serde::Deserialize;

use crate::http;

#[derive(Debug, Default, Deserialize, Clone)]
pub enum VersionType {
    Remote,
    #[default]
    Local,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub(crate) struct Version {
    pub version: String,
    pub git_hash: String,
    pub timestamp: String,
    pub content_type: VersionType,
    pub source_url: String,
}

impl Version {
    pub fn new(
        version: String,
        git_hash: String,
        timestamp: String,
        ct: VersionType,
        source: String,
    ) -> Self {
        Version {
            version,
            git_hash,
            timestamp,
            content_type: ct,
            source_url: source,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    prerelease: bool,
    draft: bool,
    published_at: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}
pub fn fetch_remote_version_list() -> impl Straw<Vec<Version>, (), Arc<anyhow::Error>> {
    sipper(async move |_| {
        let client = http::get_client();

        let response = client
            .get("https://api.github.com/repos/VisualSource/VoidCrewTerminus/releases")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|err| Arc::new(anyhow::Error::from(err)))?
            .error_for_status()
            .map_err(|err| Arc::new(anyhow::Error::from(err)))?;

        let releases = response
            .json::<Vec<GithubRelease>>()
            .await
            .map_err(|err| Arc::new(anyhow::Error::from(err)))?;

        let versions: Vec<Version> = releases
            .into_iter()
            .map(|item| Version {
                version: item.tag_name,
                git_hash: "".into(),
                timestamp: item.published_at,
                content_type: VersionType::Remote,
                source_url: "".into(),
            })
            .collect();

        Ok(versions)
    })
}

pub fn fetch_local_version_list() -> impl Straw<Vec<Version>, (), Arc<anyhow::Error>> {
    sipper(async move |_| {
        let client = http::get_client();

        let response = client
            .get("http://localhost/version")
            .send()
            .await
            .map_err(|err| Arc::new(anyhow::Error::from(err)))?
            .error_for_status()
            .map_err(|err| Arc::new(anyhow::Error::from(err)))?;

        let releases = response
            .json::<Vec<Version>>()
            .await
            .map_err(|err| Arc::new(anyhow::Error::from(err)))?;

        Ok(releases)
    })
}
