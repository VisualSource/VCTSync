#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod http;
mod screens;
mod state;
mod traits;

use crate::app::Application;

#[macro_export]
macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/", $name)
    };
}

fn main() -> iced::Result {
    iced::application(Application::new, Application::update, Application::view)
        .title("VC Launcher")
        .theme(Application::theme)
        .run()
}
