//! Timer globals (`setTimeout`, `setInterval`, `setImmediate`, `queueMicrotask`)
//! implemented directly on top of rquickjs 0.14 and tokio.
//!
//! Every timer is a future spawned onto the runtime's own scheduler via
//! [`Ctx::spawn`], so it is driven by whatever is driving the runtime
//! (`AsyncRuntime::drive` in [`crate::runtime`], or `async_with` in tests).
//! Because those futures are bound to `'js` they can hold the callback as a
//! plain [`Function`], which keeps it alive for as long as the timer is
//! pending without needing `Persistent` or a finalization registry.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    time::Duration,
};

use rquickjs::{
    CatchResultExt, Ctx, Function, JsLifetime, Result, Value,
    prelude::{Func, Opt, Rest},
};
use tokio::time::{Instant, sleep_until};

const TARGET: &str = "iced_js::timers";

/// Node clamps a delay of `0`/`NaN`/out-of-range to 1ms; matching that also
/// keeps a zero-delay interval from spinning the scheduler.
const MIN_DELAY_MS: f64 = 1.0;
/// Largest delay a 32 bit signed millisecond counter can hold, as used by Node
/// and the browsers. Anything above overflows there and is treated as 1ms.
const MAX_DELAY_MS: f64 = 2_147_483_647.0;

fn normalize_delay(delay: Option<f64>) -> Duration {
    let delay = match delay {
        Some(d) if d.is_finite() && (MIN_DELAY_MS..=MAX_DELAY_MS).contains(&d) => d,
        _ => MIN_DELAY_MS,
    };
    Duration::from_secs_f64(delay / 1000.0)
}

/// Shared cancellation flag. The handle in the registry and the one captured by
/// the spawned future point at the same cell, so `clearTimeout` is visible to a
/// timer that is already sleeping.
type Cancelled = Rc<Cell<bool>>;

/// Per-context timer registry, kept in the runtime's userdata store.
#[derive(Default, JsLifetime)]
pub struct Timers {
    next_id: Cell<u32>,
    live: RefCell<HashMap<u32, Cancelled>>,
}

impl Timers {
    fn register(&self) -> (u32, Cancelled) {
        // Ids start at 1 so a falsy `0` is never handed back to JS.
        let id = self.next_id.get().wrapping_add(1).max(1);
        self.next_id.set(id);

        let cancelled = Cancelled::default();
        self.live.borrow_mut().insert(id, cancelled.clone());
        (id, cancelled)
    }

    fn unregister(&self, id: u32) {
        self.live.borrow_mut().remove(&id);
    }

    fn cancel(&self, id: u32) {
        if let Some(cancelled) = self.live.borrow_mut().remove(&id) {
            cancelled.set(true);
        }
    }
}

/// Look up the registry for this context. Missing userdata means [`init`] was
/// never called, which is a host bug rather than something JS can trigger.
fn with_timers<'js, R>(ctx: &Ctx<'js>, f: impl FnOnce(&Timers) -> R) -> R {
    let timers = ctx
        .userdata::<Timers>()
        .expect("timers::init was not called for this context");
    f(&timers)
}

/// Run a timer callback, reporting an uncaught exception instead of losing it:
/// a spawned future has nowhere to return the error to.
fn call_callback<'js>(ctx: &Ctx<'js>, cb: &Function<'js>, args: &[Value<'js>]) {
    if let Err(err) = cb.call::<_, ()>((Rest(args.to_vec()),)).catch(ctx) {
        log::error!(target: TARGET, "uncaught exception in timer callback: {err}");
    }
}

fn set_timeout<'js>(
    ctx: Ctx<'js>,
    cb: Function<'js>,
    delay: Opt<f64>,
    args: Rest<Value<'js>>,
) -> u32 {
    let delay = normalize_delay(delay.0);
    let (id, cancelled) = with_timers(&ctx, |timers| timers.register());

    ctx.clone().spawn(async move {
        sleep_until(Instant::now() + delay).await;

        if cancelled.get() {
            return;
        }
        with_timers(&ctx, |timers| timers.unregister(id));
        call_callback(&ctx, &cb, &args.0);
    });

    id
}

fn set_interval<'js>(
    ctx: Ctx<'js>,
    cb: Function<'js>,
    delay: Opt<f64>,
    args: Rest<Value<'js>>,
) -> u32 {
    let delay = normalize_delay(delay.0);
    let (id, cancelled) = with_timers(&ctx, |timers| timers.register());

    ctx.clone().spawn(async move {
        // Tick off a fixed schedule rather than `now + delay` after each call,
        // so a slow callback doesn't make the interval drift.
        let mut next = Instant::now();
        loop {
            next += delay;
            sleep_until(next).await;

            if cancelled.get() {
                return;
            }
            call_callback(&ctx, &cb, &args.0);
            // The callback may have cleared itself.
            if cancelled.get() {
                return;
            }

            // If the callbacks ran long enough to fall behind, drop the missed
            // ticks instead of firing them back to back.
            let now = Instant::now();
            if next < now {
                next = now;
            }
        }
    });

    id
}

fn set_immediate<'js>(ctx: Ctx<'js>, cb: Function<'js>, args: Rest<Value<'js>>) -> u32 {
    let (id, cancelled) = with_timers(&ctx, |timers| timers.register());

    ctx.clone().spawn(async move {
        // One yield is enough to leave the current poll pass: the runtime
        // drains the pending job queue (promise reactions) before polling
        // spawned futures again, so this lands after microtasks but ahead of
        // any `setTimeout`, which has to wait on a real timer.
        tokio::task::yield_now().await;

        if cancelled.get() {
            return;
        }
        with_timers(&ctx, |timers| timers.unregister(id));
        call_callback(&ctx, &cb, &args.0);
    });

    id
}

/// Backs `clearTimeout`, `clearInterval` and `clearImmediate`. All three share
/// one id space, and a missing or non-numeric id is a no-op, as in the browser.
fn clear_timer(ctx: Ctx<'_>, id: Opt<Value<'_>>) {
    let Some(id) = id.0.as_ref().and_then(|v| v.as_number()) else {
        return;
    };
    if !(0.0..=f64::from(u32::MAX)).contains(&id) {
        return;
    }
    with_timers(&ctx, |timers| timers.cancel(id as u32));
}

fn queue_microtask<'js>(ctx: Ctx<'js>, cb: Function<'js>) -> Result<()> {
    // `defer` enqueues onto quickjs' own job queue, which is exactly the
    // microtask checkpoint promise reactions run on.
    cb.defer(())?;

    // ...but the job queue has no waker. `AsyncRuntime::drive` parks on the
    // *spawner*, and is woken only by `Spawner::push`, so a job queued from
    // Rust while the driver is parked sits there until something unrelated
    // wakes it. Pushing an empty future is that wake-up: the driver's loop
    // drains pending jobs before polling the scheduler, so the microtask runs
    // on the very next pass.
    //
    // Without this, anything that chains microtask -> microtask stalls
    // non-deterministically. React's root scheduling does exactly that when
    // `supportsMicrotasks` is set, which made the reconciler commit only when
    // it happened to win the race with an unrelated wake-up.
    ctx.spawn(async {});

    Ok(())
}

/// Install the timer globals and the per-context registry they use.
pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    if ctx.userdata::<Timers>().is_none() {
        // The error case is "userdata is currently borrowed", which cannot
        // happen here since we are not holding a guard.
        let _ = ctx.store_userdata(Timers::default());
    }

    let globals = ctx.globals();

    globals.set("setTimeout", Func::from(set_timeout))?;
    globals.set("setInterval", Func::from(set_interval))?;
    globals.set("setImmediate", Func::from(set_immediate))?;
    globals.set("clearTimeout", Func::from(clear_timer))?;
    globals.set("clearInterval", Func::from(clear_timer))?;
    globals.set("clearImmediate", Func::from(clear_timer))?;
    globals.set("queueMicrotask", Func::from(queue_microtask))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module, promise::Promise};

    use super::*;

    /// Evaluate `source` as a module whose default export is an async function,
    /// and return what it resolves to.
    async fn eval<T>(source: &str) -> T
    where
        T: for<'js> rquickjs::FromJs<'js> + Send + 'static,
    {
        let rt = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&rt).await.unwrap();

        ctx.async_with(async |ctx| {
            init(&ctx).unwrap();

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
    async fn set_timeout_resolves() {
        let result: String = eval(
            r#"
            export default () => new Promise((resolve) => {
                setTimeout(() => resolve("timeout"), 10);
            });
            "#,
        )
        .await;
        assert_eq!(result, "timeout");
    }

    #[tokio::test]
    async fn set_timeout_forwards_extra_args() {
        let result: Vec<i32> = eval(
            r#"
            export default () => new Promise((resolve) => {
                setTimeout((a, b) => resolve([a, b]), 1, 7, 9);
            });
            "#,
        )
        .await;
        assert_eq!(result, vec![7, 9]);
    }

    #[tokio::test]
    async fn clear_timeout_cancels() {
        let result: String = eval(
            r#"
            export default () => new Promise((resolve) => {
                const id = setTimeout(() => resolve("should not happen"), 5);
                clearTimeout(id);
                setTimeout(() => resolve("canceled"), 20);
            });
            "#,
        )
        .await;
        assert_eq!(result, "canceled");
    }

    #[tokio::test]
    async fn clear_timeout_ignores_bad_ids() {
        let result: String = eval(
            r#"
            export default () => new Promise((resolve) => {
                clearTimeout();
                clearTimeout(undefined);
                clearTimeout("nope");
                clearTimeout(-1);
                setTimeout(() => resolve("ok"), 1);
            });
            "#,
        )
        .await;
        assert_eq!(result, "ok");
    }

    #[tokio::test]
    async fn set_interval_repeats_until_cleared() {
        let result: i32 = eval(
            r#"
            export default () => new Promise((resolve) => {
                let count = 0;
                const id = setInterval(() => {
                    count++;
                    if (count === 3) {
                        clearInterval(id);
                        resolve(count);
                    }
                }, 5);
            });
            "#,
        )
        .await;
        assert_eq!(result, 3);
    }

    #[tokio::test]
    async fn multiple_intervals_run_independently() {
        let result: Vec<i32> = eval(
            r#"
            export default () => new Promise((resolve) => {
                let a = 0, b = 0;
                const idA = setInterval(() => { if (++a === 2) clearInterval(idA); }, 5);
                const idB = setInterval(() => {
                    if (++b === 3) {
                        clearInterval(idB);
                        resolve([a, b]);
                    }
                }, 10);
            });
            "#,
        )
        .await;
        assert_eq!(result, vec![2, 3]);
    }

    #[tokio::test]
    async fn set_immediate_runs_before_set_timeout() {
        let result: Vec<String> = eval(
            r#"
            export default () => new Promise((resolve) => {
                const order = [];
                setTimeout(() => {
                    order.push("timeout");
                    resolve(order);
                }, 1);
                setImmediate(() => order.push("immediate"));
            });
            "#,
        )
        .await;
        assert_eq!(result, vec!["immediate", "timeout"]);
    }

    #[tokio::test]
    async fn queue_microtask_runs_before_immediate() {
        let result: Vec<String> = eval(
            r#"
            export default () => new Promise((resolve) => {
                const order = [];
                setImmediate(() => {
                    order.push("immediate");
                    resolve(order);
                });
                queueMicrotask(() => order.push("microtask"));
            });
            "#,
        )
        .await;
        assert_eq!(result, vec!["microtask", "immediate"]);
    }

    #[tokio::test]
    async fn nested_timers_run() {
        let result: String = eval(
            r#"
            export default () => new Promise((resolve) => {
                setTimeout(() => {
                    setImmediate(() => {
                        setTimeout(() => resolve("nested"), 5);
                    });
                }, 5);
            });
            "#,
        )
        .await;
        assert_eq!(result, "nested");
    }

    #[tokio::test]
    async fn throwing_callback_does_not_stop_the_interval() {
        let result: i32 = eval(
            r#"
            export default () => new Promise((resolve) => {
                let count = 0;
                const id = setInterval(() => {
                    count++;
                    if (count === 1) throw new Error("boom");
                    if (count === 3) {
                        clearInterval(id);
                        resolve(count);
                    }
                }, 5);
            });
            "#,
        )
        .await;
        assert_eq!(result, 3);
    }

    #[tokio::test]
    async fn all_timer_globals_are_installed() {
        let result: Vec<String> = eval(
            r#"
            export default async () => [
                "setTimeout", "setInterval", "setImmediate",
                "clearTimeout", "clearInterval", "clearImmediate",
                "queueMicrotask",
            ].filter((name) => typeof globalThis[name] !== "function");
            "#,
        )
        .await;
        assert_eq!(result, Vec::<String>::new());
    }
}
