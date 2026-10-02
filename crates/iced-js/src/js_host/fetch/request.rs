use super::headers::Headers;
use crate::js_host::abort_contoller::AbortSignal;
use rquickjs::{Class, Ctx, JsLifetime, class::Trace};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct Request<'js> {
    pub method: Option<String>,
    pub headers: Headers,
    pub signal: Option<Class<'js, AbortSignal<'js>>>,
    pub url: String,
}

#[rquickjs::methods]
impl<'js> Request<'js> {
    #[qjs(constructor)]
    pub fn constructor(_ctx: Ctx<'js>, value: rquickjs::Object<'js>) -> rquickjs::Result<Self> {
        let signal = value.get::<_, Class<'js, AbortSignal<'js>>>("signal").ok();
        let method = value.get::<_, String>("method").ok();

        Ok(Self {
            headers: Headers::new(),
            method,
            signal,
            url: String::default(),
        })
    }
}
