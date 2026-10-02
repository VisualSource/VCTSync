use super::headers::Headers;
use crate::js_host::abort_controller::AbortSignal;
use rquickjs::{Class, Coerced, Ctx, JsLifetime, class::Trace, function::Opt};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct Request<'js> {
    #[qjs(get)]
    pub method: String,
    #[qjs(get)]
    pub headers: Class<'js, Headers>,
    #[qjs(get)]
    pub signal: Class<'js, AbortSignal<'js>>,
    #[qjs(get)]
    pub url: String,
}

#[rquickjs::methods]
impl<'js> Request<'js> {
    #[qjs(constructor)]
    pub fn constructor(
        ctx: Ctx<'js>,
        input: rquickjs::Value<'js>,
        options: Opt<rquickjs::Object<'js>>,
    ) -> rquickjs::Result<Self> {
        let mut req = match input.get::<Class<'js, Request<'js>>>() {
            Ok(other) => {
                let other = other.borrow();

                Self {
                    url: other.url.clone(),
                    method: other.method.clone(),
                    headers: Class::instance(
                        ctx.clone(),
                        Headers {
                            map: other.headers.borrow().map.clone(),
                        },
                    )?,
                    signal: other.signal.clone(),
                }
            }
            Err(_) => Self {
                url: input.get::<Coerced<String>>()?.0,
                method: "GET".to_string(),
                headers: Class::instance(
                    ctx.clone(),
                    Headers {
                        map: Default::default(),
                    },
                )?,
                signal: Class::instance(ctx.clone(), AbortSignal::new())?,
            },
        };

        if let Some(opts) = options.0 {
            if let Ok(method) = opts.get::<_, String>("method") {
                req.method = method.to_uppercase();
            }

            let headers = opts.get::<_, rquickjs::Value<'js>>("headers")?;
            if !headers.is_undefined() {
                req.headers = Class::instance(
                    ctx.clone(),
                    Headers::new(ctx.clone(), Opt::from(Some(headers)))?,
                )?;
            }

            if let Ok(signal) = opts.get::<_, Class<'js, AbortSignal<'js>>>("signal") {
                req.signal = signal
            }
        }

        Ok(req)
    }
}
