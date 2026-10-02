use super::event_target::Listeners;
use crate::{Event, renderer::to_node};
use iced::futures::channel::mpsc;
use rquickjs::{
    CatchResultExt, Ctx, Function, JsLifetime, Object, Result, class::Trace, function::Opt,
};
use std::sync::Arc;

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct IcedHost<'js> {
    #[qjs(skip_trace)]
    pipe: mpsc::Sender<Event>,

    listeners: Listeners<'js>,
}

impl<'js> IcedHost<'js> {
    pub fn new(pipe: mpsc::Sender<Event>) -> Self {
        Self {
            pipe,
            listeners: Listeners::new(),
        }
    }

    pub fn dispatch(&mut self, ctx: &Ctx<'js>, cmd: String, data: String) {
        let callbacks = self.listeners.take(&cmd);

        let obj = match ctx.json_parse(data).catch(ctx) {
            Ok(v) => v,
            Err(err) => {
                log::error!("failed to dispatch ipc event: {}", err);
                return;
            }
        };

        for listener in callbacks {
            if let Err(err) = listener.call::<_, ()>((obj.clone(),)).catch(ctx) {
                log::error!("{}", err);
            }
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> IcedHost<'js> {
    fn comment_tree(&mut self, id: String, tree: Object<'_>) -> Result<()> {
        // A failure here would otherwise vanish: the exception lands in the
        // React microtask that flushed the commit, where nothing reports it,
        // and Rust silently keeps rendering the previous tree.
        let tree = match to_node(tree, 0) {
            Ok(tree) => tree,
            Err(err) => {
                log::error!("commit for root {id} rejected: {err}");
                let _ = self.pipe.try_send(Event::Error {
                    root_id: Some(id),
                    reason: err.to_string(),
                });
                return Err(err);
            }
        };

        if let Err(err) = self.pipe.try_send(Event::Committed {
            root_id: id,
            tree: Arc::new(tree),
        }) {
            log::error!("{}", err);
        }

        Ok(())
    }

    fn invoke(&mut self, ctx: Ctx<'js>, cmd: String, payload: Object<'js>) -> Result<()> {
        let data = ctx.json_stringify(payload)?;

        let payload = if let Some(s) = data {
            s.to_string()?
        } else {
            String::default()
        };

        if let Err(err) = self.pipe.try_send(Event::Ipc(cmd, payload)) {
            log::error!("{}", err);
        }

        Ok(())
    }

    fn add_event_listener(
        &mut self,
        l_type: String,
        callback: Function<'js>,
        opts: Opt<Object<'js>>,
    ) {
        let once = opts
            .0
            .as_ref()
            .map(|x| x.get::<_, bool>("once").unwrap_or(false))
            .unwrap_or(false);

        self.listeners.add(l_type, callback, once);
    }

    fn remove_event_listener(&mut self, l_type: String, callback: Function<'js>) {
        self.listeners.remove(l_type, callback);
    }
}

pub fn init(ctx: &Ctx, pipe: mpsc::Sender<Event>) -> Result<()> {
    let globals = ctx.globals();

    globals.set("__ICED_INTERNALS__", IcedHost::new(pipe))?;

    Ok(())
}
