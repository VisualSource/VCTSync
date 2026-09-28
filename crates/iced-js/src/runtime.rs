use crate::{RootId, js_host, render::Node};
use iced::futures::{SinkExt, StreamExt, channel::mpsc};
use rquickjs::loader::{Loader, Resolver};
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, IntoJs, Module, embed, loader::Bundle};
use std::collections::HashMap;

pub(crate) static BUNDLED_LIBS: Bundle = embed! {
    "iced-dom": "js/dist/iced-dom.js",
    "react": "js/dist/react.js",
    "react/jsx-runtime": "js/dist/jsx-runtime.js"
};

#[derive(Debug, Clone)]
pub enum Payload {
    None,
    Click,
}

impl<'js> IntoJs<'js> for Payload {
    fn into_js(self, ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Payload::None => rquickjs::Object::new(ctx.clone()).map(|e| e.into_value()),
            Payload::Click => {
                let obj = rquickjs::Object::new(ctx.clone())?;

                let t = rquickjs::String::from_str(ctx.clone(), "click")?;
                obj.set("type", t)?;

                Ok(obj.into_value())
            }
        }
    }
}

#[derive(Clone)]
pub enum Event {
    Ipc(String),
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
        js_host::init_browser_apis(&ctx).expect("failed to init apis");

        js_host::iced_host::init(&ctx, tx).expect("failed to init host object");

        let globals = ctx.globals();
        globals
            .set(
                "__ICED_HOST__",
                rquickjs::Object::new_proto(ctx.clone(), None),
            )
            .expect("failed to register user host functions");
    })
    .await;

    eval_module(
        &ctx,
        "host::seed".to_string(),
        format!(
            r#"
                import {{ setCallbackBase }} from "iced-dom";
                setCallbackBase({});
            "#,
            callback_base
        )
        .into_bytes(),
    )
    .await?;

    Ok(ctx)
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
        rt.set_loader((BUNDLED_LIBS, loader.clone()), (BUNDLED_LIBS, loader))
            .await;
        let _handle = tokio::spawn(rt.drive()); // async loop

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
            let cmd = receiver.select_next_some().await;

            match cmd {
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
                    log::debug!("Mount ({},{:#?})", root_id, module);
                    if let Err(err) = mount_via_import(&ctx, &module).await {
                        log::error!("{:#?}", err);
                        output
                            .send(Event::Error {
                                root_id: Some(root_id),
                                reason: err,
                            })
                            .await
                            .expect("failed to send event from js worker");
                        return;
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
        }
    }))
}
