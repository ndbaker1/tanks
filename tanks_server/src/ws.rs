use std::{sync::Arc, vec};

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
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tanks_core::{
    common::{gamestate::GameState, player::Player},
    utils::Vector2,
};
use tanks_events::{ClientEvent, ServerEvent, TankWrapper};
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
    if state.clients.get(&connection_id).is_none() {
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
        self.connect_client();

        // place the player into a session
        self.process_event(ClientEvent::CreateSession).await;

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

        self.disconnect_client();
    }

    fn connect_client(&self) {
        self.state.clients.insert(
            self.connection_id.clone(),
            Client {
                id: self.connection_id.clone(),
                sender: self.sender.clone(),
            },
        );
    }

    fn disconnect_client(&self) {
        self.state.clients.remove(&self.connection_id);

        if let Some(session_id) = &self.cached_session {
            self.leave_session(session_id);
        };
    }

    fn leave_session(&self, session_id: &str) {
        // remove the client from the session and check if the session become empty
        let Some(mut session) = self.state.sessions.get_mut(session_id) else {
            return;
        };

        session.set_client_status(&self.connection_id, false);

        if session.active_client_set().is_empty() {
            drop(session); // explicit drop to avoid deadlock
            self.cleanup_session(session_id);
        }
    }

    async fn process_event(&mut self, event: ClientEvent) {
        match event {
            ClientEvent::MovementUpdate { direction } => {
                let Some(session_id) = &self.cached_session else {
                    return;
                };

                let Some(session) = self.state.sessions.get_mut(session_id) else {
                    return;
                };

                session
                    .data
                    .gamestate
                    .lock()
                    .await
                    .set_player_movement(&self.connection_id, &direction);
            }
            ClientEvent::Shoot => {
                let Some(session_id) = &self.cached_session else {
                    return;
                };

                let Some(session) = self.state.sessions.get_mut(session_id) else {
                    return;
                };

                session
                    .data
                    .gamestate
                    .lock()
                    .await
                    .player_shoot(&self.connection_id);
            }
            ClientEvent::CreateSession => {
                let session_id = generate_session_id();
                let mut session = Session::<SessionData>::new(session_id.clone());

                // Start ticking the GameState when the session goes live
                let gamestate_tick_arc = session.data.gamestate.clone();
                session.data.task_handles.push(tokio::spawn(async move {
                    loop {
                        gamestate_tick_arc.lock().await.tick();
                        tokio::time::sleep(tokio::time::Duration::from_secs_f64(1.0 / 60.0)).await;
                    }
                }));

                let gamestate_broadcast_arc = session.data.gamestate.clone();
                let state_copy = self.state.clone();
                let session_id_copy = session_id.clone();
                session.data.task_handles.push(tokio::spawn(async move {
                    loop {
                        let gamestate = gamestate_broadcast_arc.lock().await;
                        // collect stats to deliver to clients
                        let broadcast_data = ServerEvent::GameState {
                            bullets: vec![],
                            tanks: gamestate
                                .players
                                .iter()
                                .map(
                                    |(
                                        _,
                                        Player {
                                            id,
                                            angle,
                                            position,
                                            movement,
                                            ..
                                        },
                                    )| TankWrapper {
                                        angle: *angle,
                                        id: id.clone(),
                                        movement: *movement,
                                        position: *position,
                                    },
                                )
                                .collect(),
                        };

                        // broadcast to all clients
                        for client_id in state_copy
                            .sessions
                            .get(&session_id_copy)
                            .unwrap()
                            .active_client_set()
                        {
                            state_copy
                                .clients
                                .get_mut(client_id)
                                .unwrap()
                                .sender
                                .lock()
                                .await
                                .send(Message::Text(
                                    serde_json::to_string(&broadcast_data).unwrap(),
                                ))
                                .await
                                .unwrap();
                        }

                        tokio::time::sleep(tokio::time::Duration::from_secs_f64(1.0 / 10.0)).await;
                    }
                }));

                session
                    .client_statuses
                    .insert(self.connection_id.clone(), true);

                session.data.gamestate.lock().await.players.insert(
                    self.connection_id.clone(),
                    Player::new(self.connection_id.clone()),
                );

                self.cached_session = Some(session_id.clone());

                self.state.sessions.insert(session_id.clone(), session);
            }
            ClientEvent::JoinSession(session_id) => {
                let Some(mut session) = self.state.sessions.get_mut(&session_id) else {
                    return;
                };

                session
                    .client_statuses
                    .insert(self.connection_id.clone(), true);

                session.data.gamestate.lock().await.players.insert(
                    self.connection_id.clone(),
                    Player::new(self.connection_id.clone()),
                );

                self.cached_session = Some(session_id.clone());
            }
            ClientEvent::LeaveSession => {
                if let Some(session_id) = &self.cached_session {
                    self.leave_session(session_id);
                };
            }
        }
    }

    /// Remove sessions and kill background tasks spawned from it
    fn cleanup_session(&self, session_id: &str) {
        if let Some(session) = self.state.sessions.remove(session_id) {
            for task in session.1.data.task_handles {
                task.abort();
            }
        }
    }
}

fn from(gs: &GameState) -> ServerEvent {
    ServerEvent::GameState {
        bullets: vec![],
        tanks: vec![],
    }
}
