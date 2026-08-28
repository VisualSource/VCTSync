use iced::{
    Element, color,
    task::{Straw, sipper},
    widget::{Column, Row, button, column, container, row, scrollable, svg, text},
};
use serde::Deserialize;

macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $name)
    };
}

use crate::state::Message;

#[derive(Debug, Default, Deserialize)]
pub enum VersionType {
    Remote,
    #[default]
    Local,
}

#[derive(Debug, Default, Deserialize)]
pub struct Version {
    version: String,
    git_hash: String,
    timestamp: String,
    content_type: VersionType,
    source_url: String,
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

#[derive(Debug)]
pub struct VersionsScreen {
    current_version: Option<Version>,
    remote_versions: Vec<Version>,
    local_versions: Vec<Version>,
}

impl Default for VersionsScreen {
    fn default() -> Self {
        VersionsScreen {
            current_version: None,
            remote_versions: Vec::default(),
            local_versions: vec![Version::new(
                "v0.14".into(),
                "aaff".into(),
                "1/33/3".into(),
                VersionType::Local,
                "local://hash".into(),
            )],
        }
    }
}

impl VersionsScreen {
    pub fn view(&self) -> Column<'_, Message> {
        column![
            container(row![text("Versions")]),
            self.current_installed_version(),
            container(column![
                row![text("Remote")],
                scrollable(Column::with_children(
                    self.remote_versions
                        .iter()
                        .map(|data| self.remote_mod_version(data))
                        .map(Element::from)
                ))
            ]),
            container(column![
                row![text("Local")],
                scrollable(
                    Column::with_children(
                        self.local_versions
                            .iter()
                            .map(|version| self.local_mod_version(version))
                            .map(Element::from),
                    )
                    .spacing(4),
                )
            ])
        ]
    }

    fn current_installed_version(&self) -> Row<'_, Message> {
        match &self.current_version {
            Some(data) => match data.content_type {
                VersionType::Local => self.local_mod_version(&data),
                VersionType::Remote => self.remote_mod_version(&data),
            },
            None => row![],
        }
    }

    fn remote_mod_version(&self, data: &Version) -> Row<'_, Message> {
        row![
            svg(asset!("network.svg")),
            text(data.version.clone()), // version
            button(svg(asset!("hard-drive-download.svg"))).on_press(Message::InstallModVersion)
        ]
    }

    fn local_mod_version(&self, data: &Version) -> Row<'_, Message> {
        row![
            svg(asset!("flask-conical.svg"))
                .width(52)
                .height(52)
                .style(|_theme, _status| svg::Style {
                    color: Some(color!(0xFFFFFF))
                }),
            text(data.version.clone()),              // version
            container(text(data.git_hash.clone())),  // git hash
            container(text(data.timestamp.clone())), // timestamp
            button(svg(asset!("hard-drive-download.svg"))).on_press(Message::InstallModVersion)
        ]
    }

    fn fetch_remote_version_list() -> impl Straw<(), Vec<Version>, anyhow::Error> {
        sipper(async move |mut state| {
            let response =
                reqwest::get("https://api.github.com/repos/VisualSource/VoidCrewTerminus/releases")
                    .await?;
            let releases = response.json::<Vec<GithubRelease>>().await?;

            let versions: Vec<Version> = releases
                .into_iter()
                .map(|item| Version {
                    version: item.tag_name,
                    git_hash: "".into(),
                    timestamp: "".into(),
                    content_type: VersionType::Local,
                    source_url: "".into(),
                })
                .collect();

            state.send(versions).await;

            Ok(())
        })
    }

    fn fetch_local_version_list() -> impl Straw<(), Vec<Version>, anyhow::Error> {
        sipper(async move |mut state| {
            let response = reqwest::get("http://localhost/versions").await?;

            let versions = response.json::<Vec<Version>>().await?;

            state.send(versions).await;

            Ok(())
        })
    }
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    prerelease: bool,
    draft: bool,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}
