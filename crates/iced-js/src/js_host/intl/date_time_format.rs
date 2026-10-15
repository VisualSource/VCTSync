use jiff::{Timestamp, civil::DateTime, tz::TimeZone};
use rquickjs::{
    Class, Ctx, Function, JsLifetime, Object, Result, Value,
    class::Trace,
    function::{Opt, This},
};

use crate::js_host::temporal::plain_date_time::PlainDateTime;

// ponytail: en-US output only, the locale argument is ignored; needs a CLDR source (e.g. icu4x) for real locales.
#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct DateTimeFormat {
    /// A `jiff::fmt::strtime` pattern built once from the options.
    pattern: String,
    #[qjs(skip_trace)]
    tz: TimeZone,
}

#[rquickjs::methods]
impl<'js> DateTimeFormat {
    #[qjs(constructor)]
    fn constructor(
        ctx: Ctx<'js>,
        _locales: Opt<Value<'js>>,
        options: Opt<Object<'js>>,
    ) -> Result<Self> {
        let opt = |name: &str| {
            options
                .0
                .as_ref()
                .and_then(|o| o.get::<_, Option<String>>(name).ok().flatten())
        };

        let hour12 = options
            .0
            .as_ref()
            .and_then(|o| o.get::<_, Option<bool>>("hour12").ok().flatten())
            .unwrap_or(true);

        let tz = match opt("timeZone") {
            Some(name) => TimeZone::get(&name)
                .map_err(|err| rquickjs::Exception::throw_range(&ctx, &err.to_string()))?,
            None => TimeZone::system(),
        };

        let pattern = build_pattern(&opt, hour12)
            .map_err(|msg| rquickjs::Exception::throw_range(&ctx, &msg))?;

        Ok(Self { pattern, tz })
    }

    fn format(&self, ctx: Ctx<'js>, value: Opt<Value<'js>>) -> Result<String> {
        let dt = self.to_datetime(&ctx, value.0)?;
        Ok(dt.strftime(&self.pattern).to_string())
    }
}

impl DateTimeFormat {
    fn to_datetime<'js>(&self, ctx: &Ctx<'js>, value: Option<Value<'js>>) -> Result<DateTime> {
        let range = |msg: String| rquickjs::Exception::throw_range(ctx, &msg);
        let from_ms = |ms: f64| {
            Timestamp::from_millisecond(ms as i64)
                .map(|ts| ts.to_zoned(self.tz.clone()).datetime())
                .map_err(|err| range(err.to_string()))
        };

        let Some(value) = value.filter(|v| !v.is_undefined()) else {
            return Ok(Timestamp::now().to_zoned(self.tz.clone()).datetime());
        };

        if let Ok(pdt) = value.get::<Class<'js, PlainDateTime>>() {
            return Ok(pdt.borrow().data);
        }
        if let Some(n) = value.as_number() {
            return from_ms(n);
        }
        if let Some(s) = value.as_string() {
            let s = s.to_string()?;
            // Strings with an offset or `Z` are instants; plain ones are already wall-clock time.
            if let Ok(ts) = s.parse::<Timestamp>() {
                return Ok(ts.to_zoned(self.tz.clone()).datetime());
            }
            return s.parse::<DateTime>().map_err(|err| range(err.to_string()));
        }
        if let Some(obj) = value.as_object()
            && let Ok(get_time) = obj.get::<_, Function>("getTime")
        {
            return from_ms(get_time.call((This(obj.clone()),))?);
        }

        Err(rquickjs::Exception::throw_type(
            ctx,
            "expected a Date, number, string or Temporal.PlainDateTime",
        ))
    }
}

/// Maps an option value to its strftime directive, or errors on an unknown value.
fn pick(
    opt: &impl Fn(&str) -> Option<String>,
    name: &str,
    table: &[(&str, &'static str)],
) -> std::result::Result<Option<&'static str>, String> {
    let Some(value) = opt(name) else {
        return Ok(None);
    };
    table
        .iter()
        .find(|(k, _)| *k == value)
        .map(|(_, v)| Some(*v))
        .ok_or_else(|| format!("invalid {name} option: {value}"))
}

fn build_pattern(
    opt: &impl Fn(&str) -> Option<String>,
    hour12: bool,
) -> std::result::Result<String, String> {
    let date_style = pick(
        opt,
        "dateStyle",
        &[
            ("full", "%A, %B %-d, %Y"),
            ("long", "%B %-d, %Y"),
            ("medium", "%b %-d, %Y"),
            ("short", "%-m/%-d/%y"),
        ],
    )?;
    let time_style = pick(
        opt,
        "timeStyle",
        &[
            ("full", ":%S"),
            ("long", ":%S"),
            ("medium", ":%S"),
            ("short", ""),
        ],
    )?;

    if date_style.is_some() || time_style.is_some() {
        let time = time_style.map(|secs| {
            let (h, suffix) = if hour12 { ("%-I", " %p") } else { ("%H", "") };
            format!("{h}:%M{secs}{suffix}")
        });
        let sep = match opt("dateStyle").as_deref() {
            Some("full" | "long") => " at ",
            _ => ", ",
        };
        return Ok([date_style.map(str::to_string), time]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(sep));
    }

    let weekday = pick(
        opt,
        "weekday",
        &[("long", "%A"), ("short", "%a"), ("narrow", "%a")],
    )?;
    let mut year = pick(opt, "year", &[("numeric", "%Y"), ("2-digit", "%y")])?;
    let mut month = pick(
        opt,
        "month",
        &[
            ("numeric", "%-m"),
            ("2-digit", "%m"),
            ("long", "%B"),
            ("short", "%b"),
            ("narrow", "%b"),
        ],
    )?;
    let mut day = pick(opt, "day", &[("numeric", "%-d"), ("2-digit", "%d")])?;
    let hour = pick(
        opt,
        "hour",
        if hour12 {
            &[("numeric", "%-I"), ("2-digit", "%I")]
        } else {
            &[("numeric", "%H"), ("2-digit", "%H")]
        },
    )?;
    let minute = pick(opt, "minute", &[("numeric", "%M"), ("2-digit", "%M")])?;
    let second = pick(opt, "second", &[("numeric", "%S"), ("2-digit", "%S")])?;

    // Like Intl, no fields at all means a numeric date.
    if [weekday, year, month, day, hour, minute, second]
        .iter()
        .all(Option::is_none)
    {
        (year, month, day) = (Some("%Y"), Some("%-m"), Some("%-d"));
    }

    let textual_month = matches!(month, Some("%B" | "%b"));
    let date = if textual_month {
        // "October 15, 2026" / "October 2026" / "October 15"
        let mut s = month.unwrap().to_string();
        if let Some(d) = day {
            s += &format!(" {d}");
        }
        if let Some(y) = year {
            s += if day.is_some() { ", " } else { " " };
            s += y;
        }
        s
    } else {
        [month, day, year]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("/")
    };
    let date = match weekday {
        Some(w) if date.is_empty() => w.to_string(),
        Some(w) => format!("{w}, {date}"),
        None => date,
    };

    let mut time = [hour, minute, second]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(":");
    if hour12 && hour.is_some() {
        time += " %p";
    }

    Ok([date, time]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", "))
}

#[cfg(test)]
mod tests {
    use rquickjs::{CatchResultExt, Context, Runtime};

    fn eval(source: &str) -> String {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        ctx.with(|ctx| {
            crate::js_host::temporal::init(&ctx).unwrap();
            super::super::init(&ctx).unwrap();
            ctx.eval::<String, _>(source).catch(&ctx).unwrap()
        })
    }

    fn fmt(options: &str) -> String {
        eval(&format!(
            "new Intl.DateTimeFormat('en-US', {options}).format(new Temporal.PlainDateTime(2026, 10, 5, 15, 4, 9))"
        ))
    }

    #[test]
    fn formats() {
        assert_eq!(fmt("{}"), "10/5/2026");
        assert_eq!(fmt("{ dateStyle: 'full' }"), "Monday, October 5, 2026");
        assert_eq!(
            fmt("{ dateStyle: 'medium', timeStyle: 'short' }"),
            "Oct 5, 2026, 3:04 PM"
        );
        assert_eq!(
            fmt("{ dateStyle: 'long', timeStyle: 'medium' }"),
            "October 5, 2026 at 3:04:09 PM"
        );
        assert_eq!(fmt("{ timeStyle: 'short', hour12: false }"), "15:04");
        assert_eq!(fmt("{ month: 'short', day: 'numeric' }"), "Oct 5");
        assert_eq!(
            fmt("{ weekday: 'short', month: '2-digit', day: '2-digit', year: '2-digit' }"),
            "Mon, 10/05/26"
        );
        assert_eq!(fmt("{ hour: 'numeric', minute: '2-digit' }"), "3:04 PM");
        assert_eq!(
            eval(
                "new Intl.DateTimeFormat('en-US', { timeZone: 'UTC', dateStyle: 'short', timeStyle: 'short' }).format('2026-10-05T15:04:00Z')"
            ),
            "10/5/26, 3:04 PM"
        );
        assert_eq!(
            eval(
                "new Intl.DateTimeFormat('en-US', { timeZone: 'UTC', timeStyle: 'short' }).format(new Date(Date.UTC(2026, 9, 5, 9, 30)))"
            ),
            "9:30 AM"
        );
    }
}
