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
    pub trees: HashMap<RootId, Arc<Node>>,
}

impl Host {
    pub fn new<K, P>(scripts: impl IntoIterator<Item = (K, P)>) -> Self
    where
        K: Into<RootId>,
        P: Into<PathBuf>,
    {
        let pending = scripts
            .into_iter()
            .map(|(root_id, path)| (root_id.into(), path.into()))
            .collect();

        Self {
            id: 0, // GET value from a global
            tx: None,
            pending,
            trees: HashMap::new(),
        }
    }

    pub fn mount(&mut self, id: &str) -> Task<Event> {
        let Some(idx) = self.pending.iter().position(|(root_id, _)| root_id == id) else {
            return Task::none();
        };

        if let Some(tx) = &mut self.tx {
            let (root_id, path) = self.pending.swap_remove(idx);
            let _ = tx.try_send(JsCmd::Mount { root_id, path });
        }

        Task::none()
    }
    pub fn unmount(&mut self, id: &str) -> Task<Event> {
        if let Some(tx) = &mut self.tx {
            let _ = tx.try_send(JsCmd::Unmount(id.to_owned()));
        }

        Task::none()
    }

    pub fn reload(&mut self) -> Task<Event> {
        if let Some(tx) = &mut self.tx {
            if let Err(err) = tx.try_send(JsCmd::Reload) {
                log::error!("{}", err);
            }
        }

        Task::none()
    }

    pub fn update(&mut self, ev: Event) -> Task<Event> {
        match ev {
            Event::Ready(mut sender) => {
                for (root_id, path) in &self.pending {
                    let _ = sender.try_send(JsCmd::Mount {
                        root_id: root_id.to_owned(),
                        path: path.to_owned(),
                    });
                }
                self.tx = Some(sender);
                self.pending.clear();
            }
            Event::Error { root_id, reason } => {
                log::error!("root: {:?}, Reason: {}", root_id, reason);

                // The tree that root last committed belongs to a context that is
                // gone, so its callback ids resolve to nothing. Drop it rather
                // than leave widgets on screen that look live and do nothing —
                // `view` falls back to a space when a root has no tree.
                if let Some(root_id) = root_id {
                    self.trees.remove(&root_id);
                }
            }
            Event::Committed { root_id, tree } => {
                self.trees.insert(root_id, tree);
            }
            Event::Callback(id, payload) => {
                if let Some(tx) = &mut self.tx {
                    if let Err(err) = tx.try_send(JsCmd::Dispatch(id, payload)) {
                        log::error!("{}", err);
                    }
                }
            }
        }
        Task::none()
    }
}
