mod app;
mod http;
mod query;
mod screens;
mod state;

use crate::app::Application;

fn main() -> iced::Result {
    iced::application(Application::new, Application::update, Application::view)
        .theme(Application::theme)
        .run()
}
