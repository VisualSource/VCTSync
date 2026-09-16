mod fetchers;
pub mod state;

use iced::{Alignment, Element, Length, Task, widget::Column};
use iced_query::{Query, QueryClient};
use iced_xml::ui;

use crate::{
    asset, screens::profiles::state::Action, state::Message, traits::IcedScreen, utils::style_svg,
};

use fetchers::{Profile, profile_fetch};

#[derive(Debug)]
pub struct Screen {
    profiles: Query<Vec<Profile>>,
    active_profile: Option<Profile>,
}

impl Screen {
    pub fn new(client: &QueryClient) -> Self {
        Self {
            profiles: Query::new(client, "profiles::profile_list", profile_fetch, None),
            active_profile: None,
        }
    }
}

fn view_profile<'a>(item: &'a Profile) -> Element<'a, Message> {
    ui! {
        <row>

        </row>
    }
}

impl IcedScreen<state::Action> for Screen {
    fn view(&self) -> iced::Element<'_, Message> {
        let q = &self.profiles.snapshot;

        let rv = match (&q.data, &q.error) {
            (Some(data), _) => {
                if data.is_empty() {
                    ui! {
                     <view height={iced::Fill} center={Length::Fill} width={Length::Fill}>
                         <col center alignX={Alignment::Center} spacing={6}>
                             <text center>"No profiles"</text>
                             <button onPress={Action::ProfileCreate.into()}>
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
                             {iced::Element::from(Column::with_children(
                                data.iter()
                                    .map(|profile| view_profile(profile))
                                    .map(Element::from),
                            ))}
                         </scroll>
                    }
                }
            }

            (None, Some(err)) => {
                let reason = err.to_string();
                ui! {
                    <view height={iced::Fill} center={Length::Fill} width={Length::Fill}>
                        <col center alignX={Alignment::Center} spacing={6}>
                            <text>{reason}</text>
                        </col>
                    </view>
                }
            }

            (None, None) => ui! {
            <view height={iced::Fill} center={Length::Fill} width={Length::Fill}>
                <col>
                Loading

                </col>
            </view> },
        };

        ui! {
           <col>
            <row padding={4} alignY={Alignment::Center}>
                <text>"Active Profile: None"</text>
                <space width={iced::Fill}/>
                <button onPress={Action::Launch.into()}>Launch</button>
            </row>
            <hr/>
            <space height={10}/>
            {rv}
           </col>
        }
    }

    fn update(&mut self, ev: state::Action, _client: &QueryClient) -> Task<Message> {
        match ev {
            Action::Launch => Task::none(),
            Action::ProfileSet => Task::none(),
            Action::ProfileCreate => Task::none(),
            Action::ProfileDelete => Task::none(),
            Action::ProfileEdit => Task::none(),
        }
    }

    fn mount(&self, client: &QueryClient) -> Task<Message> {
        let task = self.profiles.fetch(client).map(Message::QueryUpdate);

        task
    }

    fn sync(&mut self, client: &QueryClient) {
        self.profiles.sync(client);
    }
}
