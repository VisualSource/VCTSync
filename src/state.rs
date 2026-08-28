use crate::screens;

#[derive(Debug)]
pub enum Screen {
    Logs,
    Versions(screens::versions::VersionsScreen),
    Files,
}

#[derive(Debug)]
pub struct State {
    pub screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetScreenLogs,
    SetScreenVersions,
    SetScreenFiles,

    InstallModVersion,
}
