use crate::{QueryData, QueryError, QueryFetcher, QueryKey};
use iced_runtime::task;
use std::{
    any::TypeId,
    fmt::Debug,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct QueryOptions {
    pub stale_time: Duration,
}

impl QueryOptions {
    pub fn with_stale_time(mut self, time: Duration) -> Self {
        self.stale_time = time;
        self
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            stale_time: Duration::from_mins(5),
        }
    }
}

pub struct Entry {
    pub data: Option<QueryData>,
    pub error: Option<QueryError>,
    pub type_id: TypeId,
    pub updated_at: Option<Instant>,
    pub status: QueryStatus,
    pub stale_time: Duration,
    pub handle: Option<task::Handle>,
    pub refetch: QueryFetcher,
    pub epoch: u64,
}

impl Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryEntry()",)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryStatus {
    Idle,
    Fetching,
    Success,
    Error,
}

#[derive(Debug)]
pub struct QueryState<D> {
    pub data: Option<D>,
    pub error: Option<QueryError>,
    pub status: QueryStatus,
}

impl<D> QueryState<D> {
    pub fn idle() -> Self {
        Self {
            data: None,
            error: None,
            status: QueryStatus::Idle,
        }
    }

    pub fn is_fetching(&self) -> bool {
        matches!(self.status, QueryStatus::Fetching)
    }
}

#[derive(Debug, Clone)]
pub enum QueryUpdate {
    Finished(Result<QueryData, QueryError>),
    Fetching,
}
#[derive(Debug, Clone)]
pub struct QueryEvent {
    pub key: QueryKey,
    pub state: QueryUpdate,
    pub epoch: u64,
}
