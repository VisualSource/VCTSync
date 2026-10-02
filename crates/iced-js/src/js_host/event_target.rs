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

    pub fn take(&mut self, target: &str) -> Vec<Function<'js>> {
        let mut callbacks = Vec::default();

        self.inner.retain(|x| {
            if x.target != target {
                return true;
            }

            callbacks.push(x.callback.clone());

            !x.once
        });

        callbacks
    }

    pub fn add<T>(&mut self, target: T, callback: Function<'js>, once: bool)
    where
        T: Into<String>,
    {
        self.inner
            .push(ListenerTarget::new(target.into(), callback, once));
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
