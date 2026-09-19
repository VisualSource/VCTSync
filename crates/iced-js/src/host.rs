use crate::RootId;
use crate::render::Node;
use crate::runtime::{Event, JsCmd};
use iced::Task;
use iced::futures::channel::mpsc::Sender;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};
pub struct Host {
    id: u64,
    tx: Option<Sender<JsCmd>>,
    pending: Vec<(RootId, PathBuf)>,
    trees: HashMap<RootId, Arc<Node>>,
}

impl Host {
    pub fn new(scripts: impl IntoIterator<Item = (String, String)>) {
        unimplemented!()
    }

    pub fn mount(&mut self, id: &str) -> Task<Event> {
        unimplemented!()
    }
    pub fn unmount(&mut self, id: &str) -> Task<Event> {
        unimplemented!()
    }

    pub fn update(&mut self, ev: Event) -> Task<Event> {
        unimplemented!()
    }
}
