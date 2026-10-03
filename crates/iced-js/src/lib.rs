mod host;
mod js_host;
mod loaders;
mod renderer;
mod runtime;

#[cfg(feature = "embed")]
pub use loaders::SiloAssets;
#[cfg(feature = "embed")]
pub use rust_silos; // reexport

pub use host::{Host, surface};
pub use loaders::FsAssets;
pub use runtime::{Event, js_worker};

type RootId = String;

#[cfg(test)]
mod tests {
    use crate::{Event, FsAssets, js_host, runtime::BUNDLED_RUNTIME_LIBS};
    use iced::futures::{StreamExt, channel::mpsc};
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module};

    const SCRIPT: &str = r#"
        import { createRoot } from "react-iced-native";
        import { jsx as _jsx } from "react/jsx-runtime";

        const View = () => _jsx("view", { children: _jsx("text", { children: "Hello, From JS" }) });

        createRoot("main").render(_jsx(View, {}));
    "#;

    /// Await the next event while driving the runtime from this same task, the
    /// way `js_worker` does. A drive() parked on its own task misses pending
    /// jobs enqueued by plain sync JS calls, and the schedular only wakes the
    /// last task that polled it — so the waiter has to be the driver.
    async fn next_driven(rt: &AsyncRuntime, rx: &mut mpsc::Receiver<Event>) -> Option<Event> {
        use iced::futures::future::{self, Either};

        let mut drive = std::pin::pin!(rt.drive());
        match future::select(rx.next(), drive.as_mut()).await {
            Either::Left((event, _)) => event,
            Either::Right(..) => unreachable!("runtime dropped while waiting"),
        }
    }

    /// Probe: does a `setImmediate` scheduled during module evaluation ever
    /// run when the only thing pumping the runtime afterwards is `drive()`?
    #[tokio::test(flavor = "multi_thread")]
    async fn set_immediate_runs_under_drive() {
        let (tx, mut rx) = mpsc::channel::<Event>(100);

        let rt = AsyncRuntime::new().unwrap();
        rt.set_loader(BUNDLED_RUNTIME_LIBS, BUNDLED_RUNTIME_LIBS)
            .await;

        let ctx = AsyncContext::full(&rt).await.unwrap();
        ctx.async_with(async |ctx| {
            js_host::timers::init(&ctx).unwrap();
            js_host::console::init(&ctx).unwrap();
            js_host::iced_host::init(&ctx, tx).unwrap();
        })
        .await;

        ctx.async_with(async |ctx| {
            let src = r#"setImmediate(() => { __ICED_INTERNALS__.commentTree("main", { text: "fired" }); });"#;
            let m = Module::declare(ctx.clone(), "probe", src)
                .catch(&ctx)
                .unwrap();
            let (_m, p) = m.eval().catch(&ctx).unwrap();
            p.into_future::<()>().await.catch(&ctx).unwrap();
        })
        .await;

        let Ok(Some(_)) =
            tokio::time::timeout(std::time::Duration::from_secs(2), next_driven(&rt, &mut rx))
                .await
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
        rt.set_loader(BUNDLED_RUNTIME_LIBS, BUNDLED_RUNTIME_LIBS)
            .await;

        let ctx = AsyncContext::full(&rt).await.unwrap();
        ctx.async_with(async |ctx| {
            js_host::timers::init(&ctx).unwrap();
            js_host::console::init(&ctx).unwrap();
            js_host::iced_host::init(&ctx, tx).unwrap();
        })
        .await;

        ctx.async_with(async |ctx| {
            let src = r#"queueMicrotask(() => { __ICED_INTERNALS__.commentTree("main", { text: "fired" }); });"#;
            let m = Module::declare(ctx.clone(), "probe", src)
                .catch(&ctx)
                .unwrap();
            let (_m, p) = m.eval().catch(&ctx).unwrap();
            p.into_future::<()>().await.catch(&ctx).unwrap();
        })
        .await;

        let Ok(Some(_)) =
            tokio::time::timeout(std::time::Duration::from_secs(2), next_driven(&rt, &mut rx))
                .await
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
        rt.set_loader(BUNDLED_RUNTIME_LIBS, BUNDLED_RUNTIME_LIBS)
            .await;

        let ctx = AsyncContext::full(&rt).await.unwrap();
        ctx.async_with(async |ctx| {
            js_host::timers::init(&ctx).unwrap();
            js_host::console::init(&ctx).unwrap();
            js_host::iced_host::init(&ctx, tx).unwrap();
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
            tokio::time::timeout(std::time::Duration::from_secs(2), next_driven(&rt, &mut rx))
                .await
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

    /// Drives the real worker through a reload: the script commits from a
    /// `setInterval`, so anything that survives the context swap keeps
    /// announcing itself.
    #[tokio::test(flavor = "multi_thread")]
    async fn reload_swaps_the_context_and_stops_old_timers() {
        use crate::{js_worker, renderer::Node, runtime::JsCmd};
        use std::time::Duration;
        use tokio::time::timeout;

        /// A script that commits `marker` every 10ms for as long as its context
        /// lives.
        fn ticker(marker: &str) -> String {
            format!(
                r#"setInterval(() => __ICED_INTERNALS__.commentTree("main", {{ text: "{marker}" }}), 10);"#
            )
        }

        /// Collect the markers committed over `window`, ignoring everything else.
        async fn markers(
            stream: &mut std::pin::Pin<Box<impl iced::futures::Stream<Item = Event>>>,
            window: Duration,
        ) -> Vec<String> {
            let deadline = tokio::time::Instant::now() + window;
            let mut seen = Vec::new();

            while let Ok(Some(event)) = timeout_at(deadline, stream.next()).await {
                match event {
                    Event::Committed { tree, .. } => match tree.as_ref() {
                        Node::Text(text) => seen.push(text.to_string()),
                        _ => panic!("expected a bare text commit"),
                    },
                    Event::Error { reason, .. } => panic!("js reported an error: {reason}"),
                    _ => {}
                }
            }

            seen
        }

        use tokio::time::timeout_at;

        let _ = env_logger::builder().is_test(true).try_init();

        let name = format!("iced-js-reload-{}.js", std::process::id());
        let path = std::env::temp_dir().join(&name);
        std::fs::write(&path, ticker("A")).unwrap();

        let mut stream = Box::pin(js_worker(&FsAssets::new(std::env::temp_dir())));

        let Ok(Some(Event::Ready(mut tx))) = timeout(Duration::from_secs(2), stream.next()).await
        else {
            panic!("worker never became ready");
        };

        tx.try_send(JsCmd::Mount {
            root_id: "main".to_string(),
            module: name,
        })
        .unwrap();

        let before = markers(&mut stream, Duration::from_millis(300)).await;
        assert!(
            before.iter().any(|m| m == "A"),
            "the original script never committed; got {before:?}"
        );

        std::fs::write(&path, ticker("B")).unwrap();
        tx.try_send(JsCmd::Reload).unwrap();

        // Commits queued before the swap are still in flight, so let them drain
        // before judging what is still alive.
        let _draining = markers(&mut stream, Duration::from_millis(300)).await;

        let after = markers(&mut stream, Duration::from_millis(300)).await;
        let _ = std::fs::remove_file(&path);

        assert!(
            !after.is_empty(),
            "nothing committed after the reload — the new context never mounted"
        );
        assert!(
            after.iter().all(|m| m == "B"),
            "the replaced context is still committing; got {after:?}"
        );
    }

    /// Covers the dispatch path end to end: the id Rust hands back has to reach
    /// the closure it was minted for, and `Payload` has to arrive as an object
    /// the handler can actually read a field off.
    #[tokio::test(flavor = "multi_thread")]
    async fn dispatch_reaches_the_callback_with_its_payload() {
        use crate::{
            js_worker,
            renderer::{Node, Tag},
            runtime::{JsCmd, Payload},
        };
        use std::time::Duration;
        use tokio::time::timeout;

        const CLICKABLE: &str = r#"
            import { createRoot } from "react-iced-native";
            import { jsx as _jsx } from "react/jsx-runtime";

            const View = () => _jsx("button", {
                onPress: (payload) => __ICED_INTERNALS__.commentTree("main", { text: "pressed:" + payload.type }),
                children: _jsx("text", { children: "press me" }),
            });

            createRoot("main").render(_jsx(View, {}));
        "#;

        fn on_press_of(node: &Node) -> Option<u64> {
            match node {
                Node::Text(_) => None,
                Node::Element { tag, children } => {
                    if let Tag::Button { on_press, .. } = tag {
                        if let Some(id) = on_press {
                            return Some(*id);
                        }
                    }
                    children.iter().find_map(on_press_of)
                }
            }
        }

        /// Pull commits until `pick` accepts one.
        async fn commit_until<T>(
            stream: &mut std::pin::Pin<Box<impl iced::futures::Stream<Item = Event>>>,
            missing: &str,
            pick: impl Fn(&Node) -> Option<T>,
        ) -> T {
            loop {
                match timeout(Duration::from_secs(2), stream.next()).await {
                    Ok(Some(Event::Committed { tree, .. })) => {
                        if let Some(found) = pick(tree.as_ref()) {
                            return found;
                        }
                    }
                    Ok(Some(Event::Error { reason, .. })) => {
                        panic!("js reported an error: {reason}")
                    }
                    Ok(Some(_)) => {}
                    _ => panic!("{missing}"),
                }
            }
        }

        let _ = env_logger::builder().is_test(true).try_init();

        let name = format!("iced-js-dispatch-{}.js", std::process::id());
        let path = std::env::temp_dir().join(&name);
        std::fs::write(&path, CLICKABLE).unwrap();

        let mut stream = Box::pin(js_worker(&FsAssets::new(std::env::temp_dir())));

        let Ok(Some(Event::Ready(mut tx))) = timeout(Duration::from_secs(2), stream.next()).await
        else {
            panic!("worker never became ready");
        };

        tx.try_send(JsCmd::Mount {
            root_id: "main".to_string(),
            module: name,
        })
        .unwrap();

        let callback_id = commit_until(&mut stream, "no button ever committed", on_press_of).await;

        tx.try_send(JsCmd::Dispatch(callback_id, Payload::Click))
            .unwrap();

        let pressed = commit_until(
            &mut stream,
            "the handler never ran — dispatch did not reach it",
            |node| match node {
                Node::Text(text) => Some(text.to_string()),
                _ => None,
            },
        )
        .await;

        let _ = std::fs::remove_file(&path);

        assert_eq!(pressed, "pressed:click");
    }

    /// Mirrors the example app's failure mode: an svg asset in the tree plus
    /// state-driven re-renders. Every re-render mints fresh callback ids and
    /// prunes the old ones, so dispatch only keeps working if every commit
    /// actually reaches Rust — a silently rejected commit leaves Rust holding
    /// ids the registry no longer knows.
    #[cfg(feature = "svg-element")]
    #[tokio::test(flavor = "multi_thread")]
    async fn svg_tree_keeps_dispatch_working_across_commits() {
        use crate::{
            js_worker,
            renderer::{Node, Tag},
            runtime::{JsCmd, Payload},
        };
        use std::time::Duration;
        use tokio::time::timeout;

        const COUNTER: &str = r#"
            import { createRoot } from "react-iced-native";
            import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
            import { useState } from "react";
            import icon from "./iced-js-test-icon.svg" with { type: "svg" };

            const View = () => {
                const [n, setN] = useState(0);
                return _jsxs("col", { children: [
                    _jsx("svg", { src: icon, width: 10, height: 10 }),
                    _jsx("button", {
                        onPress: () => setN((p) => p + 1),
                        children: _jsx("text", { children: "add" }),
                    }),
                    _jsx("text", { children: "count:" + n }),
                ]});
            };

            createRoot("main").render(_jsx(View, {}));
        "#;

        const ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4"/></svg>"#;

        fn on_press_of(node: &Node) -> Option<u64> {
            match node {
                Node::Text(_) => None,
                Node::Element { tag, children } => {
                    if let Tag::Button { on_press, .. } = tag {
                        if let Some(id) = on_press {
                            return Some(*id);
                        }
                    }
                    children.iter().find_map(on_press_of)
                }
            }
        }

        fn count_text_of(node: &Node) -> Option<String> {
            match node {
                Node::Text(text) => text.starts_with("count:").then(|| text.to_string()),
                Node::Element { children, .. } => children.iter().find_map(count_text_of),
            }
        }

        let _ = env_logger::builder().is_test(true).try_init();

        let script = format!("iced-js-svg-dispatch-{}.js", std::process::id());
        let dir = std::env::temp_dir();
        std::fs::write(dir.join(&script), COUNTER).unwrap();
        std::fs::write(dir.join("iced-js-test-icon.svg"), ICON).unwrap();

        let mut stream = Box::pin(js_worker(&FsAssets::new(dir.clone())));

        let Ok(Some(Event::Ready(mut tx))) = timeout(Duration::from_secs(2), stream.next()).await
        else {
            panic!("worker never became ready");
        };

        tx.try_send(JsCmd::Mount {
            root_id: "main".to_string(),
            module: script.clone(),
        })
        .unwrap();

        // Drive three rounds: each one dispatches the on_press id taken from
        // the LATEST committed tree and expects the count to advance. Round
        // n>1 uses ids minted by the previous round's re-render, which is
        // exactly where a desync shows up.
        let mut latest: Option<std::sync::Arc<Node>> = None;
        for round in 1..=3u32 {
            let want = format!("count:{}", round - 1);

            // Wait until we hold a tree showing the previous count.
            let tree = loop {
                if let Some(tree) = latest.take() {
                    if count_text_of(&tree).as_deref() == Some(want.as_str()) {
                        break tree;
                    }
                }
                match timeout(Duration::from_secs(2), stream.next()).await {
                    Ok(Some(Event::Committed { tree, .. })) => latest = Some(tree),
                    Ok(Some(Event::Error { reason, .. })) => {
                        panic!("js reported an error in round {round}: {reason}")
                    }
                    Ok(Some(_)) => {}
                    _ => panic!("round {round}: never saw a tree with {want:?}"),
                }
            };

            let id = on_press_of(&tree).expect("committed tree lost its button");
            tx.try_send(JsCmd::Dispatch(id, Payload::Click)).unwrap();

            let next = format!("count:{round}");
            loop {
                match timeout(Duration::from_secs(2), stream.next()).await {
                    Ok(Some(Event::Committed { tree, .. })) => {
                        let count = count_text_of(&tree);
                        latest = Some(tree);
                        if count.as_deref() == Some(next.as_str()) {
                            break;
                        }
                    }
                    Ok(Some(Event::Error { reason, .. })) => {
                        panic!("js reported an error in round {round}: {reason}")
                    }
                    Ok(Some(_)) => {}
                    _ => panic!(
                        "round {round}: dispatched id {id} but no commit showing {next:?} arrived — \
                         the id missed the callback registry or the commit was rejected"
                    ),
                }
            }
        }

        let _ = std::fs::remove_file(dir.join(&script));
        let _ = std::fs::remove_file(dir.join("iced-js-test-icon.svg"));
    }
}
