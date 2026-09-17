use crate::state::Message;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ToggleVCTLinesOnly,
    ExportLogFile,
    ExportToAgent,

    Expand,
    Dismiss,
}

impl Into<Message> for Action {
    fn into(self) -> Message {
        Message::LogMessage(self)
    }
}
