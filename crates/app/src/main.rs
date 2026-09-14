#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod http;
mod screens;
mod state;
mod traits;
mod utils;

use iced::{Font, Size};

use crate::app::Application;

#[macro_export]
macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/", $name)
    };
}

fn main() -> iced::Result {
    let giest_medium = include_bytes!("../../../assets/fonts/geist/Geist-Medium.ttf");

    let win_settings = iced::window::Settings {
        size: Size::new(480.0, 800.0),
        ..Default::default()
    };

    let settings = iced::Settings {
        fonts: vec![giest_medium.into()],
        default_font: Font::with_name("Geist"),
        ..Default::default()
    };

    iced::application(Application::new, Application::update, Application::view)
        .title("VC Launcher")
        .theme(Application::theme)
        .settings(settings)
        .window(win_settings)
        .run()
}
