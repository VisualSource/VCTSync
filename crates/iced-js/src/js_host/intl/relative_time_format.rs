use rquickjs::{JsLifetime, class::Trace};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct RelativeTimeFormat {}

#[rquickjs::methods]
impl<'js> RelativeTimeFormat {
    fn format(
        ctx: rquickjs::Ctx<'js>,
        value: rquickjs::Value<'js>,
        unit: String,
    ) -> rquickjs::Result<String> {
        if !value.is_int() {
            return Err(rquickjs::Exception::throw_type(&ctx, "was expecting int"));
        }

        unimplemented!()
    }
}
