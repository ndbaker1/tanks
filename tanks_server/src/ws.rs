use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, WebSocketUpgrade,
    },
    response::IntoResponse,
    Extension,
};
use futures::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use serde::Deserialize;
use tanks_core::common::player::Player;
use tanks_events::ClientEvent;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    state::{generate_session_id, Client, Session, SharedServerState},
    SessionData,
};

#[derive(Deserialize)]
pub struct ClientConnectionParams {
    id: String,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(ClientConnectionParams { id: connection_id }): Query<ClientConnectionParams>,
    Extension(state): Extension<SharedServerState<SessionData>>,
) -> impl IntoResponse {
    if state.clients.lock().await.get(&connection_id).is_none() {
        ws.on_upgrade(move |socket| {
            // By splitting we can send and receive at the same time.
            let (sender, receiver) = socket.split();

            ClientRunner::new(state, Arc::new(Mutex::new(sender)), receiver, connection_id).run()
        })
    } else {
        format!("User [{}] Already Connected", connection_id).into_response()
    }
}

struct ClientRunner<T> {
    state: SharedServerState<T>,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    receiver: SplitStream<WebSocket>,
    connection_id: String,
    cached_session: Option<String>,
}

impl ClientRunner<SessionData> {
    pub fn new(
        state: SharedServerState<SessionData>,
        sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
        receiver: SplitStream<WebSocket>,
        connection_id: String,
    ) -> Self {
        Self {
            connection_id,
            sender,
            receiver,
            state,
            cached_session: None,
        }
    }

    async fn run(mut self) {
        self.connect_client().await;

        // Loop until a text message is found.
        while let Some(Ok(message)) = self.receiver.next().await {
            match message {
                Message::Text(text) => {
                    if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                        self.process_event(event).await;
                    }
                }
                // unhandled other cases
                _ => info!("{:?}", message),
            }
        }

        self.disconnect_client().await;
    }

    async fn connect_client(&self) {
        self.state.clients.lock().await.insert(
            self.connection_id.clone(),
            Client {
                id: self.connection_id.clone(),
                sender: self.sender.clone(),
            },
        );
    }

    async fn disconnect_client(&self) {
        self.state.clients.lock().await.remove(&self.connection_id);

        if let Some(session_id) = &self.cached_session {
            self.leave_session(session_id).await;
        };
    }

    async fn leave_session(&self, session_id: &str) {
        // remove the client from the session and check if the session become empty
        let empty = if let Some(session) = self.state.sessions.lock().await.get_mut(session_id) {
            session.set_client_status(&self.connection_id, false);
            session.active_client_set().is_empty()
        } else {
            false
        };

        if empty {
            self.state.sessions.lock().await.remove(session_id);
        }
    }

    async fn process_event(&mut self, event: ClientEvent) {
        match event {
            ClientEvent::AimUpdate { angle } => {
                let Some(session_id) = &self.cached_session else {
                    return;
                };

                if let Some(session) = self.state.sessions.lock().await.get_mut(session_id) {
                    session
                        .data
                        .gamestate
                        .lock()
                        .await
                        .set_player_angle(&self.connection_id, angle);
                }
            }
            ClientEvent::MovementUpdate { direction } => {
                let Some(session_id) = &self.cached_session else {
                    return;
                };

                if let Some(session) = self.state.sessions.lock().await.get_mut(session_id) {
                    session
                        .data
                        .gamestate
                        .lock()
                        .await
                        .set_player_movement(&self.connection_id, &direction);
                }
            }
            ClientEvent::Shoot => {
                let Some(session_id) = &self.cached_session else {
                    return;
                };

                if let Some(session) = self.state.sessions.lock().await.get_mut(session_id) {
                    session
                        .data
                        .gamestate
                        .lock()
                        .await
                        .player_shoot(&self.connection_id);
                }
            }
            ClientEvent::CreateSession => {
                let new_session = self.create_session(None).await;

                self.state
                    .sessions
                    .lock()
                    .await
                    .insert(new_session.id.clone(), new_session);
            }
            ClientEvent::JoinSession(session_id) => {
                let mut lock = self.state.sessions.lock().await;

                if let Some(session) = lock.get_mut(&session_id) {
                    session
                        .client_statuses
                        .insert(self.connection_id.clone(), true);

                    session.data.gamestate.lock().await.players.insert(
                        self.connection_id.clone(),
                        Player::new(self.connection_id.clone()),
                    );

                    self.cached_session = Some(session.id.clone());
                } else {
                    drop(lock);

                    let mut session = self.create_session(Some(session_id)).await;

                    session
                        .client_statuses
                        .insert(self.connection_id.clone(), true);

                    session.data.gamestate.lock().await.players.insert(
                        self.connection_id.clone(),
                        Player::new(self.connection_id.clone()),
                    );

                    self.cached_session = Some(session.id.clone());

                    self.state
                        .sessions
                        .lock()
                        .await
                        .insert(session.id.clone(), session);
                }
            }
            ClientEvent::LeaveSession => {
                if let Some(session_id) = &self.cached_session {
                    self.leave_session(session_id).await;
                }
            }
        }
    }

    async fn create_session(
        &mut self,
        reserved_id: Option<String>,
    ) -> Session<crate::SessionContainer> {
        let session_id = reserved_id.unwrap_or_else(|| generate_session_id());
        let mut session = Session::<SessionData>::new(session_id.clone());

        session
            .client_statuses
            .insert(self.connection_id.clone(), true);

        session.data.gamestate.lock().await.players.insert(
            self.connection_id.clone(),
            Player::new(self.connection_id.clone()),
        );

        info!("new session [{}] created", session_id);

        self.cached_session = Some(session_id);

        session
    }
}
