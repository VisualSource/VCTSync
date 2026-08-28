use iced::{
    Element, Task, Theme,
    widget::{button, column, container, row, text},
};

use crate::{
    screens::versions::VersionsScreen,
    state::{Message, Screen, State, Tab},
};

pub struct Application {}

impl Application {
    pub fn theme(state: &State) -> Theme {
        Theme::Nord
    }
    pub fn new() -> State {
        State::default()
    }
    pub fn update(state: &mut State, msg: Message) -> Task<Message> {
        match msg {
            Message::SetTab(Tab::Files) => Task::none(),
            Message::SetTab(Tab::Logs) => Task::none(),
            Message::SetTab(Tab::Versions) => {
                let mut screen = VersionsScreen::default();

                let tasks = Task::batch(screen.fetch());
                state.screen = Screen::Versions(screen);

                tasks
            }
            _ => match &mut state.screen {
                Screen::Logs => todo!(),
                Screen::Versions(versions_screen) => versions_screen.update(msg),
                Screen::Files => todo!(),
            },
        }
    }
    pub fn view(state: &State) -> Element<'_, Message> {
        column![
            container(row![
                button(text("Versions")).on_press(Message::SetTab(Tab::Versions)),
                button(text("Logs")).on_press(Message::SetTab(Tab::Logs)),
                button(text("Files")).on_press(Message::SetTab(Tab::Files))
            ]),
            container(match &state.screen {
                Screen::Logs => column![],
                Screen::Versions(screen) => screen.view(),
                Screen::Files => column![],
            })
        ]
        .into()
    }
}
