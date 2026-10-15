//! WHATWG `URL` and `URLSearchParams` globals, backed by the `url` crate.
use rquickjs::{
    Array, Class, Coerced, Ctx, FromJs, Function, IntoJs, JsLifetime, Result, Value,
    atom::PredefinedAtom,
    class::Trace,
    function::{Opt, This},
};
use url::{form_urlencoded, quirks};

/// The URL interface is used to parse, construct, normalize, and encode URLs.
#[allow(clippy::upper_case_acronyms)]
#[rquickjs::class]
#[derive(Trace, JsLifetime)]
pub struct URL<'js> {
    #[qjs(skip_trace)]
    url: url::Url,

    /// Created lazily by the `searchParams` getter, so a URL that never touches
    /// it doesn't pay for the back-reference.
    search_params: Option<Class<'js, URLSearchParams<'js>>>,
}

impl<'js> URL<'js> {
    pub fn new(url: url::Url) -> Self {
        Self {
            url,
            search_params: None,
        }
    }

    /// Re-reads the query into the linked `URLSearchParams` after the URL changed.
    fn sync_search_params(&self) {
        if let Some(params) = &self.search_params {
            params.borrow_mut().list = parse_query(self.url.query());
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> URL<'js> {
    #[qjs(constructor)]
    pub fn constructor(
        ctx: Ctx<'js>,
        input: Coerced<String>,
        base: Opt<Option<Coerced<String>>>,
    ) -> Result<Self> {
        let base = base.0.flatten();
        let url = parse_url(&input.0, base.as_ref().map(|b| b.0.as_str()))
            .ok_or_else(|| invalid_url(&ctx))?;
        Ok(Self::new(url))
    }

    /// Returns true if the url (relative to base, if given) is parsable and valid.
    #[qjs(static)]
    fn can_parse(input: Coerced<String>, base: Opt<Option<Coerced<String>>>) -> bool {
        let base = base.0.flatten();
        parse_url(&input.0, base.as_ref().map(|b| b.0.as_str())).is_some()
    }

    /// Like the constructor, but returns null instead of throwing on an invalid url.
    #[qjs(static)]
    fn parse(
        ctx: Ctx<'js>,
        input: Coerced<String>,
        base: Opt<Option<Coerced<String>>>,
    ) -> Result<Value<'js>> {
        let base = base.0.flatten();
        match parse_url(&input.0, base.as_ref().map(|b| b.0.as_str())) {
            Some(url) => Class::instance(ctx.clone(), Self::new(url))?.into_js(&ctx),
            None => Ok(Value::new_null(ctx)),
        }
    }

    #[qjs(get, rename = "href", enumerable)]
    fn href(&self) -> String {
        quirks::href(&self.url).to_owned()
    }

    #[qjs(set, rename = "href", enumerable)]
    fn set_href(&mut self, ctx: Ctx<'js>, value: Coerced<String>) -> Result<()> {
        quirks::set_href(&mut self.url, &value.0).map_err(|_| invalid_url(&ctx))?;
        self.sync_search_params();
        Ok(())
    }

    #[qjs(get, rename = "origin", enumerable)]
    fn origin(&self) -> String {
        quirks::origin(&self.url)
    }

    #[qjs(get, rename = "protocol", enumerable)]
    fn protocol(&self) -> String {
        quirks::protocol(&self.url).to_owned()
    }

    #[qjs(set, rename = "protocol", enumerable)]
    fn set_protocol(&mut self, value: Coerced<String>) {
        let _ = quirks::set_protocol(&mut self.url, &value.0);
    }

    #[qjs(get, rename = "username", enumerable)]
    fn username(&self) -> String {
        quirks::username(&self.url).to_owned()
    }

    #[qjs(set, rename = "username", enumerable)]
    fn set_username(&mut self, value: Coerced<String>) {
        let _ = quirks::set_username(&mut self.url, &value.0);
    }

    #[qjs(get, rename = "password", enumerable)]
    fn password(&self) -> String {
        quirks::password(&self.url).to_owned()
    }

    #[qjs(set, rename = "password", enumerable)]
    fn set_password(&mut self, value: Coerced<String>) {
        let _ = quirks::set_password(&mut self.url, &value.0);
    }

    #[qjs(get, rename = "host", enumerable)]
    fn host(&self) -> String {
        quirks::host(&self.url).to_owned()
    }

    #[qjs(set, rename = "host", enumerable)]
    fn set_host(&mut self, value: Coerced<String>) {
        let _ = quirks::set_host(&mut self.url, &value.0);
    }

    #[qjs(get, rename = "hostname", enumerable)]
    fn hostname(&self) -> String {
        quirks::hostname(&self.url).to_owned()
    }

    #[qjs(set, rename = "hostname", enumerable)]
    fn set_hostname(&mut self, value: Coerced<String>) {
        let _ = quirks::set_hostname(&mut self.url, &value.0);
    }

    #[qjs(get, rename = "port", enumerable)]
    fn port(&self) -> String {
        quirks::port(&self.url).to_owned()
    }

    #[qjs(set, rename = "port", enumerable)]
    fn set_port(&mut self, value: Coerced<String>) {
        let _ = quirks::set_port(&mut self.url, &value.0);
    }

    #[qjs(get, rename = "pathname", enumerable)]
    fn pathname(&self) -> String {
        quirks::pathname(&self.url).to_owned()
    }

    #[qjs(set, rename = "pathname", enumerable)]
    fn set_pathname(&mut self, value: Coerced<String>) {
        quirks::set_pathname(&mut self.url, &value.0);
    }

    #[qjs(get, rename = "search", enumerable)]
    fn search(&self) -> String {
        quirks::search(&self.url).to_owned()
    }

    #[qjs(set, rename = "search", enumerable)]
    fn set_search(&mut self, value: Coerced<String>) {
        quirks::set_search(&mut self.url, &value.0);
        self.sync_search_params();
    }

    #[qjs(get, rename = "hash", enumerable)]
    fn hash(&self) -> String {
        quirks::hash(&self.url).to_owned()
    }

    #[qjs(set, rename = "hash", enumerable)]
    fn set_hash(&mut self, value: Coerced<String>) {
        quirks::set_hash(&mut self.url, &value.0);
    }

    /// A URLSearchParams object linked to this URL; mutating it rewrites `search`.
    #[qjs(get, rename = "searchParams", enumerable)]
    fn search_params(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> Result<Class<'js, URLSearchParams<'js>>> {
        if let Some(params) = &this.borrow().search_params {
            return Ok(params.clone());
        }

        let list = parse_query(this.borrow().url.query());
        let params = Class::instance(ctx, URLSearchParams::new(list, Some(this.0.clone())))?;
        this.borrow_mut().search_params = Some(params.clone());

        Ok(params)
    }

    #[qjs(rename = "toString")]
    fn to_js_string(&self) -> String {
        self.href()
    }

    #[qjs(rename = "toJSON")]
    fn to_json(&self) -> String {
        self.href()
    }
}

/// The URLSearchParams interface defines utility methods to work with the query string of a URL.
#[allow(clippy::upper_case_acronyms)]
#[rquickjs::class]
#[derive(Trace, JsLifetime)]
pub struct URLSearchParams<'js> {
    #[qjs(skip_trace)]
    list: Vec<(String, String)>,

    /// The URL this was handed out by, kept in sync on every mutation.
    owner: Option<Class<'js, URL<'js>>>,
}

impl<'js> URLSearchParams<'js> {
    pub fn new(list: Vec<(String, String)>, owner: Option<Class<'js, URL<'js>>>) -> Self {
        Self { list, owner }
    }

    /// Writes the list back into the owning URL's query.
    fn update(&self) {
        if let Some(owner) = &self.owner {
            let query = serialize(&self.list);
            owner
                .borrow_mut()
                .url
                .set_query((!query.is_empty()).then_some(query.as_str()));
        }
    }

    fn list_from_init(ctx: &Ctx<'js>, init: Value<'js>) -> Result<Vec<(String, String)>> {
        if init.is_undefined() || init.is_null() {
            return Ok(Vec::new());
        }

        if let Some(obj) = init.as_object() {
            if let Ok(other) = Class::<Self>::from_js(ctx, init.clone()) {
                return Ok(other.borrow().list.clone());
            }

            let iterator: Option<Function> = obj.get(PredefinedAtom::SymbolIterator)?;
            if iterator.is_some() {
                let mut list = Vec::new();
                for pair in rquickjs::JsIterator::<Value>::from_js(ctx, init.clone())? {
                    let pair = Vec::<Coerced<String>>::from_js(ctx, pair?)?;
                    let [name, value] = <[Coerced<String>; 2]>::try_from(pair).map_err(|_| {
                        rquickjs::Exception::throw_type(
                            ctx,
                            "URLSearchParams init pairs must have exactly two items",
                        )
                    })?;
                    list.push((name.0, value.0));
                }
                return Ok(list);
            }

            return obj
                .props::<String, Coerced<String>>()
                .map(|prop| prop.map(|(name, value)| (name, value.0)))
                .collect();
        }

        let query = Coerced::<String>::from_js(ctx, init)?.0;
        Ok(parse_query(Some(query.strip_prefix('?').unwrap_or(&query))))
    }

    /// Builds an array iterator over a snapshot of the list.
    fn iter_snapshot(
        ctx: Ctx<'js>,
        items: impl IntoIterator<Item = Value<'js>>,
    ) -> Result<Value<'js>> {
        let array = Array::new(ctx)?;
        for (i, item) in items.into_iter().enumerate() {
            array.set(i, item)?;
        }
        let values: Function = array.as_object().get("values")?;
        values.call((This(array),))
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> URLSearchParams<'js> {
    #[qjs(constructor)]
    pub fn constructor(ctx: Ctx<'js>, init: Opt<Value<'js>>) -> Result<Self> {
        let list = match init.0 {
            Some(init) => Self::list_from_init(&ctx, init)?,
            None => Vec::new(),
        };
        Ok(Self::new(list, None))
    }

    #[qjs(get, enumerable)]
    fn size(&self) -> usize {
        self.list.len()
    }

    fn append(&mut self, name: Coerced<String>, value: Coerced<String>) {
        self.list.push((name.0, value.0));
        self.update();
    }

    fn delete(&mut self, name: Coerced<String>, value: Opt<Coerced<String>>) {
        let value = value.0.map(|v| v.0);
        self.list
            .retain(|(n, v)| !(*n == name.0 && value.as_ref().is_none_or(|value| v == value)));
        self.update();
    }

    fn get(&self, ctx: Ctx<'js>, name: Coerced<String>) -> Result<Value<'js>> {
        match self.list.iter().find(|(n, _)| *n == name.0) {
            Some((_, value)) => value.as_str().into_js(&ctx),
            None => Ok(Value::new_null(ctx)),
        }
    }

    fn get_all(&self, name: Coerced<String>) -> Vec<String> {
        self.list
            .iter()
            .filter(|(n, _)| *n == name.0)
            .map(|(_, v)| v.clone())
            .collect()
    }

    fn has(&self, name: Coerced<String>, value: Opt<Coerced<String>>) -> bool {
        let value = value.0.map(|v| v.0);
        self.list
            .iter()
            .any(|(n, v)| *n == name.0 && value.as_ref().is_none_or(|value| v == value))
    }

    /// Replaces the first pair named `name` and removes the rest, or appends if there is none.
    fn set(&mut self, name: Coerced<String>, value: Coerced<String>) {
        let mut found = false;
        self.list.retain_mut(|(n, v)| {
            if *n != name.0 {
                return true;
            }
            if found {
                return false;
            }
            found = true;
            *v = value.0.clone();
            true
        });
        if !found {
            self.list.push((name.0, value.0));
        }
        self.update();
    }

    /// Stable sort by name, comparing UTF-16 code units like the spec.
    fn sort(&mut self) {
        self.list
            .sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
        self.update();
    }

    fn for_each(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        callback: Function<'js>,
        this_arg: Opt<Value<'js>>,
    ) -> Result<()> {
        let this_arg = this_arg.0.unwrap_or_else(|| Value::new_undefined(ctx));
        // Snapshot so the callback can mutate the params without holding a borrow.
        let list = this.borrow().list.clone();
        for (name, value) in list {
            callback.call::<_, ()>((This(this_arg.clone()), value, name, this.0.clone()))?;
        }
        Ok(())
    }

    #[qjs(rename = PredefinedAtom::SymbolIterator)]
    fn iterator(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        self.entries(ctx)
    }

    fn entries(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let pairs = self
            .list
            .iter()
            .map(|(n, v)| {
                let pair = Array::new(ctx.clone())?;
                pair.set(0, n.as_str())?;
                pair.set(1, v.as_str())?;
                pair.into_js(&ctx)
            })
            .collect::<Result<Vec<_>>>()?;
        Self::iter_snapshot(ctx, pairs)
    }

    fn keys(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let names = self
            .list
            .iter()
            .map(|(n, _)| n.as_str().into_js(&ctx))
            .collect::<Result<Vec<_>>>()?;
        Self::iter_snapshot(ctx, names)
    }

    fn values(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let values = self
            .list
            .iter()
            .map(|(_, v)| v.as_str().into_js(&ctx))
            .collect::<Result<Vec<_>>>()?;
        Self::iter_snapshot(ctx, values)
    }

    #[qjs(rename = "toString")]
    fn to_js_string(&self) -> String {
        serialize(&self.list)
    }
}

fn parse_url(input: &str, base: Option<&str>) -> Option<url::Url> {
    match base {
        Some(base) => url::Url::parse(base).ok()?.join(input).ok(),
        None => url::Url::parse(input).ok(),
    }
}

fn parse_query(query: Option<&str>) -> Vec<(String, String)> {
    form_urlencoded::parse(query.unwrap_or_default().as_bytes())
        .into_owned()
        .collect()
}

fn serialize(list: &[(String, String)]) -> String {
    form_urlencoded::Serializer::new(String::new())
        .extend_pairs(list)
        .finish()
}

fn invalid_url(ctx: &Ctx<'_>) -> rquickjs::Error {
    rquickjs::Exception::throw_type(ctx, "Invalid URL")
}

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();

    Class::<URL>::define(&globals)?;
    Class::<URLSearchParams>::define(&globals)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module, promise::Promise};

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
    async fn parses_components() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const u = new URL("https://user:pw@a.com:8080/b/c?d=1#e");
                return [u.href, u.origin, u.protocol, u.username, u.password, u.host,
                        u.hostname, u.port, u.pathname, u.search, u.hash, String(u)];
            };
            "#,
        )
        .await;
        assert_eq!(
            result,
            [
                "https://user:pw@a.com:8080/b/c?d=1#e",
                "https://a.com:8080",
                "https:",
                "user",
                "pw",
                "a.com:8080",
                "a.com",
                "8080",
                "/b/c",
                "?d=1",
                "#e",
                "https://user:pw@a.com:8080/b/c?d=1#e",
            ]
        );
    }

    #[tokio::test]
    async fn resolves_relative_to_base() {
        let result: Vec<String> = eval(
            r#"
            export default async () => [
                new URL("../x", "https://a.com/b/c").href,
                new URL("https://z.com/", undefined).href,
            ];
            "#,
        )
        .await;
        assert_eq!(result, ["https://a.com/x", "https://z.com/"]);
    }

    #[tokio::test]
    async fn invalid_url_throws_type_error() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                let name = "no throw";
                try { new URL("not a url"); } catch (e) { name = e.constructor.name; }
                return [name, String(URL.canParse("not a url")), String(URL.parse("nope")),
                        String(URL.canParse("/x", "https://a.com"))];
            };
            "#,
        )
        .await;
        assert_eq!(result, ["TypeError", "false", "null", "true"]);
    }

    /// The exact checks `@tanstack/db` runs in `deepClone` / `deepEquals`.
    #[tokio::test]
    async fn instanceof_checks() {
        let result: Vec<bool> = eval(
            r#"
            export default async () => {
                const u = new URL("https://a.com/");
                const clone = new URL(u.href);
                return [({}) instanceof URL, clone instanceof URL, clone.href === u.href];
            };
            "#,
        )
        .await;
        assert_eq!(result, [false, true, true]);
    }

    #[tokio::test]
    async fn search_params_stay_linked() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const u = new URL("https://a.com/?a=1");
                const p = u.searchParams;
                p.append("b", "x y");
                const afterAppend = u.href;
                u.search = "?c=3";
                const afterSearch = [...p.keys()].join(",");
                p.delete("c");
                return [afterAppend, afterSearch, u.href, String(p === u.searchParams)];
            };
            "#,
        )
        .await;
        assert_eq!(
            result,
            ["https://a.com/?a=1&b=x+y", "c", "https://a.com/", "true",]
        );
    }

    #[tokio::test]
    async fn search_params_methods() {
        let result: Vec<String> = eval(
            r#"
            export default async () => {
                const p = new URLSearchParams("?b=2&a=1&b=3");
                const out = [p.get("b"), String(p.get("z")), p.getAll("b").join("|"),
                             String(p.has("a")), String(p.has("a", "2")), String(p.size)];
                p.set("b", "9");
                p.sort();
                out.push(p.toString());
                const seen = [];
                p.forEach((v, k) => seen.push(k + "=" + v));
                out.push(seen.join("&"));
                out.push(new URLSearchParams([["x", "1"]]).toString());
                out.push(new URLSearchParams({ y: "2" }).toString());
                out.push([...new URLSearchParams("k=v")].map(([k, v]) => k + v).join());
                return out;
            };
            "#,
        )
        .await;
        assert_eq!(
            result,
            [
                "2", "null", "2|3", "true", "false", "3", "a=1&b=9", "a=1&b=9", "x=1", "y=2", "kv"
            ]
        );
    }
}
