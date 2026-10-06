use rquickjs::{Ctx, JsLifetime, U8Clamped, class::Trace};

#[derive(Trace, JsLifetime)]
#[rquickjs::class(frozen)]
pub struct Crypto {}

impl Crypto {
    pub fn new() -> Self {
        Self {}
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl Crypto {
    fn get_random_values<'js>(
        &mut self,
        ctx: Ctx<'js>,
        typed_array: rquickjs::Value<'js>,
    ) -> rquickjs::Result<rquickjs::Value<'js>> {
        if !typed_array.is_object() {
            return Err(rquickjs::Exception::throw_type(
                &ctx,
                "was expecting a typed array",
            ));
        }
        let ta = unsafe { typed_array.ref_object() };

        let is_int_view = ta.is_typed_array::<i8>()
            || ta.is_typed_array::<u8>()
            || ta.is_typed_array::<U8Clamped>()
            || ta.is_typed_array::<i16>()
            || ta.is_typed_array::<u16>()
            || ta.is_typed_array::<i32>()
            || ta.is_typed_array::<u32>()
            || ta.is_typed_array::<i64>()
            || ta.is_typed_array::<u64>();

        if !is_int_view {
            return Err(rquickjs::Exception::throw_type(
                &ctx,
                "expected an integer typed array",
            ));
        }

        let Some(mut raw) = unsafe { ta.ref_typed_array::<u8>() }.as_raw() else {
            return Err(rquickjs::Exception::throw_type(
                &ctx,
                "typed array is detached",
            ));
        };

        // should be valid  until JS runs again
        let bytes = unsafe { raw.as_mut() };
        if bytes.len() > 65_536 {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "byteLength exceeds 65536",
            ));
        }

        rand::fill(bytes);

        Ok(typed_array)
    }
}

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    globals.set("crypto", Crypto::new())?;

    Ok(())
}
