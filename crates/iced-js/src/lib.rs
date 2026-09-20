mod host;
mod js_host;
mod nodes;
mod render;
mod runtime;
mod view;

pub use host::Host;
pub use runtime::{Event, js_worker};
pub use view::view;

type RootId = String;

#[cfg(test)]
mod tests {
    use crate::{Event, js_host, runtime::BUNDLED_LIBS};
    use iced::futures::{StreamExt, channel::mpsc};
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module};

    const SCRIPT: &str = r#"
        import { createRoot } from "iced-dom";
        import { jsx as _jsx } from "react/jsx-runtime";

        const View = () => _jsx("view", { children: _jsx("text", { children: "Hello, From JS" }) });

        createRoot("main").render(_jsx(View, {}));
    "#;

    /// Probe: does a `setImmediate` scheduled during module evaluation ever
    /// run when the only thing pumping the runtime afterwards is `drive()`?
    #[tokio::test(flavor = "multi_thread")]
    async fn set_immediate_runs_under_drive() {
        let (tx, mut rx) = mpsc::channel::<Event>(100);

        let rt = AsyncRuntime::new().unwrap();
        rt.set_loader(BUNDLED_LIBS, BUNDLED_LIBS).await;
        let _drive = tokio::spawn(rt.drive());

        let ctx = AsyncContext::full(&rt).await.unwrap();
        ctx.async_with(async |ctx| {
            js_host::timers::init(&ctx).unwrap();
            js_host::console::init(&ctx).unwrap();
            js_host::react_reconciler::init(&ctx, tx).unwrap();
        })
        .await;

        ctx.async_with(async |ctx| {
            let src = r#"setImmediate(() => { iced.comment_tree("main", { text: "fired" }); });"#;
            let m = Module::declare(ctx.clone(), "probe", src)
                .catch(&ctx)
                .unwrap();
            let (_m, p) = m.eval().catch(&ctx).unwrap();
            p.into_future::<()>().await.catch(&ctx).unwrap();
        })
        .await;

        let Ok(Some(_)) = tokio::time::timeout(std::time::Duration::from_secs(2), rx.next()).await
        else {
            panic!("setImmediate callback never ran under drive()");
        };
    }

    /// Probe: same as above but via `queueMicrotask`, which React uses for
    /// root scheduling whenever `supportsMicrotasks` is true.
    #[tokio::test(flavor = "multi_thread")]
    async fn queue_microtask_runs_under_drive() {
        let (tx, mut rx) = mpsc::channel::<Event>(100);

        let rt = AsyncRuntime::new().unwrap();
        rt.set_loader(BUNDLED_LIBS, BUNDLED_LIBS).await;
        let _drive = tokio::spawn(rt.drive());

        let ctx = AsyncContext::full(&rt).await.unwrap();
        ctx.async_with(async |ctx| {
            js_host::timers::init(&ctx).unwrap();
            js_host::console::init(&ctx).unwrap();
            js_host::react_reconciler::init(&ctx, tx).unwrap();
        })
        .await;

        ctx.async_with(async |ctx| {
            let src = r#"queueMicrotask(() => { iced.comment_tree("main", { text: "fired" }); });"#;
            let m = Module::declare(ctx.clone(), "probe", src)
                .catch(&ctx)
                .unwrap();
            let (_m, p) = m.eval().catch(&ctx).unwrap();
            p.into_future::<()>().await.catch(&ctx).unwrap();
        })
        .await;

        let Ok(Some(_)) = tokio::time::timeout(std::time::Duration::from_secs(2), rx.next()).await
        else {
            panic!("queueMicrotask callback never ran under drive()");
        };
    }

    /// Drives the same path `js_worker` does — bundle loader, host globals,
    /// `drive()`, declare + eval — and asserts React actually reaches a commit.
    #[tokio::test(flavor = "multi_thread")]
    async fn script_commits_a_tree() {
        let _ = env_logger::builder().is_test(true).try_init();

        let (tx, mut rx) = mpsc::channel::<Event>(100);

        let rt = AsyncRuntime::new().unwrap();
        rt.set_loader(BUNDLED_LIBS, BUNDLED_LIBS).await;
        let _drive = tokio::spawn(rt.drive());

        let ctx = AsyncContext::full(&rt).await.unwrap();
        ctx.async_with(async |ctx| {
            js_host::timers::init(&ctx).unwrap();
            js_host::console::init(&ctx).unwrap();
            js_host::react_reconciler::init(&ctx, tx).unwrap();
        })
        .await;

        ctx.async_with(async |ctx| {
            let m = Module::declare(ctx.clone(), "script::main", SCRIPT)
                .catch(&ctx)
                .unwrap();
            let (_m, p) = m.eval().catch(&ctx).unwrap();
            p.into_future::<()>().await.catch(&ctx).unwrap();
        })
        .await;

        let Ok(committed) =
            tokio::time::timeout(std::time::Duration::from_secs(2), rx.next()).await
        else {
            panic!("timed out waiting for a commit — React scheduled work that never ran");
        };

        match committed {
            Some(Event::Committed { root_id, .. }) => assert_eq!(root_id, "main"),
            Some(Event::Error { reason, .. }) => panic!("js reported an error: {reason}"),
            Some(_) => panic!("expected a Committed event, got a different one"),
            None => panic!("event channel closed before a commit arrived"),
        }
    }
}
