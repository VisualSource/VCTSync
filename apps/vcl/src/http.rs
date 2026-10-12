use std::{sync::OnceLock, time::Duration};

use reqwest::Client;

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent("vct-sync")
            .timeout(Duration::from_mins(1))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build http client")
    })
}
