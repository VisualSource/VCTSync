#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
//mod http;
//mod screens;
mod state;
//mod traits;
//mod utils;
//mod widgets;
use crate::{app::Application, state::Message};
use env_logger::{Builder, Target};
use iced::{Font, Size, Subscription};
use iced_js::{SiloAssets, js_worker};
use std::env;

static ASSETS: iced_js::rust_silos::Silo =
    iced_js::rust_silos::embed_silo!("dist", crate = iced_js::rust_silos);

macro_rules! font {
    ($name: literal) => {
        include_bytes!(concat!("../assets/fonts/", $name, ".ttf"))
    };
}

fn keyboard_listener(_state: &Application) -> Subscription<Message> {
    iced::event::listen().filter_map(|event| match event {
        iced::event::Event::Keyboard(iced::keyboard::Event::KeyReleased {
            key: iced::keyboard::Key::Named(named),
            modifiers,
            ..
        }) => {
            if modifiers.is_empty() && named == iced::keyboard::key::Named::F5 {
                Some(Message::Reload)
            } else {
                None
            }
        }
        _ => None,
    })
}

fn main() -> iced::Result {
    #[cfg(target_os = "windows")]
    unsafe {
        if env::var_os("WGPU_BACKEND").is_none() {
            env::set_var("WGPU_BACKEND", "dx12");
        }
    }

    let mut builder = Builder::new();
    builder.filter_level(log::LevelFilter::Warn); // silence everything by default
    builder.parse_default_env(); // RUST_LOG can still override the above
    builder.target(Target::Stdout);

    builder.init();

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
        .subscription(move |state| {
            Subscription::batch([
                Subscription::run_with(SiloAssets::new(&ASSETS), js_worker).map(Message::Js),
                keyboard_listener(state),
            ])
        })
        .run()
}
