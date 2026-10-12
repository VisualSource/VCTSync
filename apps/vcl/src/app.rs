use crate::state::Message;
use iced::{Element, Task, Theme};
use iced_js::Host;

pub struct Application {
    host: iced_js::Host,
}

impl Application {
    pub fn theme(_: &Application) -> Theme {
        Theme::Dark
    }
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            host: Host::new([("main", "app.js")]),
        };

        (state, Task::none())
    }
    pub fn update(state: &mut Application, msg: Message) -> Task<Message> {
        match msg {
            Message::Reload => state.host.reload().map(Message::Js),
            Message::Js(event) => {
                if let iced_js::Event::Ipc(cmd, id, payload) = event {
                    match cmd.as_str() {
                        "fetch::install_build" => Task::done(Message::Js(
                            iced_js::Event::IpcDispatch(cmd, id, format!("null")),
                        )),
                        "logs::list-lines" => Task::perform(
                            async move { iced_js::Event::IpcDispatch(cmd, id, "[]".to_string()) },
                            Message::Js,
                        ),
                        "install::build" => Task::perform(
                            async move { iced_js::Event::IpcDispatch(cmd, id, "{}".to_string()) },
                            Message::Js,
                        ),
                        _ => Task::none(),
                    }
                } else {
                    state.host.update(event).map(Message::Js)
                }
            }
        }
    }
    pub fn view(state: &Application) -> Element<'_, Message> {
        iced_js::surface(&state.host, "main").map(Message::Js)
    }
}
