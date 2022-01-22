use crate::{ws, SafeClients, SafeSessions};
use futures::Future;
use log::info;
use std::sync::Arc;
use warp::hyper::StatusCode;
use warp::Rejection;
use warp::Reply;

pub type Result<T> = std::result::Result<T, Rejection>;
/// An Rejection Class for new clients trying to use currently online ID's
#[derive(Debug)]
pub struct IDAlreadyTaken;
impl warp::reject::Reject for IDAlreadyTaken {}

/// Will handle a Client attempting to connect a websocket with the server
/// A User Requesting to be connected to an already connected ID will be rejected
pub async fn ws_handler<T, Fut>(
    ws: warp::ws::Ws,
    id: String,
    clients: SafeClients,
    sessions: SafeSessions<T>,
    event_handler: Arc<fn(String, String, SafeClients, SafeSessions<T>) -> Fut>,
) -> Result<impl Reply>
where
    T: Clone + Send + Sync + 'static,
    Fut: Future + Send + 'static,
{
    let client_exists = clients.read().await.get(&id).is_none();
    match client_exists {
        false => {
            log::warn!("duplicate connection request for id: {}", id);
            Err(warp::reject::custom(IDAlreadyTaken))
        }
        true => Ok(ws.on_upgrade(move |socket| {
            log::info!("incoming request for id: {}", id);
            ws::client_connection(socket, id, clients, sessions, event_handler)
        })),
    }
}

/// Health Check Endpoint used to verify the service is live
pub async fn health_handler() -> Result<impl Reply> {
    info!("HEALTH_CHECK ✓");
    Ok(warp::reply::with_status("health check ✓", StatusCode::OK))
}
