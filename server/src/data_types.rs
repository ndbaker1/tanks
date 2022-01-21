use sessions::session_types::{Clients, Sessions};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SafeResource<T> = Arc<RwLock<T>>;

pub type SafeClients = SafeResource<Clients>;
pub type SafeSessions<T> = SafeResource<Sessions<T>>;
