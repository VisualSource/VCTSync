#[rquickjs::module]
pub mod compression {
    use std::io::Read;

    use rquickjs::Ctx;

    #[rquickjs::function]
    #[qjs(rename = "gzipDecodeSync")]
    pub fn gzip_decode_sync<'js>(
        ctx: Ctx<'js>,
        input: rquickjs::TypedArray<'js, u8>,
    ) -> rquickjs::Result<rquickjs::TypedArray<'js, u8>> {
        let buffer = input
            .as_raw()
            .ok_or_else(|| rquickjs::Exception::throw_type(&ctx, "buffer was detached"))?;

        // SAFETY: no JS runs while the slice is borrowed)
        let buffer_ref = unsafe { buffer.as_ref() };

        let mut decoder = libflate::gzip::MultiDecoder::new(buffer_ref)
            .map_err(|err| rquickjs::Exception::throw_message(&ctx, &err.to_string()))?;

        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(|err| rquickjs::Exception::throw_message(&ctx, &err.to_string()))?;

        rquickjs::TypedArray::new(ctx, out)
    }
}
