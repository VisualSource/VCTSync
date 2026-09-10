mod query;
mod query_client;
mod structs;

use iced_runtime::{Task, task};
use std::{any::Any, borrow::Cow, rc::Rc, sync::Arc};
use structs::QueryUpdate;

pub use query::Query;
pub use query_client::QueryClient;
pub use structs::QueryEvent;

pub type QueryKey = Cow<'static, str>;
pub type QueryData = Arc<dyn Any + Send + Sync>;
pub type QueryError = Arc<anyhow::Error>;
pub type QueryFetcher = Rc<dyn Fn() -> (Task<QueryUpdate>, task::Handle)>;
