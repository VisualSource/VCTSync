pub mod requests;
use crate::asset;

use crate::screens::builds::requests::{Version, VersionType};
use crate::state::Message;
use crate::traits::IcedScreen;
use crate::utils::{style_svg, tooltip_label};

use iced::{Alignment, Font};
use iced::{
    Element, Task,
    widget::{Column, container},
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
        if self.current_version.is_none() {
            return ui! {
            <row padding={4}>
                Installed:
                <space width={6}/>
                No Installed Version
            </row>
            };
        }

        ui! {
            <row padding={4}>
                Installed:

                <space width={6}/>

                <row spacing={4}>
                    <view width={iced::Shrink}>
                        <svg src={asset!("network.svg")} width={18} height={18} style={style_svg}/>
                    </view>
                    <view style={container::rounded_box} padding={[4,8]}>
                        <text center>
                            "0.0.22"
                        </text>
                    </view>
                </row>
            </row>
        }
    }

    fn remote_mod_version<'a>(&self, data: &'a Version) -> Element<'a, Message> {
        ui! {
            <row padding={[4,8]} alignY={Alignment::Center}>
                <view width={iced::Shrink}>
                    <svg src={asset!("network.svg")} width={24} height={24} style={style_svg}/>
                </view>
                <space width={10}/>
                <view style={container::rounded_box} padding={[4,8]}>
                    <text center>
                        {&data.version}
                    </text>
                </view>
                <space width={iced::Fill}/>
                <row spacing={4} alignY={Alignment::Center}>
                    <view>
                        <text center>{&data.timestamp}</text>
                    </view>
                    <tooltip content={tooltip_label("Install")} position={iced::widget::tooltip::Position::Left}>
                        <button>
                            <svg src={asset!("hard-drive-download.svg")}  width={24} height={24} style={style_svg}/>
                        </button>
                    </tooltip>
                </row>
            </row>
        }
    }

    fn local_mod_version<'a>(&self, data: &'a Version) -> Element<'a, Message> {
        ui! {
            <row padding={[4,8]} alignY={Alignment::Center}>
                <view width={iced::Shrink}>
                    <svg src={asset!("flask-conical.svg")} width={24} height={24} style={style_svg}/>
                </view>
                <space width={10}/>
                <view style={container::rounded_box} padding={[4,8]}>
                    <text center>
                        {&data.version}
                    </text>
                </view>
                <space width={iced::Fill}/>
                <row spacing={4} alignY={Alignment::Center}>
                    <view>
                        <text center>{&data.git_hash}</text>
                    </view>
                    <view>
                        <text center>{&data.timestamp}</text>
                    </view>
                    <button>
                        <svg src={asset!("hard-drive-download.svg")} width={24} height={24} style={style_svg}/>
                    </button>
                </row>
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
                        <text size={24} font={Font{ weight: iced::font::Weight::Bold, ..Font::DEFAULT }}>"Versions"</text>
                    </row>
                </view>
                {self.current_installed_version()}

                <space height={15}/>

                <col spacing={4}>
                    <row padding={2} alignY={Alignment::Center}>
                        <text>"Remote"</text>
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

                <space height={15}/>

                <col spacing={4}>
                    <row padding={2} alignY={Alignment::Center}>
                        <text>"Local"</text>
                        <space width={iced::Fill}/>
                         <button onPressMaybe={(!q.is_fetching()).then(|| Message::BuildsMessage(Action::RefreshRemoteVersions))}>
                            Refresh
                        </button>
                    </row>
                     <hr/>
                       <row height={256}>
                        <scroll spacing={4} >
                            {Column::with_children(
                                    self.local_versions
                                        .iter()
                                        .map(|version| self.local_mod_version(version))
                                        .map(Element::from)
                            )}
                        </scroll>
                    </row>
                </col>
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
