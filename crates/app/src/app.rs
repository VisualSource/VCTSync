use iced::{Element, Task, Theme};

use crate::{
    screens::versions::VersionsScreen,
    state::{Message, Screen, State, Tab},
};

pub struct Application {}

impl Application {
    pub fn theme(_: &State) -> Theme {
        Theme::Nord
    }
    pub fn new() -> State {
        State::default()
    }
    pub fn update(state: &mut State, msg: Message) -> Task<Message> {
        match msg {
            Message::SetTab(Tab::Files) => {
                state.screen = Screen::Files;
                Task::none()
            }
            Message::SetTab(Tab::Logs) => {
                state.screen = Screen::Logs;
                Task::none()
            }
            Message::SetTab(Tab::Versions) => {
                let mut screen = VersionsScreen::default();

                let tasks = Task::batch(screen.fetch());
                state.screen = Screen::Versions(screen);

                tasks
            }
            _ => match &mut state.screen {
                Screen::Logs => Task::none(),
                Screen::Versions(versions_screen) => versions_screen.update(msg),
                Screen::Files => Task::none(),
            },
        }
    }
    pub fn view(state: &State) -> Element<'_, Message> {
        iced_xml::ui! {
            <col>
                <view>
                    <row>
                        <button onPress={Message::SetTab(Tab::Versions)}>Builds</button>
                        <button onPress={Message::SetTab(Tab::Logs)}>Logs</button>
                        <button onPress={Message::SetTab(Tab::Files)}>Settings</button>
                    </row>
                </view>
                <view>
                {match &state.screen {
                    Screen::Logs => iced_xml::ui! { <col></col> },
                    Screen::Versions(screen) => screen.view(),
                    Screen::Files => iced_xml::ui! { <col></col> },
                }}
                </view>
            </col>
        }
    }
}
