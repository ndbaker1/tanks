use crate::{cleanup_session, SafeClients, SafeSessions};
use futures::{Future, FutureExt, StreamExt};
use sessions::session_types::{self, Client};
use std::sync::Arc;
use tokio::sync::mpsc::{self};
use urlencoding::decode;
use warp::ws::WebSocket;

/// The Initial Setup for a WebSocket Connection
pub async fn client_connection<T, F, Fut>(
    ws: WebSocket,
    connection_id: String,
    clients: SafeClients,
    sessions: SafeSessions<T>,
    event_handler: Arc<F>,
) where
    T: 'static + Clone,
    F: Fn(String, String, SafeClients, SafeSessions<T>) -> Fut,
    Fut: Future<Output = ()>,
{
    // Decode the strings coming in over URL parameters so we dont get things like '%20'
    // for spaces in our clients map
    let id = decode(&connection_id).expect("UTF-8").to_string();
    //======================================================
    // Splits the WebSocket into a Sink + Stream:
    // Sink - Pools the messages to get send to the client
    // Stream - receiver of messages from the client
    //======================================================
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    //======================================================
    // Gets an Unbounced Channel that can transport messages
    // between asynchronous tasks:
    // Sender - front end of the channel
    // Receiver - recieves the sender messages
    //======================================================
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    //======================================================
    // Spawn a thread to forward messages
    // from our channel into our WebSocket Sink
    // between asynchronous tasks using the same Client object
    //======================================================
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            log::error!("failed to send websocket msg: {}", e);
        }
    }));
    //======================================================
    // From now on we can use our client_sender.send(val: T)
    // to send messages to a given client websocket
    //======================================================

    //======================================================
    // Create a new Client and insert them into the Map
    //======================================================
    clients.write().await.insert(
        id.clone(),
        session_types::Client {
            id: id.clone(),
            sender: Some(client_sender),
            session_id: get_client_session_id(&id, &sessions).await,
        },
    );

    if let Some(client) = clients.read().await.get(&id) {
        handle_client_connect(&client, &sessions).await;
    }
    //======================================================
    // Synchronously wait for messages from the
    // Client Receiver Stream until an error occurs
    //======================================================
    while let Some(result) = client_ws_rcv.next().await {
        // Check that there was no error actually obtaining the Message
        match result {
            Ok(msg) => {
                //======================================================
                // Ensure the Message Parses to String
                //======================================================
                let message = match msg.to_str() {
                    Ok(v) => v.clone(),
                    Err(_) => {
                        log::warn!("websocket message: '{:?}' was not handled", msg);
                        log::warn!("disconnecting client <{}>", id);
                        if let Some(client) = clients.write().await.remove(&id) {
                            handle_client_disconnect(&client, &sessions).await;
                        }
                        return;
                    }
                };

                match message {
                    //======================================================
                    // ignore pings
                    //======================================================
                    "ping" | "ping\n" => {
                        log::info!("ignoring ping...");
                    }
                    //======================================================
                    // Game Session Related Events
                    //======================================================
                    _ => {
                        event_handler(
                            id.clone(),
                            message.to_string(),
                            clients.clone(),
                            sessions.clone(),
                        )
                        .await;
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "failed to recieve websocket message for id: <{}>, error: {}",
                    id,
                    e,
                );
            }
        }
    }
    //======================================================
    // Remove the Client from the Map
    // when they are finished using the socket (or error)
    //======================================================
    if let Some(client) = clients.write().await.remove(&id) {
        handle_client_disconnect(&client, &sessions).await;
    }
}

/// If a client exists in a session, then set their status to inactive.
///
/// If setting inactive status would leave no other active member, remove the session
async fn handle_client_disconnect<T>(client: &Client, sessions: &SafeSessions<T>) {
    log::info!("client <{}> disconnected", client.id);
    if let Some(session_id) = &client.session_id {
        let mut session_empty = false;
        // remove the client from the session and check if the session become empty
        if let Some(session) = sessions.write().await.get_mut(session_id) {
            if let Err(msg) = session.set_client_active_status(&client.id, false) {
                log::error!("{}", msg);
            }

            session_empty = session.get_clients_with_active_status(true).is_empty();
        }
        // remove the session if empty
        if session_empty {
            cleanup_session(session_id, sessions).await;
        }
    }
}

/// If a client exists in a session, then set their status to active
async fn handle_client_connect<T>(client: &Client, sessions: &SafeSessions<T>) {
    log::info!("{} connected", client.id);
    if let Some(session_id) = &client.session_id {
        if let Some(session) = sessions.write().await.get_mut(session_id) {
            if let Err(msg) = session.set_client_active_status(&client.id, true) {
                log::error!("{}", msg);
            }
        }
    }
}

/// Gets the SessionID of a client if it exists
async fn get_client_session_id<T>(client_id: &str, sessions: &SafeSessions<T>) -> Option<String> {
    for session in sessions.read().await.values() {
        if session.contains_client(client_id) {
            return Some(session.id.clone());
        }
    }
    return None;
}
