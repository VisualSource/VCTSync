use rquickjs::{Function, JsLifetime, class::Trace};

#[derive(Clone, Trace, JsLifetime)]
pub struct Listeners<'js> {
    inner: Vec<ListenerTarget<'js>>,
}

impl<'js> Listeners<'js> {
    pub fn new() -> Self {
        Self {
            inner: Vec::default(),
        }
    }

    pub fn add(&mut self, target: String, callback: Function<'js>, once: bool) {
        self.inner.push(ListenerTarget::new(target, callback, once));
    }
    pub fn remove(&mut self, target: String, callback: Function<'js>) {
        let Some(idx) = self
            .inner
            .iter()
            .position(|x| x.target == target && x.callback.eq(&callback))
        else {
            return;
        };

        self.inner.swap_remove(idx);
    }

    pub fn remove_at(&mut self, idx: usize) {
        self.inner.swap_remove(idx);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ListenerTarget<'js>> {
        self.inner.iter()
    }
}

#[derive(Clone, Trace, JsLifetime)]
pub struct ListenerTarget<'js> {
    pub callback: Function<'js>,
    pub once: bool,
    pub target: String,
}

impl<'js> ListenerTarget<'js> {
    pub fn new(target: String, callback: Function<'js>, once: bool) -> Self {
        Self {
            target,
            callback,
            once,
        }
    }
}
