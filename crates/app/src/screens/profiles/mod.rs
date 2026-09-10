use iced::{Element, Task};
use iced_query::QueryClient;
use iced_xml::ui;

use crate::{asset, state::Message};

pub mod state;

#[derive(Debug, Clone, Default)]
pub struct Screen {
    profiles: Vec<()>,
    active_profile: Option<()>,
}

impl Screen {
    pub fn init(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, state: state::Action) -> Task<state::Action> {
        Task::none()
    }

    pub fn sync(&mut self, client: &QueryClient) {}

    pub fn view(&self) -> Element<'_, Message> {
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
}
