use crate::state::Message;
use iced::{Element, Task, Theme};
use iced_js::Host;
use nanoserde::DeJson;

#[derive(Debug, Default, DeJson)]
struct MessageDialogConfig {
    pub title: String,
    pub description: String,
}

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
                use iced_js::Event;
                if let iced_js::Event::Ipc(cmd, id, payload) = event {
                    match cmd.as_str() {
                        "fetch::install_build" => Task::done(Message::Js(
                            iced_js::Event::IpcDispatch(cmd, id, format!("null")),
                        )),
                        "logs::list-lines" => Task::perform(
                            async move {
                                iced_js::Event::IpcDispatch(
                                    cmd,
                                    id,
                                    "[\"Hello, set log line\"]".to_string(),
                                )
                            },
                            Message::Js,
                        ),
                        "install::build" => Task::perform(
                            async move { iced_js::Event::IpcDispatch(cmd, id, "{}".to_string()) },
                            Message::Js,
                        ),
                        "message-dialog" => Task::perform(
                            async move {
                                let config = match MessageDialogConfig::deserialize_json(&payload) {
                                    Ok(c) => c,
                                    Err(err) => {
                                        log::error!("{}", err);

                                        return Event::IpcDispatch(
                                            cmd,
                                            id,
                                            format!("{{ \"error\": \"{}\" }}", err.to_string()),
                                        );
                                    }
                                };

                                let dialog = rfd::AsyncMessageDialog::new()
                                    .set_title(config.title)
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .set_description(config.description);

                                let result = dialog.show().await;

                                let output = match result {
                                    rfd::MessageDialogResult::Yes => "Yes".to_string(),
                                    rfd::MessageDialogResult::No => "No".to_string(),
                                    rfd::MessageDialogResult::Ok => "Ok".to_string(),
                                    rfd::MessageDialogResult::Cancel => "Cancel".to_string(),
                                    rfd::MessageDialogResult::Custom(custom) => custom,
                                };

                                Event::IpcDispatch(
                                    cmd,
                                    id,
                                    format!("{{ \"result\": \"{}\" }}", output),
                                )
                            },
                            Message::Js,
                        ),
                        "open-folder" => Task::perform(
                            async move {
                                let picker = rfd::AsyncFileDialog::new().pick_folder().await;

                                let result = if let Some(handle) = picker {
                                    let path_str = handle.path().to_string_lossy();

                                    format!("{{\"path\": \"{}\"}}", path_str)
                                } else {
                                    r#"{ "path": null }"#.to_string()
                                };

                                Event::IpcDispatch(cmd, id, result)
                            },
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
