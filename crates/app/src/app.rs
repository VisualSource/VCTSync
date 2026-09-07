use iced::{Element, Task, Theme};

use crate::{
    screens,
    state::{Message, Screen, Tab},
};

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
                Tab::Files => {
                    state.screen = Screen::Files;
                    Task::none()
                }
            },
            Message::BuildsMessage(event) => {
                if let Screen::Builds(screen) = &mut state.screen {
                    screen.update(event)
                } else {
                    Task::none()
                }
            }
            _ => Task::none(),
        }
    }
    pub fn view(state: &Application) -> Element<'_, Message> {
        iced_xml::ui! {
            <col>
                <view>
                    <row>
                        <button onPress={Message::SetTab(Tab::Builds)}>Builds</button>
                        <button onPress={Message::SetTab(Tab::Logs)}>Logs</button>
                        <button onPress={Message::SetTab(Tab::Files)}>Settings</button>
                    </row>
                </view>
                <view>
                {match &state.screen {
                    Screen::Logs => iced_xml::ui! { <col></col> },
                    Screen::Builds(screen) => screen.view(),
                    Screen::Files => iced_xml::ui! { <col></col> },
                }}
                </view>
            </col>
        }
    }
}
