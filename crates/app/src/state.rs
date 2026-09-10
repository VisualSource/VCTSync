use crate::screens::{self};
use iced_query::QueryEvent;

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

    QueryUpdate(QueryEvent),

    BuildsMessage(screens::builds::Action),
    ProfilesMessage(screens::profiles::state::Action),
}
