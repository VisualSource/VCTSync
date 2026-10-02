use super::headers::Headers;
use crate::js_host::abort_controller::AbortSignal;
use rquickjs::{Class, Ctx, JsLifetime, class::Trace, function::Opt};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct Request<'js> {
    pub method: Option<String>,
    pub headers: Option<Headers>,
    pub signal: Option<Class<'js, AbortSignal<'js>>>,
    pub url: String,
}

#[rquickjs::methods]
impl<'js> Request<'js> {
    #[qjs(constructor)]
    pub fn constructor(ctx: Ctx<'js>, value: rquickjs::Object<'js>) -> rquickjs::Result<Self> {
        let url = value.get::<_, String>("url").ok().unwrap_or_default();

        let signal = value.get::<_, Class<'js, AbortSignal<'js>>>("signal").ok();
        let method = value.get::<_, String>("method").ok();

        let headers = if value.contains_key("headers")? {
            let h = value.get::<_, rquickjs::Value<'js>>("headers")?;
            Some(Headers::new(ctx, Opt::from(Some(h)))?)
        } else {
            None
        };

        Ok(Self {
            headers,
            method,
            signal,
            url,
        })
    }
}
