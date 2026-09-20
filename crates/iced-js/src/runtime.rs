use std::path::PathBuf;

use iced::futures::{SinkExt, Stream, StreamExt, channel::mpsc};
use rquickjs::{AsyncContext, AsyncRuntime, Module, embed, loader::Bundle};

use crate::{RootId, js_host, render::Node};

static BUNDLED_LIBS: Bundle = embed! {
    "iced-dom": "js/dist/iced-dom.js"
};

#[derive(Debug, Clone)]
pub enum Payload {
    None,
}

#[derive(Clone)]
pub enum Event {
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
    Mount { root_id: RootId, path: PathBuf },
    Unmount(RootId),
    Dispatch(u64, Payload),
}

pub fn js_worker() -> impl Stream<Item = Event> {
    iced::stream::channel(100, async |mut output| {
        let (sender, mut receiver) = mpsc::channel(100);

        let rt = AsyncRuntime::new().expect("js runtime failed to init");
        rt.set_loader(BUNDLED_LIBS, BUNDLED_LIBS).await;

        let _handle = tokio::spawn(rt.drive()); // async loop

        let ctx = AsyncContext::full(&rt)
            .await
            .expect("failed ot init the js context");

        let tx = output.clone();
        ctx.async_with(async |ctx| {
            js_host::timers::init(&ctx).expect("failed to init timer");
            js_host::console::init(&ctx).expect("failed to init console");
            js_host::react_reconciler::init(&ctx, tx).expect("failed to init host object");
        })
        .await;
        // should be able to use
        // async with here to init host env

        output
            .send(Event::Ready(sender))
            .await
            .expect("failed to send event from js worker");

        loop {
            let cmd = receiver.select_next_some().await;

            match cmd {
                JsCmd::Dispatch(id, payload) => {
                    let result: Result<(), String> = ctx.async_with(async |_ctx| Ok(())).await;

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
                    let result: Result<(), String> = ctx.async_with(async |_ctx| Ok(())).await;

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
                JsCmd::Mount { root_id, path } => {
                    let source_file = match tokio::fs::read(path).await {
                        Ok(source_file) => source_file,
                        Err(err) => {
                            log::error!("{:#?}", err);
                            output
                                .send(Event::Error {
                                    root_id: Some(root_id),
                                    reason: err.to_string(),
                                })
                                .await
                                .expect("failed to send event from js worker");
                            continue;
                        }
                    };

                    let result: Result<(), String> = ctx
                        .async_with(async |ctx| {
                            let user = Module::declare(
                                ctx.clone(),
                                format!("script::{}", root_id),
                                source_file,
                            )
                            .map_err(|err| err.to_string())?;

                            let (_user, p) = user.eval().map_err(|err| err.to_string())?;
                            p.into_future::<()>().await.map_err(|err| err.to_string())?;

                            // since createRoot is in module global scope it should
                            // register its self
                            // maybe check ctx for rootId before exiting?
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
            }
        }
    })
}
