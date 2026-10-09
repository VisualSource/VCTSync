use rquickjs::{Class, Ctx, JsLifetime, class::Trace};

use crate::js_host::temporal::plain_date_time::PlainDateTime;

#[derive(Trace, JsLifetime)]
#[rquickjs::class(frozen)]
pub struct Now {}

impl Now {
    pub fn new() -> Self {
        Self {}
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> Now {
    pub fn plain_date_time_iso(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<rquickjs::Class<'js, PlainDateTime>> {
        let now = jiff::Zoned::now();

        Class::instance(ctx, PlainDateTime::new(now))
    }
}
