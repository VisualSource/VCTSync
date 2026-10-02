use rquickjs::{Class, Ctx, JsLifetime, Object, Result, Value, class::Trace, function::Opt};

use std::{str::FromStr, sync::OnceLock, time::Duration};

use reqwest::{
    Client, Method, StatusCode,
    header::{HeaderName, HeaderValue},
};

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

    #[qjs(get)]
    url: String,
}

impl Response {
    pub fn from_request(resp: reqwest::Response, status: StatusCode) -> Self {
        Self {
            inner: Some(resp),
            status: status,
            url: String::new(),
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl Response {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {
            inner: None,
            status: StatusCode::OK,
            url: String::default(),
        }
    }

    #[qjs(get)]
    fn ok(&self) -> bool {
        self.status.is_success() || self.status.is_client_error()
    }

    #[qjs(get)]
    fn redirected(&self) -> bool {
        self.status.is_redirection()
    }

    #[qjs(get)]
    fn status_text(&self) -> String {
        self.status.to_string()
    }

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

    async fn array_buffer(&mut self) {}
    async fn blob(&mut self) {}
    async fn bytes(&mut self) {}

    async fn clone(&mut self) {}

    async fn form_data(&mut self) {}

    async fn json<'js>(&mut self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let text = self.text(ctx.clone()).await?;

        let value = ctx.json_parse(text)?;

        Ok(value)
    }

    async fn text_stream(&mut self) {}
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class(frozen)]
struct Headers {
    #[qjs(skip_trace)]
    map: reqwest::header::HeaderMap,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl Headers {
    #[qjs(constructor)]
    fn new() -> Self {
        Self {
            map: reqwest::header::HeaderMap::new(),
        }
    }

    fn append<'js>(&mut self, ctx: Ctx<'js>, key: String, value: String) -> Result<()> {
        let header_value = HeaderValue::from_str(&value).map_err(|err| {
            rquickjs::Error::new_from_js_message("string", "HeaderValue", err.to_string())
        })?;

        let header_name = HeaderName::from_str(&key).map_err(|err| {
            rquickjs::Error::new_into_js_message("string", "HeaderName", err.to_string())
        })?;

        self.map
            .try_append(header_name, header_value)
            .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?;

        Ok(())
    }
    fn delete(&mut self, key: String) {
        self.map.remove(key);
    }
    fn entries(&self) {}
    fn for_each<'js>(&self, ctx: Ctx<'js>, func: rquickjs::Function<'js>) -> Result<()> {
        for (key, value) in self.map.iter() {
            let key = key.to_string();
            let value = value
                .to_str()
                .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?
                .to_string();

            func.call::<(String, String), ()>((key, value))?;
        }

        Ok(())
    }

    fn get<'js>(&self, _key: String, ctx: Ctx<'js>) -> Value<'js> {
        rquickjs::Null.into_value(ctx)
    }
    fn get_set_cookie(&mut self) {}
    fn has(&self, key: String) -> bool {
        self.map.contains_key(&key)
    }
    fn keys(&self) {}
    fn set<'js>(&mut self, ctx: Ctx<'js>, key: String, value: String) -> rquickjs::Result<()> {
        let header_value = HeaderValue::from_str(&value).map_err(|err| {
            rquickjs::Error::new_from_js_message("string", "HeaderValue", err.to_string())
        })?;

        let header_name = HeaderName::from_str(&key).map_err(|err| {
            rquickjs::Error::new_into_js_message("string", "HeaderName", err.to_string())
        })?;

        self.map
            .try_insert(header_name, header_value)
            .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?;

        Ok(())
    }

    fn values(&self) {}
}

// input should be RequestInit or URL
#[rquickjs::function]
async fn fetch<'js>(ctx: Ctx<'js>, input: String, init: Opt<Object<'js>>) -> Result<Response> {
    let client = get_client();

    let method = if let Some(opt) = init.0 {
        let raw = opt
            .get::<_, String>("method")
            .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?;

        //let signal = opt.get::<_, Class<'js, AbortSignal>>("signal");

        Method::from_str(&raw)
            .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?
    } else {
        Method::GET
    };

    let url = input;

    let req = client.request(method, url);

    let response = match req.send().await {
        Ok(r) => r,
        Err(err) => {
            return Err(rquickjs::Exception::throw_internal(&ctx, &err.to_string()));
        }
    };

    let status = response.status();

    Ok(Response::from_request(response, status))
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    globals.set("fetch", js_fetch)?;
    Class::<Response>::define(&globals)?;
    Class::<Headers>::define(&globals)?;

    Ok(())
}
