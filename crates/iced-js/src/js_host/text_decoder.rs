//! WHATWG `TextDecoder` global. Only the UTF-8 encoding is supported.
use rquickjs::{
    ArrayBuffer, Class, Coerced, Ctx, JsLifetime, Object, Result, Value, class::Trace,
    function::Opt,
};

const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Decodes a stream of UTF-8 bytes into strings.
#[rquickjs::class]
#[derive(Trace, JsLifetime)]
pub struct TextDecoder {
    #[qjs(get)]
    fatal: bool,
    #[qjs(get, rename = "ignoreBOM")]
    ignore_bom: bool,

    /// Set once the start of the stream has been checked for a BOM.
    #[qjs(skip_trace)]
    bom_seen: bool,
    /// Trailing bytes of an incomplete sequence, held back by `{ stream: true }`.
    #[qjs(skip_trace)]
    pending: Vec<u8>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl TextDecoder {
    #[qjs(constructor)]
    pub fn constructor(
        ctx: Ctx<'_>,
        label: Opt<Coerced<String>>,
        options: Opt<Option<Object<'_>>>,
    ) -> Result<Self> {
        if let Some(label) = label.0
            && !is_utf8_label(&label.0)
        {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                &format!("The \"{}\" encoding is not supported", label.0),
            ));
        }

        let options = options.0.flatten();

        Ok(Self {
            fatal: get_flag(options.as_ref(), "fatal")?,
            ignore_bom: get_flag(options.as_ref(), "ignoreBOM")?,
            bom_seen: false,
            pending: Vec::new(),
        })
    }

    #[qjs(get)]
    fn encoding(&self) -> &'static str {
        "utf-8"
    }

    fn decode<'js>(
        &mut self,
        ctx: Ctx<'js>,
        input: Opt<Value<'js>>,
        options: Opt<Option<Object<'js>>>,
    ) -> Result<String> {
        let stream = get_flag(options.0.flatten().as_ref(), "stream")?;

        let mut bytes = std::mem::take(&mut self.pending);
        if let Some(input) = input.0 {
            bytes.extend_from_slice(&buffer_source_bytes(&ctx, &input)?);
        }

        let mut body = &bytes[..];
        if !self.ignore_bom && !self.bom_seen {
            if let Some(rest) = body.strip_prefix(BOM) {
                body = rest;
                self.bom_seen = true;
            } else if stream && BOM.starts_with(body) {
                // could still become a BOM once more bytes arrive
                self.pending = bytes;
                return Ok(String::new());
            } else {
                self.bom_seen = true;
            }
        }

        let split = if stream {
            incomplete_tail_start(body)
        } else {
            body.len()
        };
        let (complete, tail) = body.split_at(split);

        let text = if self.fatal {
            match std::str::from_utf8(complete) {
                Ok(text) => text.to_owned(),
                Err(_) => {
                    self.bom_seen = false;
                    return Err(rquickjs::Exception::throw_type(
                        &ctx,
                        "The encoded data was not valid utf-8",
                    ));
                }
            }
        } else {
            String::from_utf8_lossy(complete).into_owned()
        };

        self.pending = tail.to_vec();
        if !stream {
            self.bom_seen = false;
        }

        Ok(text)
    }
}

fn is_utf8_label(label: &str) -> bool {
    matches!(
        label
            .trim_matches(|c: char| c.is_ascii_whitespace())
            .to_ascii_lowercase()
            .as_str(),
        "unicode-1-1-utf-8"
            | "unicode11utf8"
            | "unicode20utf8"
            | "utf-8"
            | "utf8"
            | "x-unicode20utf8"
    )
}

fn get_flag(options: Option<&Object<'_>>, key: &str) -> Result<bool> {
    let Some(options) = options else {
        return Ok(false);
    };
    Ok(options
        .get::<_, Option<Coerced<bool>>>(key)?
        .is_some_and(|flag| flag.0))
}

/// Copies the bytes out of an `ArrayBuffer` or any view over one (typed arrays, `DataView`).
fn buffer_source_bytes<'js>(ctx: &Ctx<'js>, input: &Value<'js>) -> Result<Vec<u8>> {
    if input.is_undefined() {
        return Ok(Vec::new());
    }

    let not_buffer_source = || {
        rquickjs::Exception::throw_type(
            ctx,
            "The \"input\" argument must be an ArrayBuffer or ArrayBufferView",
        )
    };

    if let Some(buffer) = ArrayBuffer::from_value(input.clone()) {
        // SAFETY: the slice is copied before any JS runs
        return Ok(unsafe { buffer.as_bytes() }.unwrap_or_default().to_vec());
    }

    let view = input.as_object().ok_or_else(not_buffer_source)?;
    let buffer = view
        .get::<_, Value>("buffer")
        .ok()
        .and_then(ArrayBuffer::from_value)
        .ok_or_else(not_buffer_source)?;
    let offset: usize = view.get("byteOffset")?;
    let length: usize = view.get("byteLength")?;

    // SAFETY: the slice is copied before any JS runs
    let bytes = unsafe { buffer.as_bytes() }.unwrap_or_default();
    Ok(bytes
        .get(offset..offset + length)
        .unwrap_or_default()
        .to_vec())
}

/// Index where a trailing, not yet complete UTF-8 sequence starts, or `bytes.len()` if
/// the input doesn't end mid-sequence.
fn incomplete_tail_start(bytes: &[u8]) -> usize {
    let len = bytes.len();

    for back in 1..=len.min(3) {
        let start = len - back;
        let needed = match bytes[start] {
            0x80..=0xBF => continue, // continuation byte, keep looking for the lead
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF4 => 4,
            _ => return len,
        };

        let incomplete = back < needed
            && matches!(std::str::from_utf8(&bytes[start..]), Err(err) if err.error_len().is_none());
        return if incomplete { start } else { len };
    }

    len
}

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();

    Class::<TextDecoder>::define(&globals)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rquickjs::{
        AsyncContext, AsyncRuntime, CatchResultExt, Function, Module, promise::Promise,
    };

    use super::*;

    async fn eval<T>(source: &str) -> T
    where
        T: for<'js> rquickjs::FromJs<'js> + Send + 'static,
    {
        let rt = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&rt).await.unwrap();

        ctx.async_with(async |ctx| {
            init(&ctx).unwrap();

            let (module, promise) = Module::declare(ctx.clone(), "test", source)
                .catch(&ctx)
                .unwrap()
                .eval()
                .catch(&ctx)
                .unwrap();
            promise.into_future::<()>().await.catch(&ctx).unwrap();

            let test: Function = module.get("default").catch(&ctx).unwrap();
            test.call::<_, Promise>(())
                .catch(&ctx)
                .unwrap()
                .into_future::<T>()
                .await
                .catch(&ctx)
                .unwrap()
        })
        .await
    }

    #[tokio::test]
    async fn decodes_utf8() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const d = new TextDecoder();
                const bytes = new Uint8Array([0x68, 0x69, 0x20, 0xC3, 0xA9, 0x20, 0xF0, 0x9F, 0x98, 0x80]);
                return [d.decode(bytes), d.decode(bytes.buffer), d.decode(), d.encoding];
            };
            "#,
        )
        .await;
        assert_eq!(result, ["hi é 😀", "hi é 😀", "", "utf-8"]);
    }

    #[tokio::test]
    async fn respects_view_offsets() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const bytes = new Uint8Array([0x61, 0x62, 0x63, 0x64, 0x65]);
                const d = new TextDecoder();
                return [d.decode(bytes.subarray(1, 4)), d.decode(new DataView(bytes.buffer, 3))];
            };
            "#,
        )
        .await;
        assert_eq!(result, ["bcd", "de"]);
    }

    #[tokio::test]
    async fn strips_bom_unless_ignored() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const bytes = new Uint8Array([0xEF, 0xBB, 0xBF, 0x61]);
                return [
                    new TextDecoder().decode(bytes),
                    new TextDecoder("utf-8", { ignoreBOM: true }).decode(bytes),
                ];
            };
            "#,
        )
        .await;
        assert_eq!(result, ["a", "\u{FEFF}a"]);
    }

    #[tokio::test]
    async fn replaces_or_throws_on_invalid() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const bytes = new Uint8Array([0x61, 0xFF, 0x62]);
                let fatal;
                try {
                    new TextDecoder("utf-8", { fatal: true }).decode(bytes);
                    fatal = "no error";
                } catch (e) {
                    fatal = e.constructor.name;
                }
                return [new TextDecoder().decode(bytes), fatal];
            };
            "#,
        )
        .await;
        assert_eq!(result, ["a\u{FFFD}b", "TypeError"]);
    }

    #[tokio::test]
    async fn streams_split_sequences() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const d = new TextDecoder();
                const parts = [
                    d.decode(new Uint8Array([0xEF, 0xBB]), { stream: true }),
                    d.decode(new Uint8Array([0xBF, 0x61, 0xF0, 0x9F]), { stream: true }),
                    d.decode(new Uint8Array([0x98, 0x80, 0xC3]), { stream: true }),
                    d.decode(),
                ];
                return parts;
            };
            "#,
        )
        .await;
        assert_eq!(result, ["", "a", "😀", "\u{FFFD}"]);
    }

    #[tokio::test]
    async fn rejects_other_encodings() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                let error;
                try { new TextDecoder("latin1"); } catch (e) { error = e.constructor.name; }
                return [new TextDecoder(" UTF8 ").encoding, error];
            };
            "#,
        )
        .await;
        assert_eq!(result, ["utf-8", "RangeError"]);
    }
}
