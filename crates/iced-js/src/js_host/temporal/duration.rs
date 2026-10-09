use rquickjs::{JsLifetime, class::Trace};

#[derive(JsLifetime, Trace)]
#[rquickjs::class]
pub struct Duration {
    #[qjs(skip_trace)]
    data: jiff::SignedDuration,
}

impl Duration {
    pub fn new(data: jiff::SignedDuration) -> Self {
        Self { data }
    }
}

#[rquickjs::methods]
impl Duration {
    #[qjs(get)]
    fn days(&self) {
        todo!()
    }

    #[qjs(get)]
    fn hours(&self) -> i64 {
        self.data.as_hours()
    }

    #[qjs(get)]
    fn minutes(&self) -> i64 {
        self.data.as_mins()
    }
}
