use rquickjs::{Class, Ctx, JsLifetime, Object, class::Trace, function::Opt};

use super::duration::Duration;

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct PlainDateTime {
    #[qjs(skip_trace)]
    data: jiff::civil::DateTime,
}

impl PlainDateTime {
    pub fn new(data: jiff::civil::DateTime) -> Self {
        Self { data }
    }
}

#[rquickjs::methods]
impl<'js> PlainDateTime {
    #[qjs(static)]
    fn compare() {}

    #[qjs(static)]
    fn from(
        ctx: Ctx<'js>,
        info: rquickjs::Value<'js>,
        _opts: Opt<rquickjs::Object<'js>>,
    ) -> rquickjs::Result<rquickjs::Class<'js, PlainDateTime>> {
        if let Ok(v) = info.get::<String>() {
            let zdt = v
                .parse::<jiff::civil::DateTime>()
                .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?;

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
        _microsecond: Opt<i32>,
        _nanosecond: Opt<i32>,
    ) -> rquickjs::Result<Self> {
        let dt = jiff::civil::DateTime::new(
            year,
            month,
            day,
            hour.unwrap_or_default(),
            minute.unwrap_or_default(),
            second.unwrap_or_default(),
            millisecond.unwrap_or_default(),
        )
        .map_err(|e| rquickjs::Exception::throw_range(&ctx, &e.to_string()))?;

        Ok(Self { data: dt })
    }

    #[qjs(get)]
    fn day(&self) -> i8 {
        self.data.day()
    }

    fn until(
        &self,
        ctx: Ctx<'js>,
        other: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Class<'js, Duration>> {
        if let Ok(other) = other.get::<Class<'js, PlainDateTime>>() {
            let other = other.borrow();

            let dur = self.data.duration_until(other.data);

            return Class::instance(ctx, Duration::new(dur));
        }

        todo!()
    }
}
