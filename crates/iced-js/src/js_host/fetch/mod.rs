mod headers;
mod request;
mod response;
use crate::js_host::abort_controller::AbortSignal;
use headers::Headers;
use request::Request;
use reqwest::{Client, Method};
use response::Response;
use rquickjs::{Class, Ctx, Object, Result, Value, function::Opt};
use std::{str::FromStr, sync::OnceLock, time::Duration};

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

    /*TODO check for url class */
    let (method, signal, url, headers) = if let Ok(url) = resource.get::<String>() {
        let url = reqwest::Url::parse(&url)
            .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?;

        if let Some(request_init) = init.0 {
            let request = Request::constructor(ctx.clone(), request_init)?;

            let method = request
                .method
                .as_ref()
                .map(|x| Method::from_str(x))
                .unwrap_or_else(|| Ok(Method::GET))
                .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?;

            let headers = request.headers.map(|x| x.map.clone());

            (method, request.signal.clone(), url, headers)
        } else {
            (Method::GET, None, url, None)
        }
    } else if let Ok(request) = resource.get::<Class<'js, Request<'js>>>() {
        let state = request.borrow();

        let method = state
            .method
            .as_ref()
            .map(|x| Method::from_str(x))
            .unwrap_or_else(|| Ok(Method::GET))
            .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?;

        let url = reqwest::Url::parse(&state.url)
            .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?;

        let headers = state.headers.as_ref().map(|x| x.map.clone());

        (method, state.signal.clone(), url, headers)
    } else {
        return Err(rquickjs::Exception::throw_type(
            &ctx,
            "invalid resource type",
        ));
    };

    if let Some(signal) = &signal
        && signal.borrow().aborted
    {
        return Err(abort_rejection(&ctx, signal));
    }

    let mut req = client.request(method, url);

    if let Some(headers) = headers {
        req = req.headers(headers);
    }

    let notify = signal.as_ref().map(|s| s.borrow().notify.clone());

    let response = match notify {
        Some(n) => tokio::select! {
            r = req.send() => r,
            _ = n.notified()=> return Err(abort_rejection(&ctx, &signal.expect("should have signal")))
        },
        None => req.send().await
    }.map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?;

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
