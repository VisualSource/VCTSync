use nanoserde::{DeJson, DeRon, SerJson, SerRon};

#[derive(Clone)]
pub enum Message {
    Js(iced_js::Event),
    Reload,
}

#[derive(Debug, DeJson, SerJson, SerRon, DeRon)]
pub struct AppSettings {}

impl AppSettings {
    pub fn new() -> Self {
        Self {}
    }
}
