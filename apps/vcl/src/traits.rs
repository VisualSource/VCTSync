use iced::Task;
use iced_query::QueryClient;

use crate::state::Message;

pub trait IcedScreen<A> {
    fn view(&self) -> iced::Element<'_, Message>;
    fn update(&mut self, ev: A, client: &QueryClient) -> Task<Message>;

    fn mount(&self, client: &QueryClient) -> Task<Message>;
    fn sync(&mut self, client: &QueryClient);
}
