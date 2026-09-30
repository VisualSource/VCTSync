use rquickjs::{Class, Ctx, Function, JsLifetime, Result, Value, class::Trace, function::Opt};

use crate::js_host::event_target::Listeners;

#[rquickjs::class]
#[derive(Trace, JsLifetime)]
struct AbortSignal<'js> {
    #[qjs(get)]
    aborted: bool,

    /// default is undefined
    /// MDN does not give it a type
    /// but is most likly just a string
    #[qjs(get)]
    reason: Option<Value<'js>>,

    listeners: Listeners<'js>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> AbortSignal<'js> {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {
            aborted: false,
            reason: None,
            listeners: Listeners::new(),
        }
    }

    fn abort(reason: Opt<String>) -> Self {
        unimplemented!()
    }
    fn any(iterator: rquickjs::JsIterator<'js, Class<'js, AbortSignal<'js>>>) -> Self {
        unimplemented!()
    }

    fn timeout(ctx: Ctx<'js>, time: u64) -> Self {
        unimplemented!()
    }

    fn throw_if_aborted(&self, ctx: Ctx<'js>) -> Result<()> {
        if self.aborted {
            return Err(rquickjs::Exception::throw_dom(&ctx, "AbortError", ""));
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

#[rquickjs::class]
#[derive(Trace, JsLifetime)]
pub struct AbortController<'js> {
    #[qjs(get)]
    signal: Class<'js, AbortSignal<'js>>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> AbortController<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> Result<Self> {
        Ok(Self {
            signal: Class::instance(ctx, AbortSignal::new())?,
        })
    }

    fn abort(&self) {}
}

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();

    Class::<AbortSignal>::define(&globals)?;
    Class::<AbortController>::define(&globals)?;

    Ok(())
}
