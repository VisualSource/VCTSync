use crate::state::Message;

#[derive(Debug, Clone)]
pub enum Action {
    Launch,
    ProfileSet,
    ProfileCreate,
    ProfileDelete,
    ProfileEdit,
}

impl Into<Message> for Action {
    fn into(self) -> Message {
        Message::ProfilesMessage(self)
    }
}
