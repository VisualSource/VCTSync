use crate::{
    QueryFetcher, QueryKey,
    structs::{Entry, QueryEvent, QueryOptions, QueryState, QueryStatus, QueryUpdate},
};
use iced::Task;
use std::{any::TypeId, cell::RefCell, collections::HashMap, fmt::Debug, sync::Arc, time::Instant};

#[derive(Debug)]
pub struct QueryClient {
    cache: RefCell<HashMap<QueryKey, Entry>>,
    options: QueryOptions,
}

impl QueryClient {
    pub fn with_options(opts: QueryOptions) -> Self {
        Self {
            cache: RefCell::new(HashMap::new()),
            options: opts,
        }
    }
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(HashMap::new()),
            options: QueryOptions::default(),
        }
    }

    pub fn register(
        &self,
        key: QueryKey,
        type_id: TypeId,
        fetcher: QueryFetcher,
        opts: Option<QueryOptions>,
    ) {
        let mut cache = self.cache.borrow_mut();

        let opts = opts.as_ref().unwrap_or_else(|| &self.options);

        if let Some(value) = cache.get_mut(&key) {
            debug_assert_eq!(value.type_id, type_id);
            value.refetch = fetcher;
            return;
        }

        cache.insert(
            key,
            Entry {
                data: None,
                error: None,
                type_id,
                updated_at: None,
                status: QueryStatus::Idle,
                stale_time: opts.stale_time.clone(),
                handle: None,
                refetch: fetcher,
                epoch: 0,
            },
        );
    }

    pub fn snapshot<D: Send + Sync + 'static>(&self, key: &QueryKey) -> QueryState<Arc<D>> {
        let cache = self.cache.borrow();

        let Some(entry) = cache.get(key) else {
            return QueryState::idle();
        };

        debug_assert_eq!(entry.type_id, TypeId::of::<D>());
        let content = if let Some(data) = entry.data.clone() {
            if let Ok(inner) = data.downcast::<D>() {
                Some(inner)
            } else {
                None
            }
        } else {
            None
        };

        QueryState {
            data: content,
            error: entry.error.clone(),
            status: entry.status.clone(),
        }
    }

    pub fn ensure(&self, key: &QueryKey) -> Task<QueryEvent> {
        let mut cache = self.cache.borrow_mut();
        let Some(entry) = cache.get_mut(key) else {
            return Task::none();
        };

        if matches!(entry.status, QueryStatus::Fetching) {
            return Task::none();
        }

        let is_stale = entry
            .updated_at
            .as_ref()
            .map(|ua| ua.elapsed() > entry.stale_time)
            .unwrap_or(true);

        if entry.data.is_none() || is_stale {
            let (task, handle) = (*entry.refetch)();
            entry.handle = Some(handle);
            entry.status = QueryStatus::Fetching;
            entry.epoch += 1;

            let epoch = entry.epoch;
            let key = key.clone();
            return task.map(move |state| QueryEvent {
                key: key.clone(),
                state,
                epoch,
            });
        }

        Task::none()
    }
    pub fn get<D: Send + Sync + 'static>(&self, key: &QueryKey) -> Option<Arc<D>> {
        if let Some(value) = self.cache.borrow().get(key) {
            debug_assert_eq!(TypeId::of::<D>(), value.type_id);

            if let Some(data) = value.data.clone() {
                if let Ok(inner) = data.downcast::<D>() {
                    return Some(inner);
                }
            }
        }

        None
    }
    pub fn status(&self, key: &QueryKey) -> QueryStatus {
        let cache = self.cache.borrow();

        if let Some(value) = cache.get(key) {
            return value.status.clone();
        }

        QueryStatus::Idle
    }
    pub fn receive(&self, update: QueryEvent) {
        let mut cache = self.cache.borrow_mut();
        let Some(entry) = cache.get_mut(&update.key) else {
            return;
        };

        if entry.epoch != update.epoch {
            return;
        }

        match update.state {
            QueryUpdate::Finished(result) => {
                entry.handle = None;
                match result {
                    Ok(value) => {
                        entry.status = QueryStatus::Success;
                        entry.updated_at = Some(Instant::now());
                        entry.data = Some(value);
                        entry.error = None;
                    }
                    Err(err) => {
                        entry.status = QueryStatus::Error;
                        entry.error = Some(err);
                    }
                }
            }
            QueryUpdate::Fetching => {
                entry.status = QueryStatus::Fetching;
            }
        }
    }
    pub fn invalidate(&self, key: &QueryKey) -> Task<QueryEvent> {
        let mut cache = self.cache.borrow_mut();
        let Some(entry) = cache.get_mut(key) else {
            return Task::none();
        };

        if let Some(handle) = &entry.handle {
            if !handle.is_aborted() {
                handle.abort();
            }
        }

        let (task, handle) = (*entry.refetch)();
        entry.status = QueryStatus::Fetching;
        entry.handle = Some(handle);
        entry.epoch += 1;

        let epoch = entry.epoch;
        let key = key.clone();
        task.map(move |state| QueryEvent {
            key: key.clone(),
            state,
            epoch,
        })
    }
    pub fn set<D: Send + Sync + 'static>(&self, key: QueryKey, data: D) {
        let mut cache = self.cache.borrow_mut();
        let Some(entry) = cache.get_mut(&key) else {
            return;
        };

        debug_assert_eq!(TypeId::of::<D>(), entry.type_id);

        entry.data = Some(Arc::new(data));
        entry.status = QueryStatus::Success;
        entry.error = None;
        entry.updated_at = Some(Instant::now());
    }
}
