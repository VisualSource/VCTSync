use iced::{Alignment, Task};
use iced_xml::ui;

use crate::traits::IcedScreen;

pub mod state;

#[derive(Debug, Clone, Default)]
pub struct Screen {
    show_mod_only_lines: bool,
}

impl IcedScreen<state::Action> for Screen {
    fn view(&self) -> iced::Element<'_, crate::state::Message> {
        ui! {
            <col spacing={4}>
                <row padding={4} alignY={Alignment::Center}>
                    <input type="checkbox" checked={self.show_mod_only_lines} label="Mod lines only"/>
                    <space width={iced::Fill}/>
                    <button>Export Log</button>
                </row>
                <hr/>
                <scroll height={iced::Fill} anchorBottom>
                    <col spacing={1}>

                    </col>
                </scroll>
            </col>
        }
    }

    fn update(
        &mut self,
        ev: state::Action,
        client: &iced_query::QueryClient,
    ) -> iced::Task<crate::state::Message> {
        Task::none()
    }

    fn mount(&self, client: &iced_query::QueryClient) -> iced::Task<crate::state::Message> {
        Task::none()
    }

    fn sync(&mut self, client: &iced_query::QueryClient) {}
}
