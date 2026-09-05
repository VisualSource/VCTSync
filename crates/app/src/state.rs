use crate::{
    query::QueryUpdate,
    screens::{self, builds::requests::Version},
};

#[derive(Debug)]
pub enum Screen {
    Builds(screens::builds::Screen),

    Logs,
    Files,
}
#[derive(Debug)]
pub struct State {
    pub screen: Screen,
}

#[derive(Debug, Clone, Default)]
pub enum Tab {
    Logs,
    #[default]
    Builds,
    Files,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTab(Tab),

    BuildsMessage(screens::builds::Action),

    Version,
}
