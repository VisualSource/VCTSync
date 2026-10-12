use crate::{
    ipc_mappings::{
        InstallBuildConfig, MessageDialogConfig, MessageDialogResult, OpenDialogResult,
    },
    state::{AppSettings, Message},
};
use iced::{Element, Task, Theme};
use iced_js::Host;
use nanoserde::{DeJson, SerJson, SerRon};

pub struct Application {
    host: iced_js::Host,
    settings: AppSettings,
}

impl Application {
    pub fn theme(_: &Application) -> Theme {
        Theme::Dark
    }
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            host: Host::new([("main", "app.js")]),
            settings: AppSettings::new(),
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
                        "save::settings" => {
                            let config = match AppSettings::deserialize_json(&payload) {
                                Ok(e) => e,
                                Err(err) => {
                                    log::error!("{}", err);

                                    return Task::done(Message::Js(Event::IpcDispatch(
                                        cmd,
                                        id,
                                        format!("null"),
                                    )));
                                }
                            };

                            let ron = config.serialize_ron();
                            state.settings = config;

                            Task::perform(
                                async move {
                                    let Some(config_dir) = dirs::config_dir() else {
                                        log::error!(
                                            "failed to write config write do to supported config dir"
                                        );
                                        return Event::IpcDispatch(cmd, id, "null".to_string());
                                    };

                                    let settings_file =
                                        config_dir.join("vcl.visualsource.github/settings.ron");

                                    if let Err(err) = tokio::fs::write(settings_file, ron).await {
                                        log::error!("{}", err)
                                    }

                                    Event::IpcDispatch(cmd, id, "null".to_string())
                                },
                                Message::Js,
                            )
                        }
                        "fetch::settings" => {
                            let data = state.settings.serialize_json();

                            Task::done(Message::Js(Event::IpcDispatch(cmd, id, data)))
                        }
                        "fetch::install_build" => Task::perform(
                            async move { Event::IpcDispatch(cmd, id, format!("null")) },
                            Message::Js,
                        ),
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
                            async move {
                                let config = match InstallBuildConfig::deserialize_json(&payload) {
                                    Ok(e) => e,
                                    Err(err) => {
                                        log::error!("{}", err);

                                        return Event::IpcDispatch(cmd, id, format!("null"));
                                    }
                                };

                                log::info!("Install: {:#?}", config);

                                iced_js::Event::IpcDispatch(cmd, id, "{}".to_string())
                            },
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

                                let data = MessageDialogResult::new(output);

                                Event::IpcDispatch(cmd, id, data.serialize_json())
                            },
                            Message::Js,
                        ),
                        "open-folder" => Task::perform(
                            async move {
                                let picker = rfd::AsyncFileDialog::new().pick_folder().await;

                                let result = if let Some(handle) = picker {
                                    let path_str = handle.path().to_string_lossy();
                                    Some(path_str.to_string())
                                } else {
                                    None
                                };

                                let data = OpenDialogResult::new(result);

                                Event::IpcDispatch(cmd, id, data.serialize_json())
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
