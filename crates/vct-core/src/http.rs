//! One shared `reqwest::Client`. Both ends talk HTTP — the GUI to GitHub and to
//! the agent, the agent's `send` subcommand to a running agent.

use std::sync::OnceLock;

use reqwest::Client;

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent(concat!("vct-sync/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("failed to build http client")
    })
}
