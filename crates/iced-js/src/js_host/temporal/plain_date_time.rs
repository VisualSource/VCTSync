use rquickjs::{Class, Ctx, JsLifetime, Object, atom::PredefinedAtom, class::Trace, function::Opt};

use super::duration::Duration;

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct PlainDateTime {
    #[qjs(skip_trace)]
    pub(crate) data: jiff::civil::DateTime,
}

impl PlainDateTime {
    pub fn new(data: jiff::civil::DateTime) -> Self {
        Self { data }
    }
}

#[rquickjs::methods]
impl<'js> PlainDateTime {
    #[qjs(static)]
    fn compare(
        ctx: Ctx<'js>,
        a: rquickjs::Value<'js>,
        b: rquickjs::Value<'js>,
    ) -> rquickjs::Result<i8> {
        let a = PlainDateTime::from(ctx.clone(), a, Opt::from(None))?;
        let b = PlainDateTime::from(ctx, b, Opt::from(None))?;

        let inner_a = a.borrow();
        let inner_b = b.borrow();
        Ok(inner_a.data.cmp(&inner_b.data) as i8)
    }

    #[qjs(static)]
    fn from(
        ctx: Ctx<'js>,
        info: rquickjs::Value<'js>,
        _opts: Opt<rquickjs::Object<'js>>,
    ) -> rquickjs::Result<rquickjs::Class<'js, PlainDateTime>> {
        if let Ok(v) = info.get::<String>() {
            let zdt = v
                .parse::<jiff::civil::DateTime>()
                .map_err(|err| rquickjs::Exception::throw_range(&ctx, &err.to_string()))?;

            Class::instance(ctx, PlainDateTime { data: zdt })
        } else if let Ok(inst) = info.get::<rquickjs::Class<'js, PlainDateTime>>() {
            let inst = inst.borrow();

            let data = inst.data.clone();

            Class::instance(ctx, PlainDateTime { data })
        } else if let Ok(_obj) = info.get::<Object<'js>>() {
            unimplemented!()
        } else {
            Err(rquickjs::Exception::throw_type(
                &ctx,
                "was expecting an object or string",
            ))
        }
    }

    #[qjs(constructor)]
    fn constructor(
        ctx: Ctx<'js>,
        year: i16,
        month: i8,
        day: i8,
        hour: Opt<i8>,
        minute: Opt<i8>,
        second: Opt<i8>,
        millisecond: Opt<i32>,
        microsecond: Opt<i32>,
        nanosecond: Opt<i32>,
    ) -> rquickjs::Result<Self> {
        let ms = millisecond.unwrap_or_default();
        let us = microsecond.unwrap_or_default();
        let ns = nanosecond.unwrap_or_default();

        for (name, v) in [("millisecond", ms), ("microsecond", us), ("nanosecond", ns)] {
            if !(0..=999).contains(&v) {
                return Err(rquickjs::Exception::throw_range(
                    &ctx,
                    &format!("{name} must be between 0 and 999"),
                ));
            }
        }

        let subsec = ms as i32 * 1_000_000 + us * 1_000 + ns;

        let dt = jiff::civil::DateTime::new(
            year,
            month,
            day,
            hour.unwrap_or_default(),
            minute.unwrap_or_default(),
            second.unwrap_or_default(),
            subsec,
        )
        .map_err(|e| rquickjs::Exception::throw_range(&ctx, &e.to_string()))?;

        Ok(Self { data: dt })
    }

    #[qjs(get)]
    fn day(&self) -> i8 {
        self.data.day()
    }

    #[qjs(get)]
    fn month(&self) -> i8 {
        self.data.month()
    }

    #[qjs(get)]
    fn millisecond(&self) -> i16 {
        self.data.millisecond()
    }

    #[qjs(get)]
    fn microsecond(&self) -> i16 {
        self.data.microsecond()
    }

    #[qjs(get)]
    fn nanosecond(&self) -> i16 {
        self.data.nanosecond()
    }

    fn until(
        &self,
        ctx: Ctx<'js>,
        other: rquickjs::Value<'js>,
        options: Opt<rquickjs::Object<'js>>,
    ) -> rquickjs::Result<Class<'js, Duration>> {
        if let Ok(other) = other.get::<Class<'js, PlainDateTime>>() {
            let other = other.borrow();

            let mut diff = jiff::civil::DateTimeDifference::new(other.data);
            if let Some(opts) = options.0
                && let Some(unit) = opts.get::<_, String>("largestUnit").ok()
            {
                diff = diff.largest(parse_unit(&ctx, &unit)?);
            }

            let dur = self
                .data
                .until(diff)
                .map_err(|err| rquickjs::Exception::throw_range(&ctx, &err.to_string()))?;

            return Class::instance(ctx, Duration::new(dur));
        }

        unimplemented!()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    fn js_to_string(&self) -> String {
        self.data.to_string()
    }
}

fn parse_unit(ctx: &Ctx<'_>, s: &str) -> rquickjs::Result<jiff::Unit> {
    use jiff::Unit;

    Ok(match s.strip_suffix('s').unwrap_or(s) {
        "year" => Unit::Year,
        "month" => Unit::Month,
        "week" => Unit::Week,
        "day" | "auto" => Unit::Day,
        "hour" => Unit::Hour,
        "minute" => Unit::Minute,
        "second" => Unit::Second,
        "millisecond" => Unit::Millisecond,
        "microsecond" => Unit::Microsecond,
        "nanosecond" => Unit::Nanosecond,
        _ => {
            return Err(rquickjs::Exception::throw_range(
                ctx,
                &format!("invalid unit: {s}"),
            ));
        }
    })
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
        let ty: String = eval("typeof Temporal.PlainDateTime");
        assert_eq!(ty, "function");
    }

    #[test]
    fn constructor_creates_instance() {
        let is_instance: bool =
            eval("new Temporal.PlainDateTime(2024, 3, 15) instanceof Temporal.PlainDateTime");
        assert!(is_instance);
    }

    #[test]
    fn constructor_sets_day() {
        let day: i32 = eval("new Temporal.PlainDateTime(2024, 3, 15).day");
        assert_eq!(day, 15);
    }

    #[test]
    fn constructor_accepts_time_fields() {
        let day: i32 = eval("new Temporal.PlainDateTime(2024, 3, 15, 13, 45, 30).day");
        assert_eq!(day, 15);
    }

    #[test]
    fn constructor_rejects_invalid_month() {
        assert_eq!(
            eval_throws("new Temporal.PlainDateTime(2024, 13, 1)"),
            "RangeError"
        );
    }

    #[test]
    fn constructor_rejects_invalid_day() {
        assert_eq!(
            eval_throws("new Temporal.PlainDateTime(2023, 2, 29)"),
            "RangeError"
        );
    }

    #[test]
    fn constructor_accepts_leap_day() {
        let day: i32 = eval("new Temporal.PlainDateTime(2024, 2, 29).day");
        assert_eq!(day, 29);
    }

    #[test]
    fn constructor_rejects_invalid_hour() {
        assert_eq!(
            eval_throws("new Temporal.PlainDateTime(2024, 1, 1, 24)"),
            "RangeError"
        );
    }

    #[test]
    fn from_parses_iso_string() {
        let day: i32 = eval("Temporal.PlainDateTime.from('2024-03-15T13:45:30').day");
        assert_eq!(day, 15);
    }

    #[test]
    fn from_parses_date_only_string() {
        let day: i32 = eval("Temporal.PlainDateTime.from('2024-03-15').day");
        assert_eq!(day, 15);
    }

    #[test]
    fn from_returns_instance() {
        let is_instance: bool = eval(
            "Temporal.PlainDateTime.from('2024-03-15T00:00') instanceof Temporal.PlainDateTime",
        );
        assert!(is_instance);
    }

    #[test]
    fn from_rejects_invalid_string() {
        assert_eq!(
            eval_throws("Temporal.PlainDateTime.from('not a date')"),
            "RangeError"
        );
    }

    #[test]
    fn from_rejects_non_object() {
        assert_eq!(eval_throws("Temporal.PlainDateTime.from(42)"), "TypeError");
    }

    #[test]
    fn from_copies_existing_instance() {
        let day: i32 = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15);
            Temporal.PlainDateTime.from(a).day
            "#,
        );
        assert_eq!(day, 15);
    }

    #[test]
    fn from_instance_returns_new_object() {
        let same: bool = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15);
            a === Temporal.PlainDateTime.from(a)
            "#,
        );
        assert!(!same, "from() should return a new object");
    }

    #[test]
    fn until_returns_duration() {
        let is_duration: bool = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15);
            const b = new Temporal.PlainDateTime(2024, 3, 16);
            a.until(b) instanceof Temporal.Duration
            "#,
        );
        assert!(is_duration);
    }

    #[test]
    fn until_measures_forward_span() {
        let parts: Vec<i64> = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15, 10, 0);
            const b = new Temporal.PlainDateTime(2024, 3, 15, 12, 30);
            const d = a.until(b);
            [d.hours, d.minutes]
            "#,
        );
        assert_eq!(parts, vec![2, 30]);
    }

    #[test]
    fn until_crosses_day_boundary() {
        let parts: Vec<i64> = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15, 22);
            const b = new Temporal.PlainDateTime(2024, 3, 17, 1);
            const d = a.until(b);
            [d.days, d.hours]
            "#,
        );
        assert_eq!(parts, vec![1, 3]);
    }

    #[test]
    fn until_balances_to_days() {
        let parts: Vec<i64> = eval(
            r#"
            const a = Temporal.PlainDateTime.from('2024-03-01T00:00');
            const b = Temporal.PlainDateTime.from('2024-03-11T05:07');
            const d = a.until(b);
            [d.days, d.hours, d.minutes]
            "#,
        );
        assert_eq!(parts, vec![10, 5, 7]);
    }

    #[test]
    fn until_is_negative_for_earlier_target() {
        let hours: i64 = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15, 12);
            const b = new Temporal.PlainDateTime(2024, 3, 15, 9);
            a.until(b).hours
            "#,
        );
        assert_eq!(hours, -3);
    }

    #[test]
    fn until_same_instant_is_zero() {
        let minutes: i64 = eval(
            r#"
            const a = Temporal.PlainDateTime.from('2024-03-15T08:00');
            a.until(Temporal.PlainDateTime.from(a)).minutes
            "#,
        );
        assert_eq!(minutes, 0);
    }

    #[test]
    fn until_largest_unit_hours() {
        let parts: Vec<i64> = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15, 22);
            const b = new Temporal.PlainDateTime(2024, 3, 17, 1);
            const d = a.until(b, { largestUnit: "hours" });
            [d.days, d.hours]
            "#,
        );
        assert_eq!(parts, vec![0, 27]);
    }

    #[test]
    fn until_largest_unit_minutes() {
        let parts: Vec<i64> = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15, 10, 0);
            const b = new Temporal.PlainDateTime(2024, 3, 15, 12, 30);
            const d = a.until(b, { largestUnit: "minutes" });
            [d.hours, d.minutes]
            "#,
        );
        assert_eq!(parts, vec![0, 150]);
    }

    #[test]
    fn until_largest_unit_accepts_singular_and_plural() {
        let parts: Vec<i64> = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15, 22);
            const b = new Temporal.PlainDateTime(2024, 3, 17, 1);
            const plural = a.until(b, { largestUnit: "days" });
            const singular = a.until(b, { largestUnit: "day" });
            [plural.days, plural.hours, singular.days, singular.hours]
            "#,
        );
        assert_eq!(parts, vec![1, 3, 1, 3]);
    }

    #[test]
    fn until_largest_unit_auto_is_days() {
        let parts: Vec<i64> = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15, 22);
            const b = new Temporal.PlainDateTime(2024, 3, 17, 1);
            const d = a.until(b, { largestUnit: "auto" });
            [d.days, d.hours]
            "#,
        );
        assert_eq!(parts, vec![1, 3]);
    }

    #[test]
    fn until_largest_unit_negative() {
        let parts: Vec<i64> = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 17, 1);
            const b = new Temporal.PlainDateTime(2024, 3, 15, 22);
            const d = a.until(b, { largestUnit: "hours" });
            [d.days, d.hours]
            "#,
        );
        assert_eq!(parts, vec![0, -27]);
    }

    #[test]
    fn until_empty_options_uses_default() {
        let parts: Vec<i64> = eval(
            r#"
            const a = new Temporal.PlainDateTime(2024, 3, 15, 22);
            const b = new Temporal.PlainDateTime(2024, 3, 17, 1);
            const d = a.until(b, {});
            [d.days, d.hours]
            "#,
        );
        assert_eq!(parts, vec![1, 3]);
    }

    #[test]
    fn until_rejects_invalid_largest_unit() {
        assert_eq!(
            eval_throws(
                r#"
                const a = new Temporal.PlainDateTime(2024, 3, 15);
                const b = new Temporal.PlainDateTime(2024, 3, 16);
                a.until(b, { largestUnit: "fortnights" })
                "#
            ),
            "RangeError"
        );
    }

    #[test]
    fn now_plain_date_time_iso_returns_instance() {
        let is_instance: bool =
            eval("Temporal.Now.plainDateTimeISO() instanceof Temporal.PlainDateTime");
        assert!(is_instance);
    }

    #[test]
    fn test_subsecond_getters() {
        let values: Vec<i32> = eval(
            r#"
            const time = new Temporal.PlainDateTime(2024,1,1,0,0,0,1,2,3);
            [time.millisecond,time.microsecond,time.nanosecond]
        "#,
        );

        assert_eq!(values[0], 1);
        assert_eq!(values[1], 2);
        assert_eq!(values[2], 3);
    }
}
