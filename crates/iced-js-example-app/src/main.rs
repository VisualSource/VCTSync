#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use env_logger::{Builder, Target};
use iced::{
    Element, Size, Subscription, Task, Theme, event,
    keyboard::{
        self,
        key::{self, Named},
    },
    widget::{column, row},
};
use iced_js::{Event, Host, js_worker, surface};
use std::env;

use crate::Message::Reload;

#[derive(Clone)]
enum Message {
    Js(Event),
    Reload,
}

struct App {
    js: Host,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut state = Self {
            js: Host::new([("main", concat!(env!("CARGO_MANIFEST_DIR"), "/js/view.js",))]),
        };

        let mount_task = state.js.mount("main");

        (state, mount_task.map(Message::Js))
    }
    pub fn update(state: &mut App, msg: Message) -> Task<Message> {
        match msg {
            Reload => state.js.reload().map(Message::Js),
            Message::Js(ev) => state.js.update(ev).map(Message::Js), // pipe update event to host handler
        }
    }
    fn view(state: &App) -> Element<'_, Message> {
        column![row![
            // render it into iced tree
            surface(&state.js, "main").map(Message::Js)
        ]]
        .into()
    }
    pub fn theme(_: &App) -> Theme {
        Theme::Dark
    }
}

fn keyboard_listener(_state: &App) -> Subscription<Message> {
    event::listen().filter_map(|event| match event {
        iced::event::Event::Keyboard(keyboard::Event::KeyReleased {
            key: key::Key::Named(named),
            modifiers,
            ..
        }) => {
            if modifiers.is_empty() && named == Named::F5 {
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
    builder.filter_level(log::LevelFilter::Off); // silence everything by default
    builder.filter_module("iced_js", log::LevelFilter::Debug);
    builder.filter_module("iced_js_example_app", log::LevelFilter::Info);
    builder.parse_default_env(); // RUST_LOG can still override the above
    builder.target(Target::Stdout);

    builder.init();

    log::info!("Hello");

    let win_settings = iced::window::Settings {
        size: Size::new(480.0, 800.0),
        ..Default::default()
    };

    iced::application(App::new, App::update, App::view)
        .title("Iced JS Example")
        .theme(App::theme)
        .window(win_settings)
        // One call only: `subscription` replaces whatever was set before it
        // rather than adding to it, so a second call would silently drop the js
        // worker and leave the app with nothing to render.
        .subscription(|state| {
            Subscription::batch([
                Subscription::run(js_worker).map(Message::Js), // setup js worker thread
                keyboard_listener(state),
            ])
        })
        .run()
}
