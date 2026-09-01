use std::{fmt::Debug, sync::Arc};

use iced::{
    Task,
    task::{self, Handle, Straw},
};

#[derive(Debug, Clone)]
pub enum QueryUpdate<D> {
    Finished(Result<D, Arc<anyhow::Error>>),
    Fetching(()),
}

#[derive(Debug)]
pub enum QueryState<D> {
    Idle,
    Fetching { _task: task::Handle },
    Finished(D),
    Error(Arc<anyhow::Error>),
}

pub struct Query<D> {
    pub state: QueryState<D>,
    pub id: String,
    fetcher: Box<dyn Fn() -> (Task<QueryUpdate<D>>, Handle)>,
}

impl<D: Send + 'static> Query<D> {
    pub fn new<S>(id: String, fetcher: impl Fn() -> S + 'static) -> Self
    where
        S: Straw<D, (), Arc<anyhow::Error>> + Send + 'static,
    {
        Self {
            id,
            state: QueryState::Idle,
            fetcher: Box::new(move || {
                Task::sip(fetcher(), QueryUpdate::Fetching, QueryUpdate::Finished).abortable()
            }),
        }
    }

    pub fn start(&mut self) -> Task<QueryUpdate<D>> {
        match self.state {
            QueryState::Idle | QueryState::Fetching { .. } | QueryState::Error(_) => {
                let (task, handle) = (self.fetcher)();

                self.state = QueryState::Fetching {
                    _task: handle.abort_on_drop(),
                };

                task
            }
            _ => Task::none(),
        }
    }

    pub fn update(&mut self, data: QueryUpdate<D>) {
        match data {
            QueryUpdate::Finished(result) => {
                self.state = match result {
                    Ok(d) => QueryState::Finished(d),
                    Err(err) => {
                        eprintln!("{:#?}", err);

                        QueryState::Error(err)
                    }
                };
            }
            QueryUpdate::Fetching(_) => {}
        }
    }
}

impl<D> Debug for Query<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
