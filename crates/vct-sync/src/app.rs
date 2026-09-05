use std::collections::VecDeque;
use std::path::PathBuf;

use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable, space, text, text_input,
};
use iced::{Element, Fill, Subscription, Task, Theme};

use vct_core::build::LocalBuild;
use vct_core::config::GuiConfig;
use vct_core::github::{self, Release};
use vct_core::protocol::Event;
use vct_core::{MOD_LOG_TAG, install};

use crate::config;
use crate::net::{Agent, LogMsg, SseMsg};

const LOG_CAP: usize = 4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Builds,
    Logs,
    Transfer,
    Settings,
}

#[derive(Debug, Default)]
enum Fetch<T> {
    #[default]
    Idle,
    Loading,
    Done(T),
    Failed(String),
}

pub struct App {
    config_path: PathBuf,
    cfg: GuiConfig,
    tab: Tab,

    agent: Option<Agent>,
    agent_online: bool,

    releases: Fetch<Vec<Release>>,
    builds: Fetch<Vec<LocalBuild>>,
    profiles: Vec<String>,
    installed: Option<(String, bool)>,
    installed_marker: Option<install::InstalledMarker>,
    busy: bool,
    build_status: Option<String>,

    log_lines: VecDeque<String>,
    log_tag_only: bool,
    log_status: Option<String>,

    outbox: Vec<vct_core::protocol::OutboxItem>,
    screenshots: Vec<PathBuf>,
    transfer_status: Option<String>,

    ed_agent: String,
    ed_token: String,
    ed_save_dir: String,
    settings_status: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tab(Tab),

    RefreshReleases,
    ReleasesLoaded(Result<Vec<Release>, String>),
    RefreshBuilds,
    BuildsLoaded(Result<Vec<LocalBuild>, String>),
    ProfileSelected(String),
    RefreshProfiles,

    InstallRelease(usize),
    InstallLocal(String),
    Installed(Result<String, String>),

    ToggleLogTagOnly(bool),
    Log(LogMsg),
    CollectLog,
    LogCollected(Result<String, String>),

    PickAndSend,
    Picked(Option<PathBuf>),
    SendPath(PathBuf),
    Sent(Result<String, String>),
    RefreshOutbox,
    OutboxLoaded(Result<Vec<vct_core::protocol::OutboxItem>, String>),
    SaveIncoming(String),
    IncomingSaved(Result<String, String>),
    Sse(SseMsg),

    EdAgent(String),
    EdToken(String),
    EdSaveDir(String),
    SaveSettings,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let (config_path, cfg) = config::load();
        let agent = Agent::new(&cfg.agent, &cfg.token);
        let app = App {
            ed_agent: cfg.agent.clone(),
            ed_token: cfg.token.clone(),
            ed_save_dir: cfg
                .save_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            config_path,
            cfg,
            tab: Tab::Builds,
            agent,
            agent_online: false,
            releases: Fetch::Idle,
            builds: Fetch::Idle,
            profiles: Vec::new(),
            installed: None,
            installed_marker: None,
            busy: false,
            build_status: None,
            log_lines: VecDeque::new(),
            log_tag_only: false,
            log_status: None,
            outbox: Vec::new(),
            screenshots: recent_screenshots(),
            transfer_status: None,
            settings_status: None,
        };
        let boot = Task::batch([
            Task::done(Message::RefreshReleases),
            Task::done(Message::RefreshBuilds),
            Task::done(Message::RefreshProfiles),
            Task::done(Message::RefreshOutbox),
        ]);
        (app, boot)
    }

    pub fn theme(&self) -> Theme {
        Theme::Nord
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = Vec::new();
        if let Some(agent) = &self.agent {
            subs.push(Subscription::run_with(agent.clone(), crate::net::sse_sub).map(Message::Sse));
        }
        if let Some(path) = self.log_path() {
            subs.push(Subscription::run_with(path, crate::net::log_sub).map(Message::Log));
        }
        Subscription::batch(subs)
    }

    fn log_path(&self) -> Option<PathBuf> {
        let profile = self.cfg.profile.as_deref()?;
        let df = install::resolve_data_folder(self.cfg.tmm_data_folder.as_deref()).ok()?;
        Some(install::log_path(&install::profile_dir(&df, profile)))
    }

    fn profile_dir(&self) -> Option<PathBuf> {
        let profile = self.cfg.profile.as_deref()?;
        let df = install::resolve_data_folder(self.cfg.tmm_data_folder.as_deref()).ok()?;
        Some(install::profile_dir(&df, profile))
    }

    fn refresh_installed(&mut self) {
        let pd = self.profile_dir();
        self.installed = pd.as_deref().and_then(install::installed_version);
        self.installed_marker = pd.as_deref().and_then(install::installed_marker);
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Tab(t) => {
                self.tab = t;
                Task::none()
            }

            Message::RefreshReleases => {
                self.releases = Fetch::Loading;
                Task::perform(github::fetch_releases(), |r| {
                    Message::ReleasesLoaded(r.map_err(|e| format!("{e:#}")))
                })
            }
            Message::ReleasesLoaded(Ok(v)) => {
                self.releases = Fetch::Done(v);
                Task::none()
            }
            Message::ReleasesLoaded(Err(e)) => {
                self.releases = Fetch::Failed(e);
                Task::none()
            }

            Message::RefreshBuilds => {
                let Some(agent) = self.agent.clone() else {
                    self.builds = Fetch::Failed("no agent configured (Settings)".into());
                    return Task::none();
                };
                self.builds = Fetch::Loading;
                Task::perform(agent.list_builds(), |r| {
                    Message::BuildsLoaded(r.map_err(|e| format!("{e:#}")))
                })
            }
            Message::BuildsLoaded(Ok(v)) => {
                self.builds = Fetch::Done(v);
                Task::none()
            }
            Message::BuildsLoaded(Err(e)) => {
                self.builds = Fetch::Failed(e);
                Task::none()
            }

            Message::RefreshProfiles => {
                if let Ok(df) =
                    install::resolve_data_folder(self.cfg.tmm_data_folder.as_deref())
                {
                    self.profiles = install::list_profiles(&df).unwrap_or_default();
                }
                self.refresh_installed();
                Task::none()
            }
            Message::ProfileSelected(p) => {
                self.cfg.profile = Some(p);
                let _ = config::save(&self.config_path, &self.cfg);
                self.log_lines.clear();
                self.refresh_installed();
                Task::none()
            }

            Message::InstallRelease(i) => {
                let Fetch::Done(list) = &self.releases else {
                    return Task::none();
                };
                let Some(rel) = list.get(i) else {
                    return Task::none();
                };
                let Some(asset) = rel.mod_asset() else {
                    self.build_status = Some("release has no zip asset".into());
                    return Task::none();
                };
                let Some(pd) = self.profile_dir() else {
                    self.build_status = Some("select a profile in Settings first".into());
                    return Task::none();
                };
                self.busy = true;
                self.build_status = Some(format!("installing {}…", rel.display_name()));
                let url = asset.browser_download_url.clone();
                let tag = rel.tag_name.clone();
                let source = format!("github:{}", tag.trim_start_matches(['v', 'V']));
                Task::perform(install_from_url(url, tag, pd, source), Message::Installed)
            }
            Message::InstallLocal(id) => {
                let Some(agent) = self.agent.clone() else {
                    return Task::none();
                };
                let Some(pd) = self.profile_dir() else {
                    self.build_status = Some("select a profile in Settings first".into());
                    return Task::none();
                };
                let source = match &self.builds {
                    Fetch::Done(list) => list
                        .iter()
                        .find(|b| b.id == id)
                        .map(|b| {
                            let cfg = if b.config.is_empty() { "build" } else { &b.config };
                            format!("local:{} {}", b.version, cfg)
                        })
                        .unwrap_or_else(|| "local".into()),
                    _ => "local".into(),
                };
                self.busy = true;
                self.build_status = Some("installing local build…".into());
                Task::perform(install_from_agent(agent, id, pd, source), Message::Installed)
            }
            Message::Installed(res) => {
                self.busy = false;
                self.build_status = Some(match res {
                    Ok(s) => s,
                    Err(e) => format!("install failed: {e}"),
                });
                self.refresh_installed();
                Task::none()
            }

            Message::ToggleLogTagOnly(v) => {
                self.log_tag_only = v;
                Task::none()
            }
            Message::Log(LogMsg::Reset) => {
                self.log_lines.clear();
                Task::none()
            }
            Message::Log(LogMsg::Lines(lines)) => {
                for l in lines {
                    self.log_lines.push_back(l);
                }
                while self.log_lines.len() > LOG_CAP {
                    self.log_lines.pop_front();
                }
                Task::none()
            }
            Message::CollectLog => {
                let Some(path) = self.log_path() else {
                    self.log_status = Some("no log path (select a profile)".into());
                    return Task::none();
                };
                let agent = self.agent.clone();
                let tag_only = self.log_tag_only;
                let save_dir = self.resolved_save_dir();
                self.log_status = Some("collecting…".into());
                Task::perform(
                    collect_log(path, tag_only, save_dir, agent),
                    Message::LogCollected,
                )
            }
            Message::LogCollected(res) => {
                self.log_status = Some(match res {
                    Ok(s) => s,
                    Err(e) => format!("collect failed: {e}"),
                });
                Task::none()
            }

            Message::PickAndSend => Task::perform(pick_file(), Message::Picked),
            Message::Picked(Some(p)) => Task::done(Message::SendPath(p)),
            Message::Picked(None) => Task::none(),
            Message::SendPath(p) => {
                let Some(agent) = self.agent.clone() else {
                    self.transfer_status = Some("no agent configured (Settings)".into());
                    return Task::none();
                };
                self.transfer_status = Some(format!("sending {}…", p.display()));
                Task::perform(send_file(agent, p), Message::Sent)
            }
            Message::Sent(res) => {
                self.transfer_status = Some(match res {
                    Ok(s) => s,
                    Err(e) => format!("send failed: {e}"),
                });
                Task::none()
            }
            Message::RefreshOutbox => {
                let Some(agent) = self.agent.clone() else {
                    return Task::none();
                };
                Task::perform(agent.outbox_list(), |r| {
                    Message::OutboxLoaded(r.map_err(|e| format!("{e:#}")))
                })
            }
            Message::OutboxLoaded(Ok(v)) => {
                self.outbox = v;
                Task::none()
            }
            Message::OutboxLoaded(Err(e)) => {
                self.transfer_status = Some(e);
                Task::none()
            }
            Message::SaveIncoming(name) => {
                let Some(agent) = self.agent.clone() else {
                    return Task::none();
                };
                let dir = self.resolved_save_dir();
                Task::perform(save_incoming(agent, name, dir), Message::IncomingSaved)
            }
            Message::IncomingSaved(res) => {
                self.transfer_status = Some(match res {
                    Ok(s) => s,
                    Err(e) => format!("save failed: {e}"),
                });
                Task::done(Message::RefreshOutbox)
            }

            Message::Sse(SseMsg::Connected) => {
                self.agent_online = true;
                Task::none()
            }
            Message::Sse(SseMsg::Disconnected) => {
                self.agent_online = false;
                Task::none()
            }
            Message::Sse(SseMsg::Event(Event::Build { build })) => {
                self.build_status = Some(format!(
                    "new build available: {} ({})",
                    build.version, build.config
                ));
                Task::done(Message::RefreshBuilds)
            }
            Message::Sse(SseMsg::Event(Event::File { item })) => {
                self.transfer_status =
                    Some(format!("file available from Linux: {}", item.name));
                if self.cfg.auto_save_incoming {
                    Task::done(Message::SaveIncoming(item.name))
                } else {
                    Task::done(Message::RefreshOutbox)
                }
            }
            Message::Sse(SseMsg::Event(Event::Ping { .. })) => Task::none(),

            Message::EdAgent(s) => {
                self.ed_agent = s;
                Task::none()
            }
            Message::EdToken(s) => {
                self.ed_token = s;
                Task::none()
            }
            Message::EdSaveDir(s) => {
                self.ed_save_dir = s;
                Task::none()
            }
            Message::SaveSettings => {
                self.cfg.agent = self.ed_agent.trim().to_string();
                self.cfg.token = self.ed_token.trim().to_string();
                self.cfg.save_dir = {
                    let s = self.ed_save_dir.trim();
                    (!s.is_empty()).then(|| PathBuf::from(s))
                };
                match config::save(&self.config_path, &self.cfg) {
                    Ok(()) => self.settings_status = Some("saved".into()),
                    Err(e) => self.settings_status = Some(format!("save failed: {e}")),
                }
                self.agent = Agent::new(&self.cfg.agent, &self.cfg.token);
                self.agent_online = false;
                Task::batch([
                    Task::done(Message::RefreshBuilds),
                    Task::done(Message::RefreshOutbox),
                ])
            }
        }
    }

    fn resolved_save_dir(&self) -> PathBuf {
        self.cfg
            .save_dir
            .clone()
            .or_else(|| dirs_download())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    // ---- view ----------------------------------------------------------------

    pub fn view(&self) -> Element<'_, Message> {
        let tab_btn = |label, t: Tab| {
            let b = button(text(label)).on_press(Message::Tab(t));
            if self.tab == t { b } else { b.style(button::secondary) }
        };
        let dot = if self.agent_online { "●" } else { "○" };
        let header = row![
            tab_btn("Builds", Tab::Builds),
            tab_btn("Logs", Tab::Logs),
            tab_btn("Transfer", Tab::Transfer),
            tab_btn("Settings", Tab::Settings),
            space().width(Fill),
            text(format!("agent {dot}")),
        ]
        .spacing(6)
        .padding(8);

        let body = match self.tab {
            Tab::Builds => self.view_builds(),
            Tab::Logs => self.view_logs(),
            Tab::Transfer => self.view_transfer(),
            Tab::Settings => self.view_settings(),
        };

        column![header, rule::horizontal(1), container(body).padding(10)].into()
    }

    fn view_builds(&self) -> Element<'_, Message> {
        let installed = match (&self.installed, &self.installed_marker) {
            (Some((v, enabled)), marker) => {
                let mut s = format!("installed: {v}");
                if !enabled {
                    s.push_str(" (disabled)");
                }
                if let Some(m) = marker {
                    s.push_str(&format!(
                        "  ·  {}  ·  {}",
                        short_hash(&m.sha256),
                        fmt_time(m.installed_at)
                    ));
                }
                s
            }
            (None, Some(m)) => format!(
                "installed: {} · {} · {}",
                m.version,
                short_hash(&m.sha256),
                fmt_time(m.installed_at)
            ),
            (None, None) => "installed: none".into(),
        };
        let profile = self.cfg.profile.as_deref().unwrap_or("<no profile>");

        let releases: Element<_> = match &self.releases {
            Fetch::Idle => text("—").into(),
            Fetch::Loading => text("loading…").into(),
            Fetch::Failed(e) => text(format!("error: {e}")).into(),
            Fetch::Done(list) if list.is_empty() => text("no published releases").into(),
            Fetch::Done(list) => column(list.iter().enumerate().map(|(i, r)| {
                let tag = if r.prerelease { " [prerelease]" } else { "" };
                row![
                    text(format!("{}{tag}", r.display_name())).width(Fill),
                    text(r.published_at.as_deref().unwrap_or("")).size(12),
                    button("Install")
                        .style(button::secondary)
                        .on_press_maybe((!self.busy).then_some(Message::InstallRelease(i))),
                    space().width(12),
                ]
                .spacing(8)
                .into()
            }))
            .spacing(4)
            .into(),
        };

        let builds: Element<_> = match &self.builds {
            Fetch::Idle => text("—").into(),
            Fetch::Loading => text("loading…").into(),
            Fetch::Failed(e) => text(format!("error: {e}")).into(),
            Fetch::Done(list) if list.is_empty() => text("no local builds").into(),
            Fetch::Done(list) => column(list.iter().map(|b| {
                row![
                    text(format!("{}  {}", b.version, b.config)).width(Fill),
                    text(fmt_time(b.mtime)).size(12),
                    button("Install")
                        .style(button::secondary)
                        .on_press_maybe(
                            (!self.busy).then_some(Message::InstallLocal(b.id.clone()))
                        ),
                    space().width(12),
                ]
                .spacing(8)
                .into()
            }))
            .spacing(4)
            .into(),
        };

        column![
            row![
                text(installed).width(Fill),
                text(format!("profile: {profile}")).size(12)
            ],
            status_line(self.build_status.as_deref()),
            space().height(6),
            row![
                text("GitHub releases").width(Fill),
                button("Refresh").style(button::text).on_press(Message::RefreshReleases)
            ],
            scrollable(releases).height(200),
            space().height(10),
            row![
                text("Local builds").width(Fill),
                button("Refresh").style(button::text).on_press(Message::RefreshBuilds)
            ],
            scrollable(builds).height(200),
        ]
        .spacing(4)
        .into()
    }

    fn view_logs(&self) -> Element<'_, Message> {
        let path = self
            .log_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "no profile selected (Settings)".into());

        let shown = self.log_lines.iter().filter(|l| {
            !self.log_tag_only || l.contains(MOD_LOG_TAG)
        });
        let log_view = column(shown.map(|l| text(l.clone()).size(12).into())).spacing(1);

        column![
            row![
                checkbox(self.log_tag_only)
                    .label("mod lines only")
                    .on_toggle(Message::ToggleLogTagOnly),
                space().width(Fill),
                button("Collect → Linux").on_press(Message::CollectLog),
            ]
            .spacing(8),
            text(path).size(11),
            status_line(self.log_status.as_deref()),
            rule::horizontal(1),
            scrollable(log_view).height(Fill).anchor_bottom(),
        ]
        .spacing(6)
        .into()
    }

    fn view_transfer(&self) -> Element<'_, Message> {
        let incoming: Element<_> = if self.outbox.is_empty() {
            text("nothing waiting").into()
        } else {
            column(self.outbox.iter().map(|it| {
                row![
                    text(format!("{}  ({} B)", it.name, it.size)).width(Fill),
                    button("Save").style(button::secondary)
                        .on_press(Message::SaveIncoming(it.name.clone())),
                ]
                .spacing(8)
                .into()
            }))
            .spacing(4)
            .into()
        };

        let shots: Element<_> = if self.screenshots.is_empty() {
            text("no recent screenshots found").into()
        } else {
            column(self.screenshots.iter().map(|p| {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                row![
                    text(name).width(Fill),
                    button("Send").style(button::secondary)
                        .on_press(Message::SendPath(p.clone())),
                ]
                .spacing(8)
                .into()
            }))
            .spacing(4)
            .into()
        };

        column![
            row![
                button("Send a file…").on_press(Message::PickAndSend),
                space().width(Fill),
                button("Refresh").style(button::text).on_press(Message::RefreshOutbox),
            ],
            status_line(self.transfer_status.as_deref()),
            space().height(6),
            text("Waiting from Linux"),
            incoming,
            space().height(10),
            text("Recent screenshots"),
            scrollable(shots).height(180),
        ]
        .spacing(4)
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        let profiles = self.profiles.clone();
        column![
            text("Agent address"),
            text_input("192.168.1.94:9787", &self.ed_agent).on_input(Message::EdAgent),
            text("Token"),
            text_input("shared bearer token", &self.ed_token).on_input(Message::EdToken),
            text("Save directory (received files, collected logs)"),
            text_input("(defaults to Downloads)", &self.ed_save_dir)
                .on_input(Message::EdSaveDir),
            space().height(6),
            row![
                text("TMM profile").width(Fill),
                pick_list(profiles, self.cfg.profile.clone(), Message::ProfileSelected),
                button("Rescan").style(button::text).on_press(Message::RefreshProfiles),
            ]
            .spacing(8),
            space().height(10),
            button("Save settings").on_press(Message::SaveSettings),
            status_line(self.settings_status.as_deref()),
        ]
        .spacing(4)
        .max_width(560)
        .into()
    }
}

// ---- async helpers ---------------------------------------------------------

async fn install_from_url(
    url: String,
    tag: String,
    profile_dir: PathBuf,
    source: String,
) -> Result<String, String> {
    let bytes = github::download_asset(&url).await.map_err(|e| format!("{e:#}"))?;
    run_install(bytes, profile_dir, Some(tag), source).await
}

async fn install_from_agent(
    agent: Agent,
    id: String,
    profile_dir: PathBuf,
    source: String,
) -> Result<String, String> {
    let bytes = agent.download_build(id).await.map_err(|e| format!("{e:#}"))?;
    run_install(bytes, profile_dir, None, source).await
}

async fn run_install(
    bytes: Vec<u8>,
    profile_dir: PathBuf,
    fallback: Option<String>,
    source: String,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        install::install(&bytes, &profile_dir, fallback.as_deref(), &source)
    })
    .await
    .map_err(|e| e.to_string())?
    .map(|r| {
        format!(
            "installed {} — {} files{}",
            r.version,
            r.files_written,
            if r.mods_yml_updated {
                ", mods.yml updated"
            } else {
                ", mods.yml entry added"
            }
        )
    })
    .map_err(|e| format!("{e:#}"))
}

async fn collect_log(
    path: PathBuf,
    tag_only: bool,
    save_dir: PathBuf,
    agent: Option<Agent>,
) -> Result<String, String> {
    let raw = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("reading log: {e}"))?;
    let body = if tag_only {
        raw.lines()
            .filter(|l| l.contains(MOD_LOG_TAG))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    } else {
        raw
    };
    let kind = if tag_only { "log" } else { "LogOutput" };
    let name = format!("VoidCrewTerminus-{kind}-{}.log", timestamp());

    tokio::fs::create_dir_all(&save_dir).await.ok();
    let local = save_dir.join(&name);
    tokio::fs::write(&local, &body)
        .await
        .map_err(|e| format!("writing {}: {e}", local.display()))?;

    match agent {
        Some(a) => {
            a.upload_file(name.clone(), body.into_bytes())
                .await
                .map_err(|e| format!("saved locally, upload failed: {e:#}"))?;
            Ok(format!("saved {} and sent to Linux", local.display()))
        }
        None => Ok(format!("saved {} (no agent configured)", local.display())),
    }
}

async fn pick_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_file()
        .await
        .map(|h| h.path().to_path_buf())
}

async fn send_file(agent: Agent, path: PathBuf) -> Result<String, String> {
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| format!("reading {}: {e}", path.display()))?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| "bad file name".to_string())?;
    agent
        .upload_file(name.clone(), bytes)
        .await
        .map_err(|e| format!("{e:#}"))?;
    Ok(format!("sent {name} to Linux"))
}

async fn save_incoming(
    agent: Agent,
    name: String,
    dir: PathBuf,
) -> Result<String, String> {
    let bytes = agent
        .clone()
        .outbox_get(name.clone())
        .await
        .map_err(|e| format!("{e:#}"))?;
    tokio::fs::create_dir_all(&dir).await.ok();
    let dest = dir.join(&name);
    tokio::fs::write(&dest, &bytes)
        .await
        .map_err(|e| format!("writing {}: {e}", dest.display()))?;
    agent.outbox_delete(name).await.ok();
    Ok(format!("saved {}", dest.display()))
}

// ---- misc ----------------------------------------------------------------

fn short_hash(h: &str) -> &str {
    &h[..h.len().min(8)]
}

/// Unix seconds → `YYYY-MM-DD HH:MM` (UTC).
fn fmt_time(unix_secs: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(unix_secs)
        .ok()
        .and_then(|t| {
            let f = time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]"
            );
            t.format(&f).ok()
        })
        .unwrap_or_else(|| unix_secs.to_string())
}

fn status_line(s: Option<&str>) -> Element<'_, Message> {
    match s {
        Some(t) => text(t.to_owned()).size(12).into(),
        None => space().height(0).into(),
    }
}

fn timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    let f = time::macros::format_description!("[year]-[month]-[day]_[hour]-[minute]-[second]");
    now.format(&f).unwrap_or_else(|_| "now".into())
}

fn home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn dirs_download() -> Option<PathBuf> {
    home().map(|h| h.join("Downloads"))
}

fn recent_screenshots() -> Vec<PathBuf> {
    let Some(h) = home() else {
        return Vec::new();
    };
    let dirs = [h.join("Pictures").join("Screenshots"), h.join("Downloads")];
    let exts = ["png", "jpg", "jpeg", "bmp", "gif"];
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for d in dirs {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            let is_img = p
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| exts.contains(&x.to_lowercase().as_str()))
                .unwrap_or(false);
            if !is_img {
                continue;
            }
            if let Ok(m) = e.metadata().and_then(|m| m.modified()) {
                files.push((m, p));
            }
        }
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    files.into_iter().take(12).map(|(_, p)| p).collect()
}
