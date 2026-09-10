use crate::asset;
use crate::{
    screens,
    state::{Message, Screen, Tab},
};
use iced::{Element, Task, Theme};

#[derive(Debug)]
pub struct Application {
    pub screen: Screen,
}

impl Application {
    pub fn theme(_: &Application) -> Theme {
        Theme::Nord
    }
    pub fn new() -> (Self, Task<Message>) {
        let mut screen = screens::builds::Screen::default();
        let fetch = screen.fetch();

        let state = Self {
            screen: Screen::Builds(screen),
        };

        (state, fetch)
    }
    pub fn update(state: &mut Application, msg: Message) -> Task<Message> {
        match msg {
            Message::SetTab(tab) => match tab {
                Tab::Logs => {
                    state.screen = Screen::Logs;
                    Task::none()
                }
                Tab::Builds => {
                    let mut screen = screens::builds::Screen::default();

                    let tasks = screen.fetch();
                    state.screen = Screen::Builds(screen);

                    tasks
                }
                Tab::Profiles => {
                    let mut screen = screens::profiles::Screen::default();

                    let tasks = screen.init();
                    state.screen = Screen::Profiles(screen);

                    tasks
                }
                Tab::Files => {
                    state.screen = Screen::Files;
                    Task::none()
                }
            },
            Message::ProfilesMessage(ev) => {
                if let Screen::Profiles(screen) = &mut state.screen {
                    screen.update(ev)
                } else {
                    Task::none()
                }
            }
            Message::BuildsMessage(event) => {
                if let Screen::Builds(screen) = &mut state.screen {
                    screen.update(event)
                } else {
                    Task::none()
                }
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
                            <button onPress={Message::SetTab(Tab::Files)}>
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
                        Screen::Files => iced_xml::ui! { <col/> },
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
