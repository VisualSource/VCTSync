use rquickjs::{Ctx, Result};

mod abort_contoller;
pub mod console;
mod fetch;
mod formatter;
pub mod iced_host;
pub mod timers;

pub fn init_browser_apis(ctx: &Ctx<'_>) -> Result<()> {
    abort_contoller::init(ctx)?;
    console::init(ctx)?;
    fetch::init(ctx)?;
    timers::init(ctx)?;

    let globals = ctx.globals();

    // declare window object
    globals.set("window", ctx.globals())?;

    Ok(())
}
