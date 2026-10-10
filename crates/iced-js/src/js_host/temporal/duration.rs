use rquickjs::{Ctx, JsLifetime, class::Trace, function::Opt};

#[derive(JsLifetime, Trace)]
#[rquickjs::class]
pub struct Duration {
    #[qjs(skip_trace)]
    data: jiff::Span,
}

impl Duration {
    pub fn new(data: jiff::Span) -> Self {
        Self { data }
    }
}

#[rquickjs::methods]
impl<'js> Duration {
    #[qjs(constructor)]
    fn constructor(
        ctx: Ctx<'js>,
        years: Opt<i64>,
        months: Opt<i64>,
        weeks: Opt<i64>,
        days: Opt<i64>,
        hours: Opt<i64>,
        minutes: Opt<i64>,
        seconds: Opt<i64>,
        milliseconds: Opt<i64>,
        microseconds: Opt<i64>,
        nanoseconds: Opt<i64>,
    ) -> rquickjs::Result<Self> {
        let items = [
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        ]
        .map(|v| v.unwrap_or_default());

        let range_error =
            |err: jiff::Error| rquickjs::Exception::throw_range(&ctx, &err.to_string());

        let span = jiff::Span::new()
            .try_years(items[0])
            .map_err(range_error)?
            .try_months(items[1])
            .map_err(range_error)?
            .try_weeks(items[2])
            .map_err(range_error)?
            .try_days(items[3])
            .map_err(range_error)?
            .try_hours(items[4])
            .map_err(range_error)?
            .try_minutes(items[5])
            .map_err(range_error)?
            .try_seconds(items[6])
            .map_err(range_error)?
            .try_milliseconds(items[7])
            .map_err(range_error)?
            .try_microseconds(items[8])
            .map_err(range_error)?
            .try_nanoseconds(items[9])
            .map_err(range_error)?;

        let sign = span.signum() as i64;
        if items.iter().any(|v| v.signum() != 0 && v.signum() != sign) {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "mixed-sign values not allowed as duration fields",
            ));
        }

        Ok(Self { data: span })
    }

    #[qjs(get)]
    fn days(&self) -> i32 {
        self.data.get_days()
    }

    #[qjs(get)]
    fn hours(&self) -> i32 {
        self.data.get_hours()
    }

    #[qjs(get)]
    fn minutes(&self) -> i64 {
        self.data.get_minutes()
    }
}

#[cfg(test)]
mod tests {
    use rquickjs::{CatchResultExt, CaughtError, Context, Runtime};

    /// Evaluate `source` as a script with the `Temporal` global installed and
    /// return its completion value.
    fn eval<T>(source: &str) -> T
    where
        T: for<'js> rquickjs::FromJs<'js>,
    {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        ctx.with(|ctx| {
            super::super::init(&ctx).unwrap();
            ctx.eval::<T, _>(source).catch(&ctx).unwrap()
        })
    }

    /// Evaluate `source`, expecting it to throw, and return the thrown
    /// exception's `name`.
    fn eval_throws(source: &str) -> String {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        ctx.with(|ctx| {
            super::super::init(&ctx).unwrap();
            match ctx.eval::<rquickjs::Value, _>(source).catch(&ctx) {
                Ok(_) => panic!("expected `{source}` to throw"),
                Err(CaughtError::Exception(e)) => e.get::<_, String>("name").unwrap(),
                Err(e) => panic!("expected an exception, got {e}"),
            }
        })
    }

    #[test]
    fn is_exposed_on_temporal() {
        let is_fn: bool = eval("typeof Temporal.Duration === 'function'");
        assert!(is_fn);
    }

    #[test]
    fn constructor_creates_instance() {
        let is_instance: bool = eval("new Temporal.Duration() instanceof Temporal.Duration");
        assert!(is_instance);
    }

    #[test]
    fn constructor_defaults_to_zero() {
        let parts: Vec<i64> = eval(
            r#"
            const d = new Temporal.Duration();
            [d.days, d.hours, d.minutes]
            "#,
        );
        assert_eq!(parts, vec![0, 0, 0]);
    }

    #[test]
    fn constructor_sets_fields() {
        let parts: Vec<i64> = eval(
            r#"
            const d = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
            [d.days, d.hours, d.minutes]
            "#,
        );
        assert_eq!(parts, vec![4, 5, 6]);
    }

    #[test]
    fn constructor_does_not_balance() {
        let parts: Vec<i64> = eval(
            r#"
            const d = new Temporal.Duration(0, 0, 0, 0, 0, 90);
            [d.hours, d.minutes]
            "#,
        );
        assert_eq!(parts, vec![0, 90]);
    }

    #[test]
    fn constructor_accepts_negative() {
        let parts: Vec<i64> = eval(
            r#"
            const d = new Temporal.Duration(0, 0, 0, -2, -3, -4);
            [d.days, d.hours, d.minutes]
            "#,
        );
        assert_eq!(parts, vec![-2, -3, -4]);
    }

    #[test]
    fn constructor_rejects_out_of_range_years() {
        assert_eq!(eval_throws("new Temporal.Duration(20000)"), "RangeError");
    }

    #[test]
    fn constructor_rejects_out_of_range_days() {
        assert_eq!(
            eval_throws("new Temporal.Duration(0, 0, 0, 1e9)"),
            "RangeError"
        );
    }

    #[test]
    fn constructor_rejects_mixed_signs() {
        assert_eq!(
            eval_throws("new Temporal.Duration(0, 0, 0, 1, -1)"),
            "RangeError"
        );
    }
}
