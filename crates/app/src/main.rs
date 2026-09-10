mod app;
mod http;
mod screens;
mod state;

use crate::app::Application;

#[macro_export]
macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/", $name)
    };
}

fn main() -> iced::Result {
    iced::application(Application::new, Application::update, Application::view)
        .title("Terminus Updater")
        .theme(Application::theme)
        .run()
}
