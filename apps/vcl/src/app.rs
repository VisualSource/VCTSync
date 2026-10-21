use crate::{
    helpers::{app_dir, delete_profile_dir, game_status, ipc_reply, load_profiles, profile_from_dir, save_profiles, write_app_file},
    ipc_mappings::{
        InstallBuildConfig, IpcError, MessageDialogConfig, MessageDialogResult, OpenDialogResult,
        InstallModConfig, InstallModResult, LaunchGameConfig, OpenFileConfig, UninstallModConfig, SetModEnabledConfig, ProfileDirConfig, DeleteDirResult, ZipPathConfig, AppInfo, OpenPathConfig, BuildProgress,
    },
    state::{AppSettings, Message},
};
use iced::{Element, Task, Theme};
use iced_js::Host;
use nanoserde::{DeJson, SerJson, SerRon};

/// Builds install like a Thunderstore mod under this "Author-Name".
const BUILD_FULL_NAME: &str = "VisualSource-VoidCrewTerminus";

pub struct Application {
    host: iced_js::Host,
    settings: AppSettings,
}

impl Application {
    pub fn theme(state: &Application) -> Theme {
        state.settings.theme()
    }
    pub fn new() -> (Self, Task<Message>) {
        let settings = app_dir().map(|dir| AppSettings::load(&dir.join("settings.ron"))).unwrap_or_default();
        let state = Self {
            host: Host::new([("main", "app.js")]),
            settings,
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
                                Ok(config) => config,
                                Err(err) => {
                                    let data = ipc_reply(Err(err.to_string()));
                                    return Task::done(Message::Js(Event::IpcDispatch(cmd, id, data)));
                                }
                            };

                            // Applied right away (theme, launch method); the file write follows.
                            let ron = config.serialize_ron();
                            state.settings = config;

                            Task::perform(
                                async move {
                                    let result = write_app_file("settings.ron", ron).await.map_err(|err| err.to_string());
                                    Event::IpcDispatch(cmd, id, ipc_reply(result))
                                },
                                Message::Js,
                            )
                        }
                        "app::info" => {
                            let info = AppInfo {
                                app_dir: app_dir().map(|dir| dir.to_string_lossy().to_string()).unwrap_or_default(),
                                version: env!("CARGO_PKG_VERSION").to_string(),
                                themes: Theme::ALL.iter().map(|theme| theme.to_string()).collect(),
                            };
                            Task::done(Message::Js(Event::IpcDispatch(cmd, id, info.serialize_json())))
                        }
                        "game::status" => {
                            let settings = state.settings.clone();
                            Task::perform(
                                async move {
                                    let status = tokio::task::spawn_blocking(move || game_status(&settings))
                                        .await
                                        .map(|status| status.serialize_json())
                                        .unwrap_or_else(|err| ipc_reply(Err(err.to_string())));
                                    Event::IpcDispatch(cmd, id, status)
                                },
                                Message::Js,
                            )
                        }
                        "game::unlink" => {
                            let settings = state.settings.clone();
                            Task::perform(
                                async move {
                                    let result = tokio::task::spawn_blocking(move || -> Result<(), bepinex_lib::LibError> {
                                        bepinex_lib::unlink_profile(settings.steam()?.game_dir())
                                    })
                                    .await
                                    .map_err(|err| err.to_string())
                                    .and_then(|r| r.map_err(|err| err.to_string()));
                                    Event::IpcDispatch(cmd, id, ipc_reply(result))
                                },
                                Message::Js,
                            )
                        }
                        "open-path" => Task::perform(
                            async move {
                                let result = OpenPathConfig::deserialize_json(&payload)
                                    .map_err(|err| err.to_string())
                                    .and_then(|config| {
                                        let opener = if cfg!(windows) { "explorer" } else { "xdg-open" };
                                        std::process::Command::new(opener)
                                            .arg(config.path)
                                            .spawn()
                                            .map(|_| ())
                                            .map_err(|err| err.to_string())
                                    });
                                Event::IpcDispatch(cmd, id, ipc_reply(result))
                            },
                            Message::Js,
                        ),
                        "fetch::settings" => {
                            let data = state.settings.serialize_json();

                            Task::done(Message::Js(Event::IpcDispatch(cmd, id, data)))
                        }
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
                        "install::build" => {
                            let config = match InstallBuildConfig::deserialize_json(&payload) {
                                Ok(config) => config,
                                Err(err) => {
                                    let data = ipc_reply(Err(err.to_string()));
                                    return Task::done(Message::Js(Event::IpcDispatch(cmd, id, data)));
                                }
                            };
                            let build_id = config.id.clone();

                            Task::sip(
                                iced::task::sipper(async move |mut sender| {
                                    use iced::futures::StreamExt;

                                    let profile = profile_from_dir(&config.profile_dir);
                                    // The download reports into a channel that's forwarded to iced; the channel
                                    // closes when the install future finishes and drops the callback.
                                    let (tx, mut rx) = iced::futures::channel::mpsc::unbounded();
                                    // ~100 progress events per download, however many chunks arrive.
                                    let mut last_sent = 0u64;
                                    let install = bepinex_lib::install_mod_download(
                                        crate::http::get_client(),
                                        &config.source_url,
                                        BUILD_FULL_NAME,
                                        &profile,
                                        Some(&config.digest),
                                        move |downloaded, total| {
                                            let step = (total / 100).max(64 * 1024);
                                            if downloaded == total || downloaded - last_sent >= step {
                                                last_sent = downloaded;
                                                let _ = tx.unbounded_send((downloaded, total));
                                            }
                                        },
                                    );
                                    let forward = async {
                                        while let Some(progress) = rx.next().await {
                                            sender.send(progress).await;
                                        }
                                    };
                                    iced::futures::join!(install, forward).0
                                }),
                                move |(downloaded, total)| {
                                    let data = BuildProgress { id: build_id.clone(), downloaded, total }.serialize_json();
                                    Message::Js(Event::IpcDispatch("build::progress".to_string(), String::new(), data))
                                },
                                move |result| {
                                    let data = match result {
                                        Ok(icon) => InstallModResult { icon: icon.map(|p| p.to_string_lossy().to_string()) }.serialize_json(),
                                        Err(err) => ipc_reply(Err(err.to_string())),
                                    };
                                    Message::Js(Event::IpcDispatch(cmd, id, data))
                                },
                            )
                        }
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

                                let buttons = match config.buttons.as_str() {
                                    "Ok" => rfd::MessageButtons::Ok,
                                    "OkCancel" => rfd::MessageButtons::OkCancel,
                                    "YesNo" => rfd::MessageButtons::YesNo,
                                    "YesNoCancel" => rfd::MessageButtons::YesNoCancel,
                                    _ => rfd::MessageButtons::OkCustom(config.buttons),
                                };

                                let level = if let Some(state) = config.level {
                                    match state.as_str() {
                                        "warning" => rfd::MessageLevel::Warning,
                                        "error" => rfd::MessageLevel::Error,
                                        _ => rfd::MessageLevel::Info,
                                    }
                                } else {
                                    rfd::MessageLevel::Info
                                };

                                let dialog = rfd::AsyncMessageDialog::new()
                                    .set_title(config.title)
                                    .set_buttons(buttons)
                                    .set_description(config.description)
                                    .set_level(level);

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
                        "open-file" => Task::perform(
                            async move {
                                let config = OpenFileConfig::deserialize_json(&payload).unwrap_or_default();
                                let mut dialog = rfd::AsyncFileDialog::new();
                                if !config.filters.is_empty() {
                                    dialog = dialog.add_filter(config.filters.join(", "), &config.filters);
                                }

                                let path = dialog
                                    .pick_file()
                                    .await
                                    .map(|handle| handle.path().to_string_lossy().to_string());

                                Event::IpcDispatch(cmd, id, OpenDialogResult::new(path).serialize_json())
                            },
                            Message::Js,
                        ),
                        "zip::read-manifest" => Task::perform(
                            async move {
                                let result = match ZipPathConfig::deserialize_json(&payload) {
                                    Ok(config) => bepinex_lib::read_zip_manifest(std::path::Path::new(&config.path))
                                        .await
                                        .map_err(|err| err.to_string()),
                                    Err(err) => Err(err.to_string()),
                                };

                                let data = result.unwrap_or_else(|error| IpcError { error }.serialize_json());

                                Event::IpcDispatch(cmd, id, data)
                            },
                            Message::Js,
                        ),
                        "mods::install" => Task::perform(
                            async move {
                                let result = match InstallModConfig::deserialize_json(&payload) {
                                    Ok(config) => {
                                        let profile = profile_from_dir(&config.profile_dir);
                                        match (config.path, config.url) {
                                            (Some(path), _) => bepinex_lib::install_mod_zip(std::path::Path::new(&path), &config.full_name, &profile)
                                                .await
                                                .map_err(|err| err.to_string()),
                                            (None, Some(url)) => bepinex_lib::install_mod_url(crate::http::get_client(), &url, &config.full_name, &profile)
                                                .await
                                                .map_err(|err| err.to_string()),
                                            (None, None) => Err("mods::install needs a path or url".to_string()),
                                        }
                                    }
                                    Err(err) => Err(err.to_string()),
                                };

                                let data = match result {
                                    Ok(icon) => InstallModResult { icon: icon.map(|p| p.to_string_lossy().to_string()) }.serialize_json(),
                                    Err(error) => IpcError { error }.serialize_json(),
                                };

                                Event::IpcDispatch(cmd, id, data)
                            },
                            Message::Js,
                        ),
                        "profiles::list" => Task::perform(
                            async move {
                                let data = match load_profiles().await {
                                    Ok(profiles) => profiles.serialize_json(),
                                    Err(err) => IpcError { error: err.to_string() }.serialize_json(),
                                };

                                Event::IpcDispatch(cmd, id, data)
                            },
                            Message::Js,
                        ),
                        "profiles::save" => Task::perform(
                            async move {
                                let data = match save_profiles(&payload).await {
                                    Ok(()) => "{}".to_string(),
                                    Err(err) => IpcError { error: err.to_string() }.serialize_json(),
                                };

                                Event::IpcDispatch(cmd, id, data)
                            },
                            Message::Js,
                        ),
                        "mods::uninstall" => Task::perform(
                            async move {
                                let result = match UninstallModConfig::deserialize_json(&payload) {
                                    Ok(config) => tokio::task::spawn_blocking(move || {
                                        bepinex_lib::uninstall_mod(&config.full_name, &profile_from_dir(&config.profile_dir))
                                    })
                                    .await
                                    .map_err(|err| err.to_string())
                                    .and_then(|r| r.map_err(|err| err.to_string())),
                                    Err(err) => Err(err.to_string()),
                                };

                                let data = match result {
                                    Ok(()) => "{}".to_string(),
                                    Err(error) => IpcError { error }.serialize_json(),
                                };

                                Event::IpcDispatch(cmd, id, data)
                            },
                            Message::Js,
                        ),
                        "mods::set-enabled" => Task::perform(
                            async move {
                                let result = match SetModEnabledConfig::deserialize_json(&payload) {
                                    Ok(config) => tokio::task::spawn_blocking(move || {
                                        bepinex_lib::set_mod_enabled(&config.full_name, &profile_from_dir(&config.profile_dir), config.enabled)
                                    })
                                    .await
                                    .map_err(|err| err.to_string())
                                    .and_then(|r| r.map_err(|err| err.to_string())),
                                    Err(err) => Err(err.to_string()),
                                };

                                let data = match result {
                                    Ok(()) => "{}".to_string(),
                                    Err(error) => IpcError { error }.serialize_json(),
                                };

                                Event::IpcDispatch(cmd, id, data)
                            },
                            Message::Js,
                        ),
                        "profiles::delete-dir" => Task::perform(
                            async move {
                                let data = match ProfileDirConfig::deserialize_json(&payload) {
                                    Ok(config) => {
                                        let dir = profile_from_dir(&config.profile_dir).root().to_path_buf();
                                        let shown = dir.to_string_lossy().to_string();
                                        let app = app_dir().unwrap_or_default();
                                        match tokio::task::spawn_blocking(move || delete_profile_dir(&dir, &app)).await {
                                            Ok(Ok(deleted)) => DeleteDirResult { deleted, dir: shown }.serialize_json(),
                                            Ok(Err(err)) => IpcError { error: err.to_string() }.serialize_json(),
                                            Err(err) => IpcError { error: err.to_string() }.serialize_json(),
                                        }
                                    }
                                    Err(err) => IpcError { error: err.to_string() }.serialize_json(),
                                };

                                Event::IpcDispatch(cmd, id, data)
                            },
                            Message::Js,
                        ),
                        "launch-game" => {
                            let settings = state.settings.clone();
                            Task::perform(
                            async move {
                                let launch = move |config: LaunchGameConfig| -> Result<(), bepinex_lib::LibError> {
                                    let steam = settings.steam()?;
                                    let profile = profile_from_dir(&config.profile_dir);
                                    bepinex_lib::link_profile(&profile, steam.game_dir())?;
                                    bepinex_lib::Launcher::new(steam, profile).launch(
                                        bepinex_lib::LaunchMode::Modded,
                                        settings.launch_method(),
                                        config.launch_args.split_whitespace(),
                                    )?;
                                    Ok(())
                                };

                                let result = match LaunchGameConfig::deserialize_json(&payload) {
                                    Ok(config) => tokio::task::spawn_blocking(move || launch(config))
                                        .await
                                        .map_err(|err| err.to_string())
                                        .and_then(|r| r.map_err(|err| err.to_string())),
                                    Err(err) => Err(err.to_string()),
                                };

                                let data = match result {
                                    Ok(()) => "{}".to_string(),
                                    Err(error) => IpcError { error }.serialize_json(),
                                };

                                Event::IpcDispatch(cmd, id, data)
                            },
                            Message::Js,
                        )
                        }
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
