use iced::{Element, Task};
use iced_xml::ui;

use crate::state::Message;

pub mod state;

#[derive(Debug, Clone, Default)]
pub struct Screen {}

impl Screen {
    pub fn init(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, state: state::Action) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        ui! {
            <col/>
        }
    }
}
