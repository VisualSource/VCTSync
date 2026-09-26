use rquickjs::{Class, Ctx, JsLifetime, Object, Result, class::Trace, function::Opt};

use std::{sync::OnceLock, time::Duration};

use reqwest::{Client, Method, StatusCode};

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent("vct-sync")
            .timeout(Duration::from_mins(1))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build http client")
    })
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
struct Response {
    #[qjs(skip_trace)]
    inner: Option<reqwest::Response>,
    #[qjs(skip_trace)]
    status: StatusCode,
    #[qjs(skip_trace)]
    status_text: String,

    #[qjs(skip_trace)]
    url: String,
}

impl Response {
    pub fn new(resp: reqwest::Response, status: StatusCode) -> Self {
        Self {
            inner: Some(resp),
            status: status,
            status_text: String::new(),
            url: String::new(),
        }
    }
}

#[rquickjs::methods]
impl Response {
    async fn text<'js>(&mut self, ctx: Ctx<'js>) -> Result<String> {
        let body = self.inner.take();

        let Some(resp) = body else {
            return Err(rquickjs::Exception::throw_type(
                &ctx,
                "body already disturbed",
            ));
        };

        let text = resp
            .text()
            .await
            .map_err(|err| rquickjs::Exception::throw_dom(&ctx, "network", &err.to_string()))?;

        Ok(text)
    }
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class(frozen)]
struct Headers {
    #[qjs(skip_trace)]
    map: reqwest::header::HeaderMap,
}

// input should be RequestInit or URL
#[rquickjs::function]
async fn fetch<'js>(ctx: Ctx<'js>, input: String, init: Opt<Object<'js>>) -> Result<Response> {
    let client = get_client();

    let method = Method::GET;
    let url = "";

    let req = client.request(method, url);

    let response = match req.send().await {
        Ok(r) => r,
        Err(err) => {
            log::error!("{}", err);
            return Err(rquickjs::Error::Exception);
        }
    };

    let status = response.status();

    Ok(Response::new(response, status))
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    globals.set("fetch", js_fetch)?;
    Class::<Response>::define(&globals)?;
    Class::<Headers>::define(&globals)?;

    Ok(())
}
