//! vct-sync — the Windows GUI. Installs builds into a Thunderstore Mod Manager
//! profile, tails the BepInEx log, and moves files to/from the Linux agent.

mod app;
mod config;
mod net;

use app::App;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vct_sync=info,vct_core=info".into()),
        )
        .init();

    iced::application(App::boot, App::update, App::view)
        .title("vct-sync")
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}
