mod screens;
mod state;
mod widgets;

use state::{Message, Screen, State};

use iced::{
    Element, Task, Theme,
    widget::{button, column, container, row, text},
};

use crate::screens::versions::VersionsScreen;

fn main() -> iced::Result {
    iced::application(new, update, view).theme(theme).run()
}

fn theme(state: &State) -> Theme {
    Theme::TokyoNight
}

fn new() -> State {
    State {
        screen: Screen::Versions(VersionsScreen::default()),
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SetScreenLogs => state.screen = Screen::Logs,
        Message::SetScreenVersions => state.screen = Screen::Versions(VersionsScreen::default()),
        Message::SetScreenFiles => state.screen = Screen::Files,

        Message::InstallModVersion => {}
    }

    Task::none()
}

fn view(state: &State) -> Element<'_, Message> {
    column![
        container(row![
            button(text("Versions")).on_press(Message::SetScreenVersions),
            button(text("Logs")).on_press(Message::SetScreenLogs),
            button(text("Files")).on_press(Message::SetScreenFiles)
        ]),
        container(match &state.screen {
            Screen::Logs => column![],
            Screen::Versions(screen) => screen.view(),
            Screen::Files => column![],
        })
    ]
    .into()
}
