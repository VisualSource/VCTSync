use reqwest::header::{HeaderName, HeaderValue};
use rquickjs::{Ctx, JsLifetime, Result, Value, class::Trace, function::Opt};
use std::str::FromStr;

#[derive(Trace, JsLifetime)]
#[rquickjs::class(frozen)]
pub struct Headers {
    #[qjs(skip_trace)]
    pub map: reqwest::header::HeaderMap,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl Headers {
    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>, value: Opt<Value<'js>>) -> Result<Self> {
        let mut map = reqwest::header::HeaderMap::default();

        if let Some(init) = value.0 {
            match init.type_of() {
                rquickjs::Type::Array => {
                    let list = unsafe { init.ref_array() };

                    for item in list.iter::<rquickjs::Array<'js>>() {
                        let key_value_pair = item?;

                        let key = key_value_pair.get::<String>(0).map_err(|err| {
                            rquickjs::Error::new_from_js_message(
                                "unknown",
                                "String",
                                err.to_string(),
                            )
                        })?;
                        let value = key_value_pair.get::<String>(1).map_err(|err| {
                            rquickjs::Error::new_from_js_message(
                                "unknown",
                                "String",
                                err.to_string(),
                            )
                        })?;

                        let header_key = HeaderName::from_str(&key).map_err(|err| {
                            rquickjs::Exception::throw_type(&ctx, &err.to_string())
                        })?;
                        let header_value = HeaderValue::from_str(&value).map_err(|err| {
                            rquickjs::Exception::throw_type(&ctx, &err.to_string())
                        })?;

                        map.append(header_key, header_value);
                    }
                }

                rquickjs::Type::Object => {
                    let obj = unsafe { init.ref_object() };

                    if obj.instance_of::<Headers>() {
                        let header_init = obj.as_class::<Headers>().ok_or_else(|| {
                            rquickjs::Exception::throw_type(
                                &ctx,
                                "was expecting an Headers instance",
                            )
                        })?;

                        let inner = header_init.borrow();

                        map = inner.map.clone();
                    } else {
                        for header in obj.props::<String, String>() {
                            let (key, value) = header?;

                            let header_key = HeaderName::from_str(&key).map_err(|err| {
                                rquickjs::Exception::throw_type(&ctx, &err.to_string())
                            })?;
                            let header_value = HeaderValue::from_str(&value).map_err(|err| {
                                rquickjs::Exception::throw_type(&ctx, &err.to_string())
                            })?;

                            map.append(header_key, header_value);
                        }
                    }
                }

                _ => {
                    return Err(rquickjs::Exception::throw_type(
                        &ctx,
                        "Headers Constructor: Argument 1 could not be converted to any of: seq<seq<string>> or record<string,string>",
                    ));
                }
            }
        }

        Ok(Self { map })
    }

    fn append<'js>(&mut self, ctx: Ctx<'js>, key: String, value: String) -> Result<()> {
        let header_value = HeaderValue::from_str(&value).map_err(|err| {
            rquickjs::Error::new_from_js_message("string", "HeaderValue", err.to_string())
        })?;

        let header_name = HeaderName::from_str(&key).map_err(|err| {
            rquickjs::Error::new_into_js_message("string", "HeaderName", err.to_string())
        })?;

        self.map
            .try_append(header_name, header_value)
            .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?;

        Ok(())
    }
    fn delete(&mut self, key: String) {
        self.map.remove(key);
    }
    fn entries(&self) {}
    fn for_each<'js>(&self, ctx: Ctx<'js>, func: rquickjs::Function<'js>) -> Result<()> {
        for (key, value) in self.map.iter() {
            let key = key.to_string();
            let value = value
                .to_str()
                .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?
                .to_string();

            func.call::<(String, String), ()>((key, value))?;
        }

        Ok(())
    }

    fn get<'js>(&self, _key: String, ctx: Ctx<'js>) -> Value<'js> {
        rquickjs::Null.into_value(ctx)
    }
    fn get_set_cookie(&mut self) {}
    fn has(&self, key: String) -> bool {
        self.map.contains_key(&key)
    }
    fn keys(&self) {}
    fn set<'js>(&mut self, ctx: Ctx<'js>, key: String, value: String) -> rquickjs::Result<()> {
        let header_value = HeaderValue::from_str(&value).map_err(|err| {
            rquickjs::Error::new_from_js_message("string", "HeaderValue", err.to_string())
        })?;

        let header_name = HeaderName::from_str(&key).map_err(|err| {
            rquickjs::Error::new_into_js_message("string", "HeaderName", err.to_string())
        })?;

        self.map
            .try_insert(header_name, header_value)
            .map_err(|err| rquickjs::Exception::throw_internal(&ctx, &err.to_string()))?;

        Ok(())
    }

    fn values(&self) {}
}
