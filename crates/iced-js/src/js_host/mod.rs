use rquickjs::{Ctx, Result};

mod abort_controller;
pub mod console;
mod crypto;
mod event_target;
#[cfg(feature = "fetch")]
mod fetch;
#[cfg(feature = "js-temporal")]
mod temporal;

mod intl;

mod formatter;
pub mod iced;
pub mod iced_host;

pub mod timers;

pub fn init_browser_apis(ctx: &Ctx<'_>) -> Result<()> {
    abort_controller::init(ctx)?;
    console::init(ctx)?;
    crypto::init(ctx)?;
    timers::init(ctx)?;

    #[cfg(feature = "js-temporal")]
    temporal::init(ctx)?;

    #[cfg(feature = "fetch")]
    fetch::init(ctx)?;

    let globals = ctx.globals();

    // declare window object
    globals.set("window", ctx.globals())?;

    Ok(())
}
