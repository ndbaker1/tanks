use futures::Future;
use serde::Serialize;
use sessions::session_types::Client;
use sessions::session_types::{Clients, Sessions};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::filters::BoxedFilter;
use warp::ws::Message;
use warp::{Filter, Reply};

mod handler;
mod ws;

type SafeResource<T> = Arc<RwLock<T>>;

pub type SafeClients = SafeResource<Clients>;
pub type SafeSessions<T> = SafeResource<Sessions<T>>;

pub struct ServerConfig<T, C1, C2, F1, F2>
where
    T: Clone,
    C1: Fn(String, String, SafeClients, SafeSessions<T>) -> F1 + Send + Sync + 'static,
    C2: Fn(SafeClients, SafeSessions<T>) -> F2 + Send + Sync + 'static,
    F1: Future<Output = ()> + Send + Sync + 'static,
    F2: Future<Output = ()> + Send + Sync + 'static,
{
    pub event_handler: C1,
    pub tick_handler: C2,
    _d: PhantomData<T>,
}

impl<T, C1, C2, F1, F2> ServerConfig<T, C1, C2, F1, F2>
where
    T: Clone,
    C1: Fn(String, String, SafeClients, SafeSessions<T>) -> F1 + Send + Sync + 'static,
    C2: Fn(SafeClients, SafeSessions<T>) -> F2 + Send + Sync + 'static,
    F1: Future<Output = ()> + Send + Sync + 'static,
    F2: Future<Output = ()> + Send + Sync + 'static,
{
    pub fn from(event_handler: C1, tick_handler: C2) -> Self {
        Self {
            event_handler,
            tick_handler,
            _d: PhantomData,
        }
    }
}

/// Composite backend and frontend routes for the entire server
pub fn server<T, C1, C2, F1, F2>(
    server_config: ServerConfig<T, C1, C2, F1, F2>,
) -> BoxedFilter<(impl Reply,)>
where
    T: Clone + Send + Sync + 'static,
    C1: Fn(String, String, SafeClients, SafeSessions<T>) -> F1 + Send + Sync + 'static,
    C2: Fn(SafeClients, SafeSessions<T>) -> F2 + Send + Sync + 'static,
    F1: Future<Output = ()> + Send + Sync + 'static,
    F2: Future<Output = ()> + Send + Sync + 'static,
{
    warp::path("api")
        .and(backend(server_config))
        .or(frontend())
        .boxed()
}

/// Routes handling server requests and connections
fn backend<T, C1, C2, F1, F2>(config: ServerConfig<T, C1, C2, F1, F2>) -> BoxedFilter<(impl Reply,)>
where
    T: Clone + Send + Sync + 'static,
    C1: Fn(String, String, SafeClients, SafeSessions<T>) -> F1 + Send + Sync + 'static,
    C2: Fn(SafeClients, SafeSessions<T>) -> F2 + Send + Sync + 'static,
    F1: Future<Output = ()> + Send + Sync + 'static,
    F2: Future<Output = ()> + Send + Sync + 'static,
{
    let clients: SafeClients = Arc::new(RwLock::new(HashMap::new()));
    let sessions: SafeSessions<T> = Arc::new(RwLock::new(HashMap::new()));
    let health = warp::path!("health").and_then(handler::health_handler);

    let event_handler = Arc::new(config.event_handler);

    let (clients1, sessions1) = (clients.clone(), sessions.clone());
    let socket = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        // pass copies of our references for the client and sessions maps to our handler
        .and(warp::any().map(move || clients1.clone()))
        .and(warp::any().map(move || sessions1.clone()))
        .and(warp::any().map(move || event_handler.clone()))
        .and_then(handler::ws_handler);

    let (clients2, sessions2) = (clients.clone(), sessions.clone());
    let tick_handler = Arc::new(config.tick_handler);
    tokio::spawn(async move {
        log::info!("running server tick");
        tick_handler(clients2.clone(), sessions2.clone()).await
    });

    health.or(socket).boxed()
}

/// Routes for serving static website files
fn frontend() -> BoxedFilter<(impl Reply,)> {
    warp::fs::dir("dist").boxed()
}

/// Send an update to a single client
pub fn notify_client<T>(client: &Client, message: &T)
where
    T: Serialize,
{
    let sender = match &client.sender {
        Some(s) => s,
        None => {
            return log::error!(
                "sender was lost for clienT: Send + Sync + 'static {}",
                client.id
            )
        }
    };
    if let Err(e) = sender.send(Ok(Message::text(serde_json::to_string(message).unwrap()))) {
        log::error!("failed to send message to {} with err: {}", client.id, e,);
    }
}

/// Remove a sessions and the possible game state that accompanies it
pub async fn cleanup_session<T>(session_id: &str, sessions: &SafeSessions<T>) {
    // remove session
    sessions.write().await.remove(session_id);
    // log status
    log::info!("removed empty session");
    log::info!("sessions live: {}", sessions.read().await.len());
}
