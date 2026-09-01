use crate::{
    query::QueryUpdate,
    screens::{self, versions::requests::Version},
};

#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    Logs,
    Versions(screens::versions::VersionsScreen),
    Files,
}

#[derive(Debug, Default)]
pub struct State {
    pub screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Tab {
    Logs,
    Versions,
    Files,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTab(Tab),
    Version,

    QueryUpdate(String, QueryUpdate<Vec<Version>>),
}
