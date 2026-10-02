mod headers;
mod request;
mod response;
use crate::js_host::abort_controller::AbortSignal;
use headers::Headers;
use request::Request;
use reqwest::{Client, Method};
use response::Response;
use rquickjs::{Class, Ctx, Object, Result, Value, function::Opt};
use std::{sync::OnceLock, time::Duration};

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

// input should be RequestInit or URL
#[rquickjs::function]
async fn fetch<'js>(
    ctx: Ctx<'js>,
    resource: Value<'js>,
    init: Opt<Object<'js>>,
) -> Result<Response> {
    let client = get_client();
    let request = Request::constructor(ctx.clone(), resource, init)?;

    let signal = &request.signal;
    if signal.borrow().aborted {
        return Err(abort_rejection(&ctx, signal));
    }

    let url = reqwest::Url::parse(&request.url)
        .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?;

    let method = request
        .method
        .parse::<Method>()
        .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?;

    let req = client
        .request(method, url)
        .headers(request.headers.borrow().map.clone());

    let notify = signal.borrow().notify.clone();
    let response = tokio::select! {
        r = req.send() => r,
        _ = notify.notified()=> return Err(abort_rejection(&ctx, signal))
    }
    .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?;

    let status = response.status();

    Ok(Response::from_request(response, status))
}

fn abort_rejection<'js>(ctx: &Ctx<'js>, signal: &Class<'js, AbortSignal<'js>>) -> rquickjs::Error {
    let s = signal.borrow();

    if let Some(reason) = &s.reason {
        return ctx.throw(reason.clone());
    }

    rquickjs::Exception::throw_dom(ctx, "AbortError", "abort rejection")
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    globals.set("fetch", js_fetch)?;
    Class::<Response>::define(&globals)?;
    Class::<Headers>::define(&globals)?;
    Class::<Request>::define(&globals)?;

    Ok(())
}
