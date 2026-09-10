use iced::Task;
use iced_query::QueryClient;
use iced_xml::ui;

use crate::{asset, state::Message, traits::IcedScreen};

pub mod state;

#[derive(Debug, Clone, Default)]
pub struct Screen {
    profiles: Vec<()>,
    active_profile: Option<()>,
}

impl IcedScreen<state::Action> for Screen {
    fn view(&self) -> iced::Element<'_, Message> {
        ui! {
           <col>
            <row>
                <row>
                    <text>"Active Profile: None"</text>
                </row>
                <space width={iced::Fill}/>
                <button>Launch</button>
            </row>
            <hr/>
            <space height={10}/>
            {if self.profiles.is_empty() {
               ui!{
                 <col>
                    <text center>"No profiles"</text>
                    <button>
                        <row>
                            Create
                            <svg src={asset!("layers-plus.svg")}/>
                        </row>
                    </button>
                 </col>
               }
            } else {
                ui! {
                    <scroll>
                        <col>

                        </col>
                    </scroll>
                }
            }}
           </col>
        }
    }

    fn update(&mut self, ev: state::Action, client: &QueryClient) -> Task<Message> {
        Task::none()
    }

    fn mount(&self, client: &QueryClient) -> Task<Message> {
        Task::none()
    }

    fn sync(&mut self, client: &QueryClient) {}
}
