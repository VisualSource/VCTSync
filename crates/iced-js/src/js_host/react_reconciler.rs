use iced::futures::channel::mpsc;
use rquickjs::{Ctx, Function, JsLifetime, Object, Result, String, class::Trace};
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

    fn invoke(&mut self, cmd: String<'_>, _payload: Object<'_>) -> Result<()> {
        let cmd = cmd.to_string()?;

        if let Err(err) = self.pipe.try_send(Event::Ipc(cmd)) {
            log::error!("{}", err);
        }

        Ok(())
    }
}

pub fn init(ctx: &Ctx, pipe: mpsc::Sender<Event>) -> Result<()> {
    let globals = ctx.globals();

    globals.set("__ICED_INTERNALS__", IcedHost::new(pipe))?;

    Ok(())
}
