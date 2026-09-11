use crate::{screens::settings::state::Action, state::Message, traits::IcedScreen};
use iced::Task;
use iced_query::QueryClient;
use iced_xml::ui;

pub mod state;

#[derive(Debug, Clone)]
pub struct Screen {
    agent_address: String,
    agent_token: String,
    agent_save_dir: String,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            agent_address: String::default(),
            agent_token: String::default(),
            agent_save_dir: String::default(),
        }
    }
}

impl IcedScreen<state::Action> for Screen {
    fn view(&self) -> iced::Element<'_, Message> {
        ui! {
         <col>

            <text>"Dev Agent"</text>
            <col>
                Agent Address
                <input onChange={|value|Action::AgentAddressChange(value).into()} type="text" value={&self.agent_address} placeholder="192.168.1.94:9787" />

                Token
                <input onChange={|value|Action::AgentTokenChange(value).into()} type="text" value={&self.agent_token} placeholder="shared bearer token"/>
                <text>"Save Dir (default downloads)"</text>
                <input onChange={|value|Action::AgentSaveDirChange(value).into()} type="text" value={&self.agent_save_dir}  placeholder="C:\\Downloads"/>

                <space height={8}/>
            </col>

            <button onPress={Message::SettingsMessage(Action::SaveSettings)}>Save Settings</button>
         </col>
        }
    }

    fn update(&mut self, ev: state::Action, _client: &QueryClient) -> iced::Task<Message> {
        match ev {
            Action::AgentAddressChange(v) => self.agent_address = v,
            Action::AgentTokenChange(v) => self.agent_token = v,
            Action::AgentSaveDirChange(v) => self.agent_save_dir = v,
            Action::SaveSettings => {}
        }

        Task::none()
    }

    fn mount(&self, _client: &QueryClient) -> iced::Task<Message> {
        Task::none()
    }

    fn sync(&mut self, _client: &QueryClient) {}
}
