use reqwest::StatusCode;
use rquickjs::{Ctx, JsLifetime, Result, Value, class::Trace};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct Response {
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
            status,
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

    async fn array_buffer(&mut self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Response.arrayBuffer is not implemented",
        ))
    }
    async fn blob(&mut self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Response.blob is not implemented",
        ))
    }
    async fn bytes<'js>(
        &mut self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<rquickjs::TypedArray<'js, u8>> {
        let Some(resp) = self.inner.take() else {
            return Err(rquickjs::Exception::throw_type(
                &ctx,
                "body already disturbed",
            ));
        };

        let body = resp
            .bytes()
            .await
            .map_err(|err| rquickjs::Exception::throw_dom(&ctx, "network", &err.to_string()))?;

        rquickjs::TypedArray::new_copy(ctx, &body[..])
    }

    async fn clone(&mut self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Response.clone is not implemented",
        ))
    }

    async fn form_data(&mut self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Response.formData is not implemented",
        ))
    }

    async fn json<'js>(&mut self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let text = self.text(ctx.clone()).await?;

        let value = ctx.json_parse(text)?;

        Ok(value)
    }

    async fn text_stream(&mut self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Response.textStream is not implemented",
        ))
    }
}
