use rquickjs::{Ctx, JsLifetime, class::Trace, function::Opt};

mod relative_time_format;

use relative_time_format::RelativeTimeFormat;

#[derive(Trace, JsLifetime)]
#[rquickjs::class(frozen)]
pub struct Intl {}

#[rquickjs::methods(rename_all = "PascalCase")]
impl<'js> Intl {
    #[qjs(static)]
    pub fn relative_time_format(
        ctx: Ctx<'js>,
        locale: rquickjs::Value<'js>,
        opts: Opt<rquickjs::Object<'js>>,
    ) -> rquickjs::Result<rquickjs::Class<'js, RelativeTimeFormat>> {
        todo!()
    }
}
