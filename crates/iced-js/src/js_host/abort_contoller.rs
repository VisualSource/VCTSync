use rquickjs::{
    Class, Ctx, Function, JsLifetime, Result, String, Value, class::Trace, function::Opt,
};

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

    listeners: Vec<Function<'js>>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> AbortSignal<'js> {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {
            aborted: false,
            reason: None,
            listeners: Vec::new(),
        }
    }

    fn abort(reason: Opt<String<'js>>) -> Self {
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

    fn add_event_listener(&mut self, func: Function<'js>) {
        self.listeners.push(func);
    }
    fn remove_event_listener(&mut self, func: Function<'js>) {
        let i = self.listeners.iter().position(|fun| fun.eq(&func));

        if let Some(idx) = i {
            self.listeners.swap_remove(idx);
        }
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
