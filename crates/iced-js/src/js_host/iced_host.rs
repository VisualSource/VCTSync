use iced::futures::channel::mpsc;
use rquickjs::{
    CatchResultExt, Ctx, Function, JsLifetime, Object, Result, class::Trace, function::Opt,
};
use std::sync::Arc;

use crate::{Event, render::to_node};

#[derive(Clone, Trace, JsLifetime)]
struct ListenerTarget<'js> {
    callback: Function<'js>,
    once: bool,
    target: String,
}

impl<'js> ListenerTarget<'js> {
    fn new(target: String, callback: Function<'js>, once: bool) -> Self {
        Self {
            target,
            callback,
            once,
        }
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct IcedHost<'js> {
    #[qjs(skip_trace)]
    pipe: mpsc::Sender<Event>,

    listeners: Vec<ListenerTarget<'js>>,
}

impl<'js> IcedHost<'js> {
    pub fn new(pipe: mpsc::Sender<Event>) -> Self {
        Self {
            pipe,
            listeners: Vec::default(),
        }
    }

    pub fn dispatch(&mut self, ctx: &Ctx<'js>, cmd: std::string::String) {
        let mut remove = Vec::default();
        for (idx, listen) in self.listeners.iter().enumerate() {
            if listen.target != cmd {
                continue;
            }

            if listen.once {
                remove.push(idx);
            }

            if let Err(err) = listen.callback.call::<(), ()>(()).catch(&ctx) {
                log::error!("{}", err);
            }
        }

        for idx in remove {
            self.listeners.swap_remove(idx);
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> IcedHost<'js> {
    fn comment_tree(&mut self, id: String, tree: Object<'_>) -> Result<()> {
        let tree = to_node(tree, 0)?;

        if let Err(err) = self.pipe.try_send(Event::Committed {
            root_id: id,
            tree: Arc::new(tree),
        }) {
            log::error!("{}", err);
        }

        Ok(())
    }

    fn invoke(&mut self, cmd: String, _payload: Object<'_>) -> Result<()> {
        if let Err(err) = self.pipe.try_send(Event::Ipc(cmd)) {
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

        self.listeners
            .push(ListenerTarget::new(l_type, callback, once));
    }

    fn remove_event_listener(&mut self, l_type: String, callback: Function<'js>) {
        let Some(idx) = self
            .listeners
            .iter()
            .position(|x| x.target == l_type && x.callback.eq(&callback))
        else {
            return;
        };

        self.listeners.swap_remove(idx);
    }
}

pub fn init(ctx: &Ctx, pipe: mpsc::Sender<Event>) -> Result<()> {
    let globals = ctx.globals();

    globals.set("__ICED_INTERNALS__", IcedHost::new(pipe))?;

    Ok(())
}
