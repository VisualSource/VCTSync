use std::sync::Arc;

use iced::{
    Element, Function, Task, color,
    task::{Straw, sipper},
    widget::{Column, Row, button, column, container, row, scrollable, svg, text},
};
use serde::Deserialize;

macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $name)
    };
}

use crate::{
    http,
    query::{self, Query},
    state::Message,
};

#[derive(Debug, Default, Deserialize, Clone)]
pub enum VersionType {
    Remote,
    #[default]
    Local,
}

#[derive(Debug, Default, Deserialize, Clone)]
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
    remote_versions: Query<Vec<Version>>,
    local_versions: Vec<Version>,
}

impl Default for VersionsScreen {
    fn default() -> Self {
        VersionsScreen {
            current_version: None,
            remote_versions: Query::<Vec<Version>>::new(
                "versions::remote".to_string(),
                VersionsScreen::fetch_remote_version_list,
            ),
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
        let rv = match &self.remote_versions.state {
            query::QueryState::Finished(data) => Column::with_children(
                data.iter()
                    .map(|version| self.remote_mod_version(version))
                    .map(Element::from),
            ),
            query::QueryState::Error(err) => column![text("Query Error:"), text(err.to_string())],
            _ => column![],
        };

        column![
            container(row![text("Versions")]),
            self.current_installed_version(),
            container(column![row![text("Remote")], scrollable(rv).spacing(4)]),
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
            .padding(2)
        ]
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::QueryUpdate(id, data) => {
                if id == self.remote_versions.id {
                    self.remote_versions.update(data);
                }

                Task::none()
            }
            _ => Task::none(),
        }
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
            button(svg(asset!("hard-drive-download.svg")))
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
            container(text(data.version.clone())).padding(2), // version
            container(text(data.git_hash.clone())),           // git hash
            container(text(data.timestamp.clone())),          // timestamp
            button(svg(asset!("hard-drive-download.svg")))
        ]
    }

    pub fn fetch(&mut self) -> Vec<Task<Message>> {
        let rt = self.remote_versions.start();

        vec![rt.map(Message::QueryUpdate.with(self.remote_versions.id.clone()))]
    }

    fn fetch_remote_version_list() -> impl Straw<Vec<Version>, (), Arc<anyhow::Error>> {
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
                    timestamp: "".into(),
                    content_type: VersionType::Local,
                    source_url: "".into(),
                })
                .collect();

            Ok(versions)
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
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}
