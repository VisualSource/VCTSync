use crate::state::Message;

#[derive(Debug, Clone)]
pub enum Action {}

impl Into<Message> for Action {
    fn into(self) -> Message {
        Message::ProfilesMessage(self)
    }
}
