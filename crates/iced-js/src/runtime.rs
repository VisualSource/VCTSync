use iced::futures::{
    SinkExt, StreamExt,
    channel::mpsc,
    future::{self, Either},
};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, IntoJs, Module, embed,
    loader::Bundle,
    loader::{Loader, Resolver},
};
use std::collections::HashMap;

use crate::{
    RootId,
    js_host::{self, iced_host::IcedHost},
    loaders::SystemModule,
    renderer::Node,
};

pub(crate) static BUNDLED_RUNTIME_LIBS: Bundle = embed! {
    "react-iced-native": "js/dist/react-iced-native.js",
    "react": "js/dist/react.js",
    "react/jsx-runtime": "js/dist/jsx-runtime.js"
};

#[derive(Debug, Clone)]
pub enum Payload {
    None,
    Click,

    TextInputChange(String),
    BoolInputChange(bool),
}

impl<'js> IntoJs<'js> for Payload {
    fn into_js(self, ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Payload::None => rquickjs::Object::new(ctx.clone()).map(|e| e.into_value()),
            Payload::Click => {
                let obj = rquickjs::Object::new(ctx.clone())?;
                obj.set("type", "click")?;

                Ok(obj.into_value())
            }
            Payload::BoolInputChange(value) => {
                let obj = rquickjs::Object::new(ctx.clone())?;
                obj.set("type", "change")?;
                obj.set("value", value)?;
                Ok(obj.into_value())
            }
            Payload::TextInputChange(value) => {
                let obj = rquickjs::Object::new(ctx.clone())?;
                obj.set("type", "change")?;
                obj.set("value", value)?;
                Ok(obj.into_value())
            }
        }
    }
}

#[derive(Clone)]
pub enum Event {
    IpcDispatch(String, String),
    Ipc(String, String),
    Ready(mpsc::Sender<JsCmd>),
    Error {
        root_id: Option<RootId>,
        reason: String,
    },
    Committed {
        root_id: RootId,
        tree: std::sync::Arc<Node>,
    },
    Callback(u64, Payload),
}

pub enum JsCmd {
    Mount { root_id: RootId, module: String },
    Unmount(RootId),
    Dispatch(u64, Payload),
    IpcDispatch(String, String),
    Reload,
}

/// Ids one context may mint before the next reload's base could collide with
/// them.
// ponytail: fixed stride. Read the outgoing context's high-water mark instead
// if a single generation ever mints more than this.
const CALLBACK_STRIDE: u64 = 1_000_000;

/// Declare and evaluate `source` as an ES module, awaiting its evaluation
/// promise. An exception comes back as the formatted error string.
async fn eval_module(ctx: &AsyncContext, name: String, source: Vec<u8>) -> Result<(), String> {
    ctx.async_with(async move |ctx| {
        let module = Module::declare(ctx.clone(), name, source)
            .catch(&ctx)
            .map_err(|err| err.to_string())?;

        let (_module, promise) = module.eval().catch(&ctx).map_err(|err| err.to_string())?;
        promise
            .into_future::<()>()
            .await
            .catch(&ctx)
            .map_err(|err| err.to_string())?;

        Ok(())
    })
    .await
}

/// Build a context with the host globals installed, seeding the callback id
/// counter past everything an earlier context minted.
async fn new_context(
    rt: &AsyncRuntime,
    output: &mpsc::Sender<Event>,
    callback_base: u64,
) -> Result<AsyncContext, String> {
    let ctx = AsyncContext::full(rt)
        .await
        .map_err(|err| err.to_string())?;

    let tx = output.clone();
    ctx.async_with(async |ctx| {
        #[cfg(feature = "svg-element")]
        crate::loaders::svg::init(&ctx).expect("failed to init svg api");

        js_host::init_browser_apis(&ctx).expect("failed to init apis");

        js_host::iced_host::init(&ctx, tx).expect("failed to init host object");
    })
    .await;

    eval_module(
        &ctx,
        "host::seed".to_string(),
        format!(
            r#"
                import {{ setCallbackBase }} from "react-iced-native";
                setCallbackBase({});
            "#,
            callback_base
        )
        .into_bytes(),
    )
    .await?;

    Ok(ctx)
}

/// Run every pending QuickJS job to completion.
///
/// `rt.drive()` only wakes when a Rust future is spawned into the runtime; a
/// job enqueued by a plain synchronous call into JS (a dispatched callback
/// doing `setState`, which React flushes through `queueMicrotask`) never wakes
/// it and would sit in the queue forever. So every command that calls into JS
/// has to drain the queue itself afterwards.
async fn drain_jobs(rt: &AsyncRuntime) {
    loop {
        match rt.execute_pending_job().await {
            Ok(true) => continue,
            Ok(false) => break,
            // The job that threw is consumed, so this makes progress.
            Err(err) => {
                let reason = err
                    .0
                    .async_with(async |ctx| format!("{:?}", ctx.catch()))
                    .await;
                log::error!("uncaught error in js job: {reason}");
            }
        }
    }
}

async fn mount_via_import(ctx: &AsyncContext, esm_import: &str) -> Result<(), String> {
    ctx.async_with(async |ctx| {
        let module = Module::import(&ctx, esm_import)
            .catch(&ctx)
            .map_err(|err| err.to_string())?;

        module
            .into_future::<()>()
            .await
            .catch(&ctx)
            .map_err(|err| err.to_string())?;

        Ok(())
    })
    .await
}

pub fn js_worker<A>(loader: &A) -> iced::futures::stream::BoxStream<'static, Event>
where
    A: Resolver + Loader + Clone + Send + 'static,
{
    let loader = loader.clone();
    Box::pin(iced::stream::channel(100, async |mut output| {
        let (sender, mut receiver) = mpsc::channel(100);

        let rt = AsyncRuntime::new().expect("js runtime failed to init");
        rt.set_loader(
            (
                BUNDLED_RUNTIME_LIBS,
                SystemModule::default(),
                loader.clone(),
            ),
            (BUNDLED_RUNTIME_LIBS, SystemModule::default(), loader),
        )
        .await;

        // Polled in the select below rather than spawned on its own task: the
        // runtime's schedular holds a single waker — whichever task polled it
        // last. Processing a command polls it from this task (async_with,
        // drain_jobs), which would steal timer wakeups from a separate drive
        // task and leave e.g. a setInterval stalled until the next command.
        // Driving from the same task that waits for commands closes that hole.
        let mut drive = std::pin::pin!(rt.drive());

        let mut ctx = new_context(&rt, &output, 0)
            .await
            .expect("failed ot init the js context");

        // Every mounted root, kept so a reload can re-read the scripts. A root
        // whose script failed stays here, so fixing the file and reloading
        // again recovers it.
        let mut mounted: HashMap<RootId, String> = HashMap::new();
        let mut generation: u64 = 0;

        output
            .send(Event::Ready(sender))
            .await
            .expect("failed to send event from js worker");

        loop {
            let cmd = match future::select(receiver.select_next_some(), drive.as_mut()).await {
                Either::Left((cmd, _)) => cmd,
                // drive() resolves only when the runtime is dropped, and `rt`
                // outlives this loop.
                Either::Right(..) => unreachable!("runtime dropped while the worker is running"),
            };

            match cmd {
                JsCmd::IpcDispatch(cmd, data) => {
                    let result: Result<(), String> = ctx
                        .async_with(async |ctx| {
                            let globals = ctx.globals();

                            let host_obj = globals
                                .get::<_, rquickjs::Class<'_, IcedHost>>("__ICED_INTERNALS__")
                                .catch(&ctx)
                                .map_err(|err| err.to_string())?;

                            let mut host = host_obj.borrow_mut();

                            host.dispatch(&ctx, cmd, data);

                            Ok(())
                        })
                        .await;

                    if let Err(err) = result {
                        log::error!("{:#?}", err);
                        output
                            .send(Event::Error {
                                root_id: None,
                                reason: err.to_string(),
                            })
                            .await
                            .expect("failed to send event from js worker");
                    }
                }
                JsCmd::Dispatch(id, payload) => {
                    let result: Result<(), String> = ctx
                        .async_with(async |ctx| {
                            let global = ctx.globals();

                            let iced = global
                                .get::<_, rquickjs::Object<'_>>("__ICED_REACT_RUNTIME__")
                                .catch(&ctx)
                                .map_err(|err| err.to_string())?;

                            let dispatch = iced
                                .get::<_, rquickjs::Function<'_>>("dispatch")
                                .catch(&ctx)
                                .map_err(|err| err.to_string())?;

                            dispatch
                                .call::<_, ()>((id, payload))
                                .catch(&ctx)
                                .map_err(|err| err.to_string())?;

                            Ok(())
                        })
                        .await;

                    if let Err(err) = result {
                        log::error!("{:#?}", err);
                        output
                            .send(Event::Error {
                                root_id: None,
                                reason: err.to_string(),
                            })
                            .await
                            .expect("failed to send event from js worker");
                    }
                }
                JsCmd::Unmount(root_id) => {
                    mounted.remove(&root_id);

                    let result: Result<(), String> = ctx
                        .async_with(async |ctx| {
                            let global = ctx.globals();

                            let iced = global
                                .get::<_, rquickjs::Object<'_>>("__ICED_REACT_RUNTIME__")
                                .catch(&ctx)
                                .map_err(|err| err.to_string())?;

                            let dispatch = iced
                                .get::<_, rquickjs::Function<'_>>("destroyRoot")
                                .catch(&ctx)
                                .map_err(|err| err.to_string())?;

                            dispatch
                                .call::<_, ()>((root_id.clone(),))
                                .catch(&ctx)
                                .map_err(|err| err.to_string())?;

                            Ok(())
                        })
                        .await;

                    if let Err(err) = result {
                        log::error!("{:#?}", err);
                        output
                            .send(Event::Error {
                                root_id: Some(root_id),
                                reason: err.to_string(),
                            })
                            .await
                            .expect("failed to send event from js worker");
                    }
                }
                JsCmd::Mount { root_id, module } => {
                    if let Err(err) = mount_via_import(&ctx, &module).await {
                        log::error!("{:#?}", err);
                        output
                            .send(Event::Error {
                                root_id: Some(root_id),
                                reason: err,
                            })
                            .await
                            .expect("failed to send event from js worker");
                        continue;
                    }

                    mounted.insert(root_id, module);
                }
                JsCmd::Reload => {
                    generation += 1;

                    // Build the replacement before touching the live one, so a
                    // failure here leaves the running context intact.
                    let next = match new_context(&rt, &output, generation * CALLBACK_STRIDE).await {
                        Ok(next) => next,
                        Err(err) => {
                            log::error!("{:#?}", err);
                            output
                                .send(Event::Error {
                                    root_id: None,
                                    reason: err,
                                })
                                .await
                                .expect("failed to send event from js worker");
                            continue;
                        }
                    };

                    // Timers are per-context userdata, so this has to run on the
                    // outgoing context — and it has to run at all: a spawned
                    // interval holds its own reference, so one left ticking keeps
                    // the replaced context alive and committing over the new one.
                    ctx.async_with(async |ctx| js_host::timers::cancel_all(&ctx))
                        .await;
                    ctx = next;

                    for (root_id, source) in &mounted {
                        if let Err(err) = mount_via_import(&ctx, source).await {
                            log::error!("{:#?}", err);
                            output
                                .send(Event::Error {
                                    root_id: Some(root_id.to_owned()),
                                    reason: err,
                                })
                                .await
                                .expect("failed to send event from js worker");
                        }
                    }
                }
            }

            drain_jobs(&rt).await;
        }
    }))
}
