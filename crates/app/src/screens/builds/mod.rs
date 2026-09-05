pub mod requests;
use crate::asset;
use crate::query::QueryUpdate;
use crate::screens::builds::requests::{Version, VersionType};
use crate::{
    query::{self, Query},
    state::Message,
};
use iced::{
    Element, Function, Task, color,
    widget::{Column, column, container, row, svg, text},
};
use iced_xml::ui;

use requests::fetch_remote_version_list;

#[derive(Debug, Clone)]
pub enum Action {
    QueryUpdate(String, QueryUpdate<Vec<Version>>),
}

#[derive(Debug)]
pub struct Screen {
    current_version: Option<Version>,
    remote_versions: Query<Vec<Version>>,
    local_versions: Vec<Version>,
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
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

impl Screen {
    pub fn view(&self) -> Element<'_, Message> {
        let rv = match &self.remote_versions.state {
            query::QueryState::Finished(data) => Column::with_children(
                data.iter()
                    .map(|version| self.remote_mod_version(version))
                    .map(Element::from),
            ),
            query::QueryState::Error(err) => column![text("Query Error:"), text(err.to_string())],
            _ => column![],
        };

        ui! {
            <col>
                <view>
                    <row>
                        Versions
                    </row>
                </view>
                {self.current_installed_version()}
                <view>
                    <col>
                        <row>
                            Remote
                        </row>
                        <scroll spacing={4}>
                            {rv}
                        </scroll>
                    </col>
                </view>
                <view>
                    <col>
                        <row>
                            Local
                        </row>
                        <scroll spacing={4}>
                            {Column::with_children(
                                    self.local_versions
                                        .iter()
                                        .map(|version| self.local_mod_version(version))
                                        .map(Element::from),
                            )}
                        </scroll>
                    </col>
                </view>
            </col>
        }
    }

    pub fn update(&mut self, msg: Action) -> Task<Message> {
        match msg {
            Action::QueryUpdate(id, data) => {
                if id == self.remote_versions.id {
                    self.remote_versions.update(data);
                }
                Task::none()
            }
        }
    }
    fn current_installed_version(&self) -> Element<'_, Message> {
        match &self.current_version {
            Some(data) => match data.content_type {
                VersionType::Local => self.local_mod_version(&data),
                VersionType::Remote => self.remote_mod_version(&data),
            },
            None => row![].into(),
        }
    }

    fn remote_mod_version(&self, data: &Version) -> Element<'_, Message> {
        ui! {
            <row>
                <view width={iced::Shrink}>
                    <svg src={asset!("network.svg")}  width={32} height={32} style={|_theme, _status| svg::Style {
                        color: Some(color!(0xFFFFFF))
                    }}/>
                </view>
                <view style={container::rounded_box}>
                    <text>
                        {data.version.clone()}
                    </text>
                </view>
                <button>
                    <svg src={asset!("hard-drive-download.svg")}  width={32} height={32} style={|_theme, _status| svg::Style {
                        color: Some(color!(0xFFFFFF))
                    }}/>
                </button>
            </row>
        }
    }

    fn local_mod_version(&self, data: &Version) -> Element<'_, Message> {
        ui! {
            <row padding={2}>
                <svg src={asset!("flask-conical.svg")} width={32} height={32} style={|_theme, _status| svg::Style {
                    color: Some(color!(0xFFFFFF))
                }}/>
                <view style={container::rounded_box} padding={2}>
                    <text>{data.version.clone()}</text>
                </view>
                <view>
                   <text>{data.git_hash.clone()}</text>
                </view>
                <view>
                   <text>{data.timestamp.clone()}</text>
                </view>
                <svg src={asset!("hard-drive-download.svg")}/>
            </row>
        }
    }

    pub fn fetch(&mut self) -> Vec<Task<Message>> {
        let rt = self.remote_versions.start();

        let quey_task = rt.map(|t| {
            Message::BuildsMessage(Action::QueryUpdate("versions::remote".to_string(), t))
        });

        vec![quey_task]
    }
}
