use std::sync::Arc;

use iced::task::{Straw, sipper};

#[derive(Debug, Clone)]

pub struct Mod {
    name: String,
    icon: String,
}

#[derive(Debug, Clone)]
pub struct Profile {
    name: String,
    mods: Vec<Mod>,
}

pub fn profile_fetch() -> impl Straw<Vec<Profile>, (), Arc<anyhow::Error>> {
    sipper(async move |_| Ok(vec![]))
}
