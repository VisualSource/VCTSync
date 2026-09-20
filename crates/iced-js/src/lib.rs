mod host;
mod js_host;
mod render;
mod runtime;
mod view;

pub use host::Host;
pub use runtime::{Event, js_worker};
pub use view::view;

type RootId = String;

#[cfg(test)]
mod tests {
    /*  use rquickjs::{AsyncContext, AsyncRuntime, context::EvalOptions, loader::Bundle};

    #[tokio::test]
    async fn test() {
        let rt = AsyncRuntime::new().expect("failed to setup runtime");

        let ctx = AsyncContext::full(&rt).await.expect("failed to create ctx");
        ctx.async_with(async |ctx| {
            let mut opts = EvalOptions::default();
            opts.global = false;
            opts.promise = true;
            opts.strict = true;

            ctx.eval_file_with_options::<(), _>(
                concat!(env!("CARGO_MANIFEST_DIR"), "/example/view.js"),
                opts,
            )
            .expect("failed to execute file");
        })
        .await;
    }*/
}
