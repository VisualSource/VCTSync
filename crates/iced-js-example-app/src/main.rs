use env_logger::{Builder, Target};
use iced::{
    Element, Size, Subscription, Task, Theme,
    widget::{column, row},
};
use iced_js::{Event, Host, js_worker, view};
use std::env;

#[derive(Clone)]
enum Message {
    Js(Event),
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
            Message::Js(ev) => state.js.update(ev).map(Message::Js), // pipe update event to host handler
        }
    }
    fn view(state: &App) -> Element<'_, Message> {
        column![row![
            // render it into iced tree
            view(&state.js, "main").map(Message::Js)
        ]]
        .into()
    }
    pub fn theme(_: &App) -> Theme {
        Theme::Dark
    }
}

fn main() -> iced::Result {
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
        .subscription(|_| Subscription::run(js_worker).map(Message::Js)) // setup js worker thread
        .run()
}
