use crate::state::Message;

#[derive(Debug, Clone)]
pub enum Action {
    AgentAddressChange(String),
    AgentTokenChange(String),
    AgentSaveDirChange(String),

    SaveSettings,
}

impl Into<Message> for Action {
    fn into(self) -> Message {
        Message::SettingsMessage(self)
    }
}
