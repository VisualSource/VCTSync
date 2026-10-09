use rquickjs::{Class, Object};

use crate::js_host::temporal::{now::Now, plain_date_time::PlainDateTime};

mod duration;
mod now;
mod plain_date_time;

pub fn init(ctx: &rquickjs::Ctx<'_>) -> rquickjs::Result<()> {
    let temporal = Object::new_proto(ctx.clone(), None)?;

    temporal.set("now", Now::new())?;

    Class::<PlainDateTime>::define(&temporal)?;
    Class::<duration::Duration>::define(&temporal)?;

    ctx.globals().set("Temporal", temporal)?;

    Ok(())
}
