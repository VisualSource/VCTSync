use crate::{
    QueryKey,
    query_client::QueryClient,
    structs::{QueryEvent, QueryOptions, QueryState, QueryUpdate},
};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
    rc::Rc,
    sync::Arc,
};

use iced_runtime::{Task, task::Straw};

/// local query data accessor
pub struct Query<D> {
    pub key: QueryKey,
    pub snapshot: QueryState<Arc<D>>,
}

impl<D: Send + Sync + 'static> Query<D> {
    pub fn new<S>(
        client: &QueryClient,
        key: impl Into<QueryKey>,
        fetcher: impl Fn() -> S + 'static,
        opts: Option<QueryOptions>,
    ) -> Self
    where
        S: Straw<D, (), Arc<anyhow::Error>> + Send + 'static,
    {
        let key = key.into();

        let thunk = Rc::new(move || {
            Task::sip(
                fetcher(),
                |_| QueryUpdate::Fetching,
                |result| {
                    QueryUpdate::Finished(result.map(|d| Arc::new(d) as Arc<dyn Any + Send + Sync>))
                },
            )
            .abortable()
        });

        client.register(key.clone(), TypeId::of::<D>(), thunk, opts);
        let snapshot = client.snapshot::<D>(&key);

        Self { key, snapshot }
    }

    pub fn fetch(&self, client: &QueryClient) -> Task<QueryEvent> {
        client.ensure(&self.key)
    }

    pub fn sync(&mut self, query_client: &QueryClient) {
        self.snapshot = query_client.snapshot::<D>(&self.key);

        //TODO: should do a stale check here
    }
}

impl<D> Debug for Query<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Query({})", self.key)
    }
}
