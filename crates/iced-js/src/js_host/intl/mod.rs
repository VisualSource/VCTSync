use rquickjs::Ctx;

#[cfg(feature = "js-temporal")]
mod date_time_format;
#[cfg(feature = "js-temporal")]
mod relative_time_format;

pub fn init<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    let intl = rquickjs::Object::new_proto(ctx.clone(), None)?;

    #[cfg(feature = "js-temporal")]
    rquickjs::Class::<relative_time_format::RelativeTimeFormat>::define(&intl)?;
    #[cfg(feature = "js-temporal")]
    rquickjs::Class::<date_time_format::DateTimeFormat>::define(&intl)?;

    globals.set("Intl", intl)?;

    Ok(())
}
