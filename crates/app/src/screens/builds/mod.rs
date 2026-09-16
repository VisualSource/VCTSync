pub mod requests;

use crate::asset;

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

use requests::{
    Progress, Version, fetch_active_version, fetch_local_version_list, fetch_remote_version_list,
    install_build,
};

#[derive(Debug, Clone)]
pub enum Action {
    RefreshRemoteBuilds,
    RefreshLocalBuilds,
    InstallBuild(Version),

    InstallProgress(Progress),
}

impl Into<Message> for Action {
    fn into(self) -> Message {
        Message::BuildsMessage(self)
    }
}

#[derive(Debug)]
pub struct Screen {
    current_version: Query<Option<Version>>,
    remote_versions: Query<Vec<Version>>,
    local_versions: Query<Vec<Version>>,
}

impl Screen {
    pub fn new(client: &QueryClient) -> Self {
        Self {
            current_version: Query::<Option<Version>>::new(
                client,
                "build::active",
                fetch_active_version,
                None,
            ),
            remote_versions: Query::<Vec<Version>>::new(
                client,
                "build::remote",
                fetch_remote_version_list,
                None,
            ),
            local_versions: Query::<Vec<Version>>::new(
                client,
                "builds::local",
                fetch_local_version_list,
                None,
            ),
        }
    }

    fn current_installed_version(&self) -> Element<'_, Message> {
        let q = &self.current_version.snapshot;
        match (&q.data, &q.error) {
            (Some(data), _) => {
                if let Some(info) = (*data).as_ref() {
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
                                        {&info.version}
                                    </text>
                                </view>
                            </row>
                        </row>
                    }
                } else {
                    ui! {
                        <row padding={4}>
                            Installed:
                            <space width={6}/>
                            No Installed Version
                        </row>
                    }
                }
            }
            (None, Some(err)) => {
                let reason = err.to_string();
                ui! {
                    <col>
                        <text>{reason}</text>
                    </col>
                }
            }

            (None, None) => ui! { <col>Loading</col> },
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
                        <button onPress={Action::InstallBuild(data.clone()).into()}>
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
                    <button onPress={Action::InstallBuild(data.clone()).into()}>
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

        let lq = &self.local_versions.snapshot;
        let local_builds = match (&lq.data, &lq.error) {
            (Some(data), _) => iced::Element::from(Column::with_children(
                data.iter()
                    .map(|version| self.local_mod_version(version))
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
                        <button onPressMaybe={(!q.is_fetching()).then(|| Message::BuildsMessage(Action::RefreshRemoteBuilds))}>
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
                         <button onPressMaybe={(!q.is_fetching()).then(|| Message::BuildsMessage(Action::RefreshLocalBuilds))}>
                            Refresh
                        </button>
                    </row>
                     <hr/>
                       <row height={256}>
                        <scroll spacing={4}>
                            {local_builds}
                        </scroll>
                    </row>
                </col>
            </col>
        }
    }

    fn update(&mut self, ev: Action, client: &QueryClient) -> Task<Message> {
        match ev {
            Action::RefreshRemoteBuilds => client
                .invalidate(&self.remote_versions.key)
                .map(Message::QueryUpdate),
            Action::RefreshLocalBuilds => client
                .invalidate(&self.local_versions.key)
                .map(Message::QueryUpdate),
            Action::InstallBuild(build) => {
                let install_task = install_build(build);

                Task::sip(install_task, Action::InstallProgress, |ev| {
                    Action::InstallProgress(match ev {
                        Ok(v) => v,
                        Err(err) => Progress::Error(err),
                    })
                })
                .map(Message::BuildsMessage)
            }
            Action::InstallProgress(state) => match state {
                Progress::Inc(_) => Task::none(),
                Progress::Done => client
                    .invalidate(&self.current_version.key)
                    .map(Message::QueryUpdate),
                Progress::Error(error) => {
                    eprintln!("{}", error);
                    Task::none()
                }
            },
        }
    }

    fn mount(&self, client: &QueryClient) -> Task<Message> {
        let rt = self.remote_versions.fetch(client).map(Message::QueryUpdate);
        let lb = self.local_versions.fetch(client).map(Message::QueryUpdate);
        let av = self.current_version.fetch(client).map(Message::QueryUpdate);

        Task::batch([rt, lb, av])
    }

    fn sync(&mut self, client: &QueryClient) {
        self.remote_versions.sync(client);
        self.local_versions.sync(client);
        self.current_version.sync(client);
    }
}
