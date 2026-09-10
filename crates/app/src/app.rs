use crate::asset;
use crate::traits::IcedScreen;
use crate::{
    screens,
    state::{Message, Screen, Tab},
};
use iced::{Element, Task, Theme};
use iced_query::QueryClient;

macro_rules! mount_page {
    ($state:ident, $enum:path, $screen: expr) => {{
        let screen = $screen;
        let tasks = screen.mount(&$state.query_client);
        $state.screen = $enum(screen);

        tasks
    }};
}
macro_rules! page_update {
    ($state: ident, $enum:path, $ev:expr) => {
        if let $enum(screen) = &mut $state.screen {
            screen.update($ev, &$state.query_client)
        } else {
            Task::none()
        }
    };
}

#[derive(Debug)]
pub struct Application {
    pub screen: Screen,
    pub query_client: QueryClient,
}

impl Application {
    pub fn theme(_: &Application) -> Theme {
        Theme::Dark
    }
    pub fn new() -> (Self, Task<Message>) {
        let query_client = QueryClient::new();

        let screen = screens::builds::Screen::new(&query_client);
        let fetch = screen.mount(&query_client);

        let state = Self {
            screen: Screen::Builds(screen),
            query_client: query_client,
        };

        (state, fetch)
    }
    pub fn update(state: &mut Application, msg: Message) -> Task<Message> {
        match msg {
            Message::QueryUpdate(data) => {
                state.query_client.receive(data);

                match &mut state.screen {
                    Screen::Builds(screen) => screen.sync(&state.query_client),
                    Screen::Profiles(screen) => screen.sync(&state.query_client),
                    Screen::Logs => {}
                    Screen::Settings(screen) => screen.sync(&state.query_client),
                }

                Task::none()
            }
            Message::SetTab(tab) => match tab {
                Tab::Logs => {
                    state.screen = Screen::Logs;
                    Task::none()
                }
                Tab::Builds => mount_page!(
                    state,
                    Screen::Builds,
                    screens::builds::Screen::new(&state.query_client)
                ),

                Tab::Profiles => mount_page!(
                    state,
                    Screen::Profiles,
                    screens::profiles::Screen::default()
                ),
                Tab::Settings => {
                    mount_page!(state, Screen::Settings, screens::settings::Screen::new())
                }
            },
            Message::ProfilesMessage(ev) => {
                page_update!(state, Screen::Profiles, ev)
            }
            Message::BuildsMessage(event) => {
                page_update!(state, Screen::Builds, event)
            }
            Message::SettingsMessage(event) => {
                page_update!(state, Screen::Settings, event)
            }
        }
    }
    pub fn view(state: &Application) -> Element<'_, Message> {
        iced_xml::ui! {
            <row>
                <view>
                    <col spacing={4}>
                        <!-- Profile list -->
                        <tooltip content={tooltip_label("Profiles")} position={iced::widget::tooltip::Position::Right}>
                            <button onPress={Message::SetTab(Tab::Profiles)}>
                                <svg width={18} height={18} src={asset!("library.svg")}/>
                            </button>
                        </tooltip>

                        <!-- Thunder store install version install list  -->
                        <tooltip content={tooltip_label("Builds")} position={iced::widget::tooltip::Position::Right}>
                            <button onPress={Message::SetTab(Tab::Builds)}>
                                <svg width={18} height={18} src={asset!("boxes.svg")}/>
                            </button>
                        </tooltip>
                        <tooltip content={tooltip_label("Logs")} position={iced::widget::tooltip::Position::Right}>
                            <button onPress={Message::SetTab(Tab::Logs)}>
                                <svg width={18} height={18} src={asset!("scroll-text.svg")}/>
                            </button>
                        </tooltip>
                        <space height={iced::Fill}/>
                        <tooltip content={tooltip_label("Settings")} position={iced::widget::tooltip::Position::Right}>
                            <button onPress={Message::SetTab(Tab::Settings)}>
                                <svg width={18} height={18} src={asset!("settings.svg")}/>
                            </button>
                        </tooltip>
                        <space height={4}/>
                    </col>
                </view>
                <vr/>
                <space width={4}/>
                <view>
                    {match &state.screen {
                        Screen::Logs => iced_xml::ui! { <col/> },
                        Screen::Builds(screen) => screen.view(),
                        Screen::Profiles(screen) => screen.view(),
                        Screen::Settings(screen) => screen.view(),
                    }}
                </view>
            </row>
        }
    }
}

fn tooltip_label(text: &str) -> Element<'_, Message> {
    iced_xml::ui! {
        <view padding={[4,8]} style={iced::widget::container::rounded_box}>
            <text>{text}</text>
        </view>
    }
}
