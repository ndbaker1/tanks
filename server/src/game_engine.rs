use crate::{
    data_types::{self, SafeClients},
    shared_types::{
        ClientEvent, ClientEventCode, EventBuilder, ServerEvent, ServerEventCode,
        ServerEventDataBuilder,
    },
    ws::cleanup_session,
};
use nanoid::nanoid;
use serde_json::from_str;
use sessions::session_types;
use std::collections::HashMap;
use warp::ws::Message;

/// Handle the Client events from a given Session
pub async fn handle_event(
    client_id: String,
    event: String,
    clients: data_types::SafeClients,
    sessions: data_types::SafeSessions<u32>,
) {
    //======================================================
    // Deserialize into Session Event object
    //======================================================
    let client_event = match from_str::<ClientEvent>(&event) {
        Ok(obj) => obj,
        Err(_) => {
            log::error!("failed to parse ClientEvent struct from string: {}", event);
            return;
        }
    };

    match client_event.event_code {
        ClientEventCode::SessionRequest => {
            let session_id: String = match get_client_session_id(&client_id, &clients).await {
                Some(session_id) => session_id,
                None => return, // no session is ok
            };

            // create a base server event
            let mut server_data = ServerEventDataBuilder::default()
                .session_id(session_id.clone())
                .build()
                .unwrap();

            if let Some(session) = sessions.read().await.get(&session_id) {
                server_data.session_client_ids = Some(session.get_client_ids());
            }

            notify_client_async(
                &client_id,
                &EventBuilder::default()
                    .event_code(ServerEventCode::SessionResponse)
                    .data(server_data)
                    .build()
                    .unwrap(),
                &clients,
            )
            .await;
        }
        ClientEventCode::CreateSession => {
            log::info!("request from {} to create new session", client_id);
            create_session(&client_id, None, &sessions, &clients).await;
        }
        ClientEventCode::JoinSession => {
            log::info!("request from {} to join new session", client_id);

            let session_id = match client_event.data {
                Some(data) => match data.session_id {
                    Some(session_id) => session_id,
                    None => panic!("no session id found!"),
                },
                None => {
                    log::error!("the session id to join was missing in the request");
                    return;
                } // no session was found on a session join request? ¯\(°_o)/¯
            };

            log::info!(
                "checking if client {} is already in session {}",
                client_id,
                session_id
            );
            if let Some(session) = sessions.read().await.get(&session_id) {
                if session.client_statuses.contains_key(&client_id) {
                    log::info!(
                        "client {} was already in session {}. (no-op)",
                        client_id,
                        session.id
                    );
                    return;
                }
            }

            // removing client front session
            remove_client_from_current_session(&client_id, &clients, &sessions).await;

            // Joining Some Session that already exists
            if let Some(session) = sessions.write().await.get_mut(&session_id) {
                // do not allow clients to join an active game
                if session.data.is_none() {
                    log::info!("adding client {} into session {}", client_id, session_id);
                    insert_client_into_given_session(&client_id, &clients, session).await;
                }
                // notify the user that they cannot joing the current session
                else {
                    log::info!(
                        "client {} was not allowed into in-progess session {}",
                        client_id,
                        session_id
                    );
                    notify_client_async(
                        &client_id,
                        &EventBuilder::default()
                            .event_code(ServerEventCode::CannotJoinInProgress)
                            .data(
                                ServerEventDataBuilder::default()
                                    .session_id(session_id)
                                    .build()
                                    .unwrap(),
                            )
                            .build()
                            .unwrap(),
                        &clients,
                    )
                    .await;
                }
                return;
            }

            // Attempt to join a Reserved session, which will be created if it doesnt exist
            log::info!("creating a session from id: {}", session_id);
            create_session(&client_id, Some(&session_id), &sessions, &clients).await;
        }
        ClientEventCode::LeaveSession => {
            remove_client_from_current_session(&client_id, &clients, &sessions).await;
        }
        ClientEventCode::StartGame => {
            let session_id = match get_client_session_id(&client_id, &clients).await {
                Some(s_id) => s_id,
                None => return,
            };

            if let Some(session) = sessions.read().await.get(&session_id) {
                // initialize game data here...
            }
        }
        ClientEventCode::Play => {
            let session_id: String = match get_client_session_id(&client_id, &clients).await {
                Some(s_id) => s_id,
                None => return,
            };

            // if let Some(game_state) = game_states.write().await.get_mut(&session_id) {
            // if game_state.get_turn_player() != client_id {
            //     if let Some(client) = clients.read().await.get(client_id) {
            //         notify_client(
            //             &EventBuilder::default()
            //                 .event_code(ServerEventCode::LogicError)
            //                 .message("It is not your turn to play.")
            //                 .build()
            //                 .unwrap(),
            //             &client,
            //         );
            //     }
            //     // exit
            //     return;
            // }

            // let player_index = match game_state.get_player_index(client_id) {
            //     Some(index) => index,
            //     None => return,
            // };

            // match game_state.play(column, player_index) {
            //     Ok(won) => {
            //         if let Some(session) = sessions.read().await.get(&session_id) {
            //             // if the move was a winning move, then notify everyone that the game is over
            //             if won {
            //                 notify_session(
            //                     &EventBuilder::default()
            //                         .event_code(ServerEventCode::GameEnded)
            //                         .data(
            //                             ServerEventDataBuilder::default()
            //                                 // .client_id(client_id)
            //                                 // .game_data(game_state.as_shared_game_data(None))
            //                                 .build()
            //                                 .unwrap(),
            //                         )
            //                         .build()
            //                         .unwrap(),
            //                     session,
            //                     &clients,
            //                 )
            //                 .await;
            //             }
            //             // else continue emitting the game format
            //             else {
            //                 for (client_name, _) in &session.client_statuses {
            //                     if let Some(client) = clients.read().await.get(client_name) {
            //                         // notify_client(
            //                         //     &EventBuilder::default()
            //                         //         .event_code(ServerEventCode::TurnStart)
            //                         //         .data(
            //                         //             ServerEventDataBuilder::default()
            //                         //                 .client_id(game_state.get_turn_player())
            //                         //                 .game_data(
            //                         //                     game_state.as_shared_game_data(Some(
            //                         //                         client_name,
            //                         //                     )),
            //                         //                 )
            //                         //                 .build()
            //                         //                 .unwrap(),
            //                         //         )
            //                         //         .build()
            //                         //         .unwrap(),
            //                         //     &client,
            //                         // );
            //                     }
            //                 }
            //             }
            //         }
            //     }
            //     Err(e) => {
            //         if let Some(client) = clients.read().await.get(client_id) {
            //             notify_client(
            //                 &quick_server_error("This column has reached its max."),
            //                 &client,
            //             );
            //         }
            //         eprintln!(
            //             "[ERROR] player {} failed to play with err: {}",
            //             client_id, e,
            //         )
            //     }
            // }
            // }
        }
    }
}

/// Creates a Session with a given Client as its creator / first member
pub async fn create_session<T>(
    client_id: &str,
    session_id: Option<&str>,
    sessions: &data_types::SafeSessions<T>,
    clients: &data_types::SafeClients,
) where
    T: Clone,
{
    log::info!("creating session..");
    let session = &mut session_types::Session {
        client_statuses: HashMap::new(),
        owner: client_id.to_string(),
        id: match session_id {
            Some(id) => id.to_string(),
            None => get_rand_session_id(),
        },
        data: None,
    };

    // insert the host client into the session
    session.insert_client(&client_id.to_string(), true);

    log::info!("writing new session {} to global sessions", session.id);
    // add a new session into the server
    sessions
        .write()
        .await
        .insert(session.id.clone(), session.clone());

    log::info!("attaching session {} to client {}", session.id, client_id);
    // update the session reference within the client
    if let Some(client) = clients.write().await.get_mut(client_id) {
        client.session_id = Some(session.id.clone());
    }

    log::info!("send notification to client {}", client_id);
    if let Some(client) = clients.read().await.get(client_id) {
        notify_client(
            &EventBuilder::default()
                .event_code(ServerEventCode::ClientJoined)
                .data(
                    ServerEventDataBuilder::default()
                        .session_id(session.id.clone())
                        .client_id(client_id.to_string())
                        .session_client_ids(session.get_client_ids())
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
            &client,
        );
    }
    log::info!("finished creating session {}", session.id);
    log::info!("sessions live: {}", sessions.read().await.len());
}

/// Send an update to all clients in the session
///
/// Uses a Read lock on clients
async fn notify_session<T>(
    game_update: &ServerEvent,
    session: &session_types::Session<T>,
    clients: &data_types::SafeClients,
) {
    for (client_id, _) in &session.client_statuses {
        if let Some(client) = clients.read().await.get(client_id) {
            notify_client(game_update, client);
        }
    }
}

/// Send and update to a set of clients
async fn _notify_clients(
    game_update: &ServerEvent,
    client_ids: &Vec<String>,
    clients: &data_types::SafeClients,
) {
    for client_id in client_ids {
        if let Some(client) = clients.read().await.get(client_id) {
            notify_client(game_update, client);
        }
    }
}

/// Send an update to single clients
async fn notify_client_async(client_id: &str, game_update: &ServerEvent, clients: &SafeClients) {
    if let Some(client) = clients.read().await.get(client_id) {
        notify_client(game_update, client);
    } else {
        log::error!("could not find client: {}", client_id);
    }
}

/// Send an update to single clients
fn notify_client(game_update: &ServerEvent, client: &session_types::Client) {
    let sender = match &client.sender {
        Some(s) => s,
        None => return log::error!("sender was lost for client: {}", client.id),
    };
    if let Err(e) = sender.send(Ok(Message::text(
        serde_json::to_string(game_update).unwrap(),
    ))) {
        log::error!("failed to send message to {} with err: {}", client.id, e,);
    }
}

/// Removes a client from the session that they currently exist under
async fn remove_client_from_current_session<T>(
    client_id: &str,
    clients: &data_types::SafeClients,
    sessions: &data_types::SafeSessions<T>,
) {
    log::info!(
        "attempting to remove client {} from their current session",
        client_id
    );

    let session_id: String = match get_client_session_id(client_id, clients).await {
        Some(s_id) => s_id,
        None => {
            log::warn!("client {} was not in a session", client_id);
            return;
        } // client did not exist in any session
    };

    let mut session_empty: bool = false;
    if let Some(session) = sessions.write().await.get_mut(&session_id) {
        // notify all clients in the sessions that the client will be leaving
        notify_session(
            &EventBuilder::default()
                .event_code(ServerEventCode::ClientLeft)
                .data(
                    ServerEventDataBuilder::default()
                        .client_id(client_id.to_string())
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
            &session,
            &clients,
        )
        .await;
        // remove the client from the session
        session.remove_client(&client_id.to_string());

        log::info!("removed client {} from session {}", client_id, session_id);

        // revoke the client's copy of the session_id
        if let Some(client) = clients.write().await.get_mut(client_id) {
            client.session_id = None;
        }
        // checks the statuses to see if any users are still active
        session_empty = session.get_clients_with_active_status(true).is_empty();
        // if the session is not empty, make someone else the owner
        if !session_empty {
            set_new_session_owner(session, &clients, &session.get_client_ids()[0]);
        }
    }
    // clean up the session from the map if it is empty
    // * we cannot do this in the scope above because because we are already holding a mutable reference to a session within the map
    if session_empty {
        cleanup_session(&session_id, sessions).await;
    }
}

/// Takes a mutable session reference in order to add a client to a given session
///
/// Uses a Read lock for Clients
async fn insert_client_into_given_session<T>(
    client_id: &str,
    clients: &data_types::SafeClients,
    session: &mut session_types::Session<T>,
) {
    // add client to session
    session.insert_client(client_id, true);
    // update session_id of client
    if let Some(client) = clients.write().await.get_mut(client_id) {
        client.session_id = Some(session.id.clone());
    }
    // notify all clients in the session that the client has joined
    notify_session(
        &EventBuilder::default()
            .event_code(ServerEventCode::ClientJoined)
            .data(
                ServerEventDataBuilder::default()
                    .session_id(session.id.clone())
                    .client_id(client_id.to_string())
                    .session_client_ids(session.get_client_ids())
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
        &session,
        &clients,
    )
    .await;
}

fn set_new_session_owner<T>(
    session: &mut session_types::Session<T>,
    _clients: &data_types::SafeClients,
    client_id: &String,
) {
    session.owner = client_id.clone();
    // notify_all_clients(
    //   &ServerEvent {
    //     event_code: ServerEventCode::SessionOwnerChange,
    //     session_id: Some(session.id.clone()),
    //     client_id: Some(client_id.clone()),
    //     session_client_ids: None,
    //   },
    //   &session,
    //   &clients,
    // );
}

/// Gets a random new session 1 that is 5 characters long
/// This should almost ensure session uniqueness when dealing with a sizeable number of sessions
fn get_rand_session_id() -> String {
    let alphabet: [char; 26] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    nanoid!(5, &alphabet)
}

/// pull the session id off of a client
async fn get_client_session_id(
    client_id: &str,
    clients: &data_types::SafeClients,
) -> Option<String> {
    if let Some(client) = &clients.read().await.get(client_id) {
        client.session_id.clone()
    } else {
        None
    }
}
