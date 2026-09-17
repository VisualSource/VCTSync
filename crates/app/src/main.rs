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
mod widgets;
use iced::{Font, Size};

use crate::app::Application;

#[macro_export]
macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/icons/", $name)
    };
}

macro_rules! font {
    ($name: literal) => {
        include_bytes!(concat!("../../../assets/fonts/", $name, ".ttf"))
    };
}

fn main() -> iced::Result {
    let geist_regular = font!("Geist-Regular");
    let geist_medium = font!("Geist-Medium");
    let geist_bold = font!("Geist-Bold");
    let geist_semibold = font!("Geist-SemiBold");
    let geist_mono_regular = font!("GeistMono-Regular");
    let geist_mono_medium = font!("GeistMono-Medium");
    let geist_mono_bold = font!("GeistMono-Bold");
    let geist_mono_semibold = font!("GeistMono-SemiBold");

    let win_settings = iced::window::Settings {
        size: Size::new(480.0, 800.0),
        ..Default::default()
    };

    let settings = iced::Settings {
        id: Some("io.github.visualsource.voidcrew-launcher".into()),
        fonts: vec![
            geist_regular.into(),
            geist_medium.into(),
            geist_bold.into(),
            geist_semibold.into(),
            geist_mono_medium.into(),
            geist_mono_bold.into(),
            geist_mono_semibold.into(),
            geist_mono_regular.into(),
        ],
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
