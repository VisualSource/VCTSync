use jiff::{
    Span,
    fmt::friendly::{Designator, Direction, Spacing, SpanPrinter},
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
        _locals: Opt<rquickjs::Value<'js>>,
        options: Opt<Object<'js>>,
    ) -> Result<Self> {
        let style = options
            .0
            .as_ref()
            .and_then(|x| x.get::<_, String>("style").ok())
            .unwrap_or_else(|| "long".to_string());

        let style = match style.as_str() {
            "long" => Designator::Verbose,
            "short" => Designator::Short,
            "narrow" => Designator::Compact,
            _ => {
                return Err(rquickjs::Exception::throw_range(
                    &ctx,
                    "invalid style option",
                ));
            }
        };

        let formmater = SpanPrinter::new()
            .designator(style)
            .spacing(Spacing::BetweenUnitsAndDesignators)
            .direction(Direction::Suffix);

        Ok(Self { formmater })
    }

    fn format(&self, ctx: rquickjs::Ctx<'js>, value: i64, unit: String) -> Result<String> {
        let span = match unit.as_str() {
            "minute" => Span::new().minutes(value),
            "hour" => Span::new().hours(value),
            "day" => Span::new().days(value),
            _ => return Err(rquickjs::Exception::throw_range(&ctx, "unknown unit")),
        };

        Ok(self.formmater.span_to_string(&span))
    }
}
