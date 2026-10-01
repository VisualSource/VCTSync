use std::time::Duration;

use rquickjs::{
    CatchResultExt, Class, Ctx, Function, JsLifetime, Result, Value, class::Trace, function::Opt,
};

use crate::js_host::event_target::Listeners;

const TARGET: &str = "iced_js::abort";
/// The AbortSignal interface represents a signal object that allows you to communicate with an
///  asynchronous operation (such as a fetch request)
/// and abort it if required via an AbortController object.
#[rquickjs::class]
#[derive(Trace, JsLifetime)]
pub struct AbortSignal<'js> {
    #[qjs(get)]
    aborted: bool,

    /// default is undefined
    /// MDN does not give it a type
    /// but is most likly just a string
    #[qjs(get)]
    reason: Option<Value<'js>>,

    listeners: Listeners<'js>,

    /// Signals handed out by [`AbortSignal::any`] that abort when this one does.
    /// Held as a traced field rather than captured in a callback, so the GC can
    /// still collect the group once JS drops it.
    dependents: Vec<Class<'js, Self>>,
}

impl<'js> AbortSignal<'js> {
    pub fn new() -> Self {
        Self {
            aborted: false,
            reason: None,
            listeners: Listeners::new(),
            dependents: Vec::new(),
        }
    }

    fn fire(this: &Class<'js, Self>, ctx: &Ctx<'js>, reason: Value<'js>) -> Result<()> {
        let (callbacks, dependents) = {
            let mut inner = this.borrow_mut(); // prevent holding ref when dispatching handlers
            if inner.aborted {
                return Ok(());
            }
            inner.aborted = true;
            inner.reason = Some(reason.clone());
            (
                inner.listeners.take("abort"),
                std::mem::take(&mut inner.dependents),
            )
        };

        let event = rquickjs::Object::new(ctx.clone())?;
        event.set("type", "abort")?;
        event.set("target", this.clone())?;

        for callback in callbacks {
            if let Err(err) = callback.call::<_, ()>((event.clone(),)).catch(ctx) {
                log::error!(target: TARGET, "uncaught exception: {err}");
            }
        }

        for dependent in dependents {
            Self::fire(&dependent, ctx, reason.clone())?;
        }

        Ok(())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> AbortSignal<'js> {
    #[qjs(constructor)]
    pub fn constructor(ctx: Ctx<'js>) -> Result<()> {
        Err(rquickjs::Exception::throw_type(&ctx, "Illegal constructor"))
    }

    #[qjs(static)]
    fn abort(ctx: Ctx<'js>, reason: Opt<Value<'js>>) -> Result<Class<'js, Self>> {
        let mut inst = AbortSignal::new();
        inst.aborted = true;
        inst.reason = Some(reason.0.unwrap_or_else(|| abort_error(&ctx)));

        Class::instance(ctx, inst)
    }

    /// The AbortSignal.any() static method takes an iterable of abort signals and returns an AbortSignal.
    /// The returned abort signal is aborted when any of the input iterable abort signals are aborted.
    /// The abort reason will be set to the reason of the first signal that is aborted.
    /// If any of the given abort signals are already aborted then so will be the returned AbortSignal.
    #[qjs(static)]
    fn any(
        ctx: Ctx<'js>,
        iterator: rquickjs::JsIterator<'js, Class<'js, Self>>,
    ) -> Result<Class<'js, Self>> {
        let mut list = Vec::<Class<'js, Self>>::default();

        for item in iterator {
            let item = item?;

            let state = item.borrow();
            if state.aborted {
                let mut inst = Self::new();
                inst.aborted = true;
                inst.reason = state.reason.clone();
                drop(state);
                return Class::instance(ctx, inst);
            }
            drop(state);

            list.push(item);
        }

        let composite = Class::instance(ctx, AbortSignal::new())?;

        for source in list {
            source.borrow_mut().dependents.push(composite.clone());
        }

        Ok(composite)
    }

    /// The AbortSignal.timeout() static method returns an AbortSignal that will automatically abort after a specified time.
    ///
    /// The signal aborts with a TimeoutError DOMException on timeout.
    ///
    /// The timeout is based on active rather than elapsed time, and will effectively be paused if the code is running in a suspended worker,
    ///  or while the document is in a back-forward cache ("bfcache").
    ///
    /// To combine multiple signals, you can use AbortSignal.any(),
    /// for example, to directly abort a download using either a timeout signal or by calling AbortController.abort().
    #[qjs(static)]
    fn timeout(ctx: Ctx<'js>, time: u64) -> Result<Class<'js, AbortSignal<'js>>> {
        let signal = Class::instance(ctx.clone(), AbortSignal::new())?;

        let inner_signal = signal.clone();
        ctx.clone().spawn(async move {
            tokio::time::sleep(Duration::from_millis(time)).await;

            if let Err(err) = Self::fire(
                &inner_signal,
                &ctx,
                dom_exception(&ctx, "TimeoutError", "timeout"),
            )
            .catch(&ctx)
            {
                log::error!(target: TARGET, "AbortSignal.timeout dispatch failed: {err}");
            }
        });

        Ok(signal)
    }

    /// The throwIfAborted() method throws the signal's abort reason if the signal has been aborted;
    /// otherwise it does nothing.
    ///
    /// An API that needs to support aborting can accept an AbortSignal object and use throwIfAborted()
    /// to test and throw when the abort event is signaled.
    ///
    /// This method can also be used to abort operations at particular points in code,
    /// rather than passing to functions that take a signal.
    fn throw_if_aborted(&self, ctx: Ctx<'js>) -> Result<()> {
        if self.aborted {
            let error = self.reason.to_owned().unwrap_or_else(|| abort_error(&ctx));

            return Err(ctx.throw(error));
        }

        Ok(())
    }

    fn add_event_listener(
        &mut self,
        target: String,
        func: Function<'js>,
        opts: Opt<rquickjs::Object<'js>>,
    ) {
        let once = opts
            .0
            .as_ref()
            .map(|x| x.get::<_, bool>("once").unwrap_or(false))
            .unwrap_or(false);

        self.listeners.add(target, func, once);
    }
    fn remove_event_listener(&mut self, target: String, func: Function<'js>) {
        self.listeners.remove(target, func);
    }
}

///
#[rquickjs::class]
#[derive(Trace, JsLifetime)]
pub struct AbortController<'js> {
    /// The signal read-only property of the AbortController interface returns an AbortSignal object instance,
    /// which can be used to communicate with/abort an asynchronous operation as desired.
    #[qjs(get)]
    signal: Class<'js, AbortSignal<'js>>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> AbortController<'js> {
    #[qjs(constructor)]
    pub fn constructor(ctx: Ctx<'js>) -> Result<Self> {
        Ok(Self {
            signal: Class::instance(ctx, AbortSignal::new())?,
        })
    }

    /// The abort() method of the AbortController interface aborts an asynchronous operation before it has completed.
    /// This is able to abort fetch requests, the consumption of any response bodies, or streams.
    fn abort(&self, ctx: Ctx<'js>, reason: Opt<Value<'js>>) -> Result<()> {
        AbortSignal::fire(
            &self.signal,
            &ctx,
            reason.0.unwrap_or_else(|| abort_error(&ctx)),
        )
    }
}

fn dom_exception<'js>(ctx: &Ctx<'js>, name: &str, reason: &str) -> Value<'js> {
    let _ = rquickjs::Exception::throw_dom(ctx, name, reason);
    ctx.catch()
}

fn abort_error<'js>(ctx: &Ctx<'js>) -> Value<'js> {
    dom_exception(ctx, "AbortError", "signal is aborted without reason")
}

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();

    Class::<AbortSignal>::define(&globals)?;
    Class::<AbortController>::define(&globals)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rquickjs::{AsyncContext, AsyncRuntime, Module, promise::Promise};

    use super::*;

    async fn eval<T>(source: &str) -> T
    where
        T: for<'js> rquickjs::FromJs<'js> + Send + 'static,
    {
        let rt = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&rt).await.unwrap();

        ctx.async_with(async |ctx| {
            init(&ctx).unwrap();
            crate::js_host::timers::init(&ctx).unwrap();

            let (module, promise) = Module::declare(ctx.clone(), "test", source)
                .catch(&ctx)
                .unwrap()
                .eval()
                .catch(&ctx)
                .unwrap();
            promise.into_future::<()>().await.catch(&ctx).unwrap();

            let test: Function = module.get("default").catch(&ctx).unwrap();
            test.call::<_, Promise>(())
                .catch(&ctx)
                .unwrap()
                .into_future::<T>()
                .await
                .catch(&ctx)
                .unwrap()
        })
        .await
    }

    #[tokio::test]
    async fn controller_abort_fires_listener_with_reason() {
        let result: Vec<String> = eval(
            r#"
            export default () => new Promise((resolve) => {
                const c = new AbortController();
                c.signal.addEventListener("abort", (e) => {
                    resolve([String(c.signal.aborted), c.signal.reason, e.type, String(e.target === c.signal)]);
                });
                c.abort("nope");
            });
            "#,
        )
        .await;
        assert_eq!(result, ["true", "nope", "abort", "true"]);
    }

    #[tokio::test]
    async fn abort_is_idempotent() {
        let result: i32 = eval(
            r#"
            export default async () => {
                const c = new AbortController();
                let n = 0;
                c.signal.addEventListener("abort", () => n++);
                c.abort();
                c.abort();
                return n;
            };
            "#,
        )
        .await;
        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn constructor_is_illegal() {
        let result: String = eval(
            r#"
            export default async () => {
                try { new AbortSignal(); return "no throw"; }
                catch (e) { return e.constructor.name; }
            };
            "#,
        )
        .await;
        assert_eq!(result, "TypeError");
    }

    #[tokio::test]
    async fn static_abort_defaults_reason_to_abort_error() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const s = AbortSignal.abort();
                return [String(s.aborted), String(s.reason && s.reason.name)];
            };
            "#,
        )
        .await;
        assert_eq!(result, ["true", "AbortError"]);
    }

    #[tokio::test]
    async fn throw_if_aborted_throws_reason() {
        let result: String = eval(
            r#"
            export default async () => {
                const s = AbortSignal.abort("boom");
                try { s.throwIfAborted(); return "no throw"; } catch (e) { return e; }
            };
            "#,
        )
        .await;
        assert_eq!(result, "boom");
    }

    #[tokio::test]
    async fn timeout_aborts_with_timeout_error() {
        let result: Vec<String> = eval(
            r#"
            export default () => new Promise((resolve) => {
                const s = AbortSignal.timeout(10);
                s.addEventListener("abort", () => resolve([String(s.aborted), s.reason.name]));
            });
            "#,
        )
        .await;
        assert_eq!(result, ["true", "TimeoutError"]);
    }

    #[tokio::test]
    async fn any_forwards_source_reason() {
        let result: String = eval(
            r#"
            export default () => new Promise((resolve) => {
                const a = new AbortController();
                const b = new AbortController();
                const s = AbortSignal.any([a.signal, b.signal]);
                s.addEventListener("abort", () => resolve(s.reason));
                b.abort("from source");
            });
            "#,
        )
        .await;
        assert_eq!(result, "from source");
    }

    /// The canonical usage: a timeout racing a controller, where the timeout never fires.
    #[tokio::test]
    async fn any_with_pending_timeout_source() {
        let result: String = eval(
            r#"
            export default () => new Promise((resolve) => {
                const c = new AbortController();
                const s = AbortSignal.any([AbortSignal.timeout(10_000), c.signal]);
                s.addEventListener("abort", () => resolve(s.reason));
                c.abort("beat the clock");
            });
            "#,
        )
        .await;
        assert_eq!(result, "beat the clock");
    }

    #[tokio::test]
    async fn any_already_aborted_is_aborted_at_construction() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const s = AbortSignal.any([AbortSignal.abort("already")]);
                return [String(s.aborted), s.reason];
            };
            "#,
        )
        .await;
        assert_eq!(result, ["true", "already"]);
    }

    #[tokio::test]
    async fn remove_event_listener_unregisters() {
        let result: i32 = eval(
            r#"
            export default async () => {
                const c = new AbortController();
                let n = 0;
                const fn = () => n++;
                c.signal.addEventListener("abort", fn);
                c.signal.removeEventListener("abort", fn);
                c.abort();
                return n;
            };
            "#,
        )
        .await;
        assert_eq!(result, 0);
    }
}
