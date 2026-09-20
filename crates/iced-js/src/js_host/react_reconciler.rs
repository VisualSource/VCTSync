use iced::futures::channel::mpsc;
use rquickjs::{Ctx, JsLifetime, Object, Result, String, class::Trace};
use std::sync::Arc;

use crate::{Event, render::to_node};

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct IcedHost {
    #[qjs(skip_trace)]
    pipe: mpsc::Sender<Event>,
}

impl IcedHost {
    pub fn new(pipe: mpsc::Sender<Event>) -> Self {
        Self { pipe }
    }
}

#[rquickjs::methods]
impl IcedHost {
    fn comment_tree(&mut self, id: String<'_>, tree: Object<'_>) -> Result<()> {
        let root_id = id.to_string()?;

        let tree = to_node(tree, 0)?;

        if let Err(err) = self.pipe.try_send(Event::Committed {
            root_id,
            tree: Arc::new(tree),
        }) {
            log::error!("{}", err);
        }

        Ok(())
    }
}

pub fn init(ctx: &Ctx, pipe: mpsc::Sender<Event>) -> Result<()> {
    let globals = ctx.globals();

    globals.set("iced", IcedHost::new(pipe))?;

    Ok(())
}
