pub mod requests;
use crate::asset;
use crate::screens::versions::requests::{Version, VersionType};
use crate::{
    query::{self, Query},
    state::Message,
};
use iced::{
    Element, Function, Task, color,
    widget::{Column, Row, button, column, container, row, scrollable, svg, text},
};

use requests::fetch_remote_version_list;

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
                fetch_remote_version_list,
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
}
