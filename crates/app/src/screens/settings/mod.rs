use iced::Element;
use iced_query::QueryClient;
use iced_xml::ui;

use crate::state::Message;

pub mod state;

#[derive(Debug, Clone)]
pub struct Screen {}

impl Screen {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ev: state::Action) {}

    pub fn sync(&mut self, client: &QueryClient) {}

    pub fn view(&self) -> Element<'_, Message> {
        ui! {
            <col>
            </col>
        }
    }
}
