use crate::screens::{self};

#[derive(Debug)]
pub enum Screen {
    Builds(screens::builds::Screen),
    Profiles(screens::profiles::Screen),

    Logs,
    Files,
}

#[derive(Debug, Clone, Default)]
pub enum Tab {
    Logs,
    #[default]
    Builds,
    Files,
    Profiles,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTab(Tab),

    BuildsMessage(screens::builds::Action),
    ProfilesMessage(screens::profiles::state::Action),
}
