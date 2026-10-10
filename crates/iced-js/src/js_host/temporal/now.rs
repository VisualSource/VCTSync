use rquickjs::{Class, Ctx, JsLifetime, class::Trace, function::Opt};

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
    #[qjs(rename = "plainDateTimeISO")]
    pub fn plain_date_time_iso(
        &self,
        ctx: Ctx<'js>,
        tz: Opt<String>,
    ) -> rquickjs::Result<rquickjs::Class<'js, PlainDateTime>> {
        let tz = match tz.0 {
            Some(name) => jiff::tz::TimeZone::get(&name)
                .map_err(|err| rquickjs::Exception::throw_range(&ctx, &err.to_string()))?,
            None => jiff::tz::TimeZone::system(),
        };

        let now = jiff::Timestamp::now().to_zoned(tz);
        Class::instance(ctx, PlainDateTime::new(now.datetime()))
    }
}
