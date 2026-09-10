pub mod requests;
use crate::asset;

use crate::screens::builds::requests::{Version, VersionType};
use crate::state::Message;
use crate::traits::IcedScreen;
use iced::{
    Element, Task, color,
    widget::{Column, container, row, svg},
};
use iced_query::{Query, QueryClient};
use iced_xml::ui;

use requests::fetch_remote_version_list;

#[derive(Debug, Clone)]
pub enum Action {
    RefreshRemoteVersions,
}

#[derive(Debug)]
pub struct Screen {
    current_version: Option<Version>,
    remote_versions: Query<Vec<Version>>,
    local_versions: Vec<Version>,
}

impl Screen {
    pub fn new(client: &QueryClient) -> Self {
        Self {
            current_version: None,
            remote_versions: Query::<Vec<Version>>::new(
                client,
                "versions::remote",
                fetch_remote_version_list,
                None,
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

    fn current_installed_version(&self) -> Element<'_, Message> {
        match &self.current_version {
            Some(data) => match data.content_type {
                VersionType::Local => self.local_mod_version(&data),
                VersionType::Remote => self.remote_mod_version(&data),
            },
            None => row![].into(),
        }
    }

    fn remote_mod_version<'a>(&self, data: &'a Version) -> Element<'a, Message> {
        ui! {
            <row padding={2}>
                <view width={iced::Shrink}>
                    <svg src={asset!("network.svg")}  width={24} height={24} style={|_theme, _status| svg::Style {
                        color: Some(color!(0xFFFFFF))
                    }}/>
                </view>
                <space width={10}/>
                <view style={container::rounded_box} padding={[4,8]}>
                    <text center>
                        {&data.version}
                    </text>
                </view>
                <space width={iced::Fill}/>
                <row spacing={4}>
                    <view>
                        <text>{&data.timestamp}</text>
                    </view>
                    <button>
                        <svg src={asset!("hard-drive-download.svg")}  width={24} height={24} style={|_theme, _status| svg::Style {
                            color: Some(color!(0xFFFFFF))
                        }}/>
                    </button>
                    <button>
                        <svg src={asset!("hard-drive-download.svg")}  width={24} height={24} style={|_theme, _status| svg::Style {
                            color: Some(color!(0xFFFFFF))
                        }}/>
                    </button>
                </row>
            </row>
        }
    }

    fn local_mod_version<'a>(&self, data: &'a Version) -> Element<'a, Message> {
        ui! {
            <row padding={2}>
                <svg src={asset!("flask-conical.svg")} width={32} height={32} style={|_theme, _status| svg::Style {
                    color: Some(color!(0xFFFFFF))
                }}/>
                <view style={container::rounded_box} padding={2}>
                    <text>{&data.version}</text>
                </view>
                <view>
                   <text>{&data.git_hash}</text>
                </view>
                <view>
                   <text>{&data.timestamp}</text>
                </view>
                <svg src={asset!("hard-drive-download.svg")}/>
            </row>
        }
    }
}

impl IcedScreen<Action> for Screen {
    fn view(&self) -> iced::Element<'_, Message> {
        let q = &self.remote_versions.snapshot;

        let rv = match (&q.data, &q.error) {
            (Some(data), _) => iced::Element::from(Column::with_children(
                data.iter()
                    .map(|version| self.remote_mod_version(version))
                    .map(Element::from),
            )),

            (None, Some(err)) => {
                let reason = err.to_string();
                ui! {
                    <col>
                        <text>{reason}</text>
                    </col>
                }
            }

            (None, None) => ui! { <col>Loading</col> },
        };

        ui! {
            <col>
                <view>
                    <row>
                        Versions
                    </row>
                </view>
                {self.current_installed_version()}

                <col spacing={4}>
                    <row padding={2}>
                        <text center alignY={iced::Alignment::Center}>"Remote"</text>
                        <space width={iced::Fill}/>
                        <button onPressMaybe={(!q.is_fetching()).then(|| Message::BuildsMessage(Action::RefreshRemoteVersions))}>
                            Refresh
                        </button>
                    </row>
                    <hr/>
                    <row height={256}>
                        <scroll spacing={4} >
                            {rv}
                        </scroll>
                    </row>
                </col>

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
                                        .map(Element::from)
                            )}
                        </scroll>
                    </col>
                </view>
            </col>
        }
    }

    fn update(&mut self, ev: Action, client: &QueryClient) -> Task<Message> {
        match ev {
            Action::RefreshRemoteVersions => client
                .invalidate(&self.remote_versions.key)
                .map(Message::QueryUpdate),
        }
    }

    fn mount(&self, client: &QueryClient) -> Task<Message> {
        let rt = self.remote_versions.fetch(client).map(Message::QueryUpdate);

        Task::batch(vec![rt])
    }

    fn sync(&mut self, client: &QueryClient) {
        self.remote_versions.sync(client);
    }
}
