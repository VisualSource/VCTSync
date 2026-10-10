use jiff::{
    Span,
    fmt::friendly::{Designator, Direction, SpanPrinter},
};
use rquickjs::{Ctx, JsLifetime, Object, Result, class::Trace, function::Opt};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct RelativeTimeFormat {
    #[qjs(skip_trace)]
    formmater: SpanPrinter,
}

#[rquickjs::methods]
impl<'js> RelativeTimeFormat {
    #[qjs(constructor)]
    fn constructor(
        ctx: Ctx<'js>,
        locals: Opt<rquickjs::Value<'js>>,
        options: Opt<Object<'js>>,
    ) -> Result<Self> {
        let formmater = SpanPrinter::new()
            .designator(Designator::Verbose)
            .direction(Direction::Suffix);

        Ok(Self { formmater })
    }

    fn format(&self, ctx: rquickjs::Ctx<'js>, value: i64, unit: String) -> Result<String> {
        let mut span = Span::new();

        match unit.as_str() {
            "minute" => {
                span = span.minutes(value);
            }
            "hour" => {
                span = span.hours(value);
            }
            "day" => {
                span = span.days(value);
            }
            _ => return Err(rquickjs::Exception::throw_range(&ctx, "unknown unit")),
        }

        Ok(self.formmater.span_to_string(&span))
    }
}
