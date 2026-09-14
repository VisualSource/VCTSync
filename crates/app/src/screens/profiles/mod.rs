use iced::{Alignment, Length, Task};
use iced_query::QueryClient;
use iced_xml::ui;

use crate::{asset, state::Message, traits::IcedScreen, utils::style_svg};

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
            <row padding={4} alignY={Alignment::Center}>
                <text>"Active Profile: None"</text>
                <space width={iced::Fill}/>
                <button>Launch</button>
            </row>
            <hr/>
            <space height={10}/>
            {if self.profiles.is_empty() {
               ui!{
                <view height={iced::Fill} center={Length::Fill} width={Length::Fill}>
                    <col center alignX={Alignment::Center} spacing={6}>
                        <text center>"No profiles"</text>
                        <button>
                            <row center width={iced::Shrink} spacing={6}>
                                Create
                                <svg src={asset!("layers-plus.svg")} style={style_svg}/>
                            </row>
                        </button>
                    </col>
                </view>
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
