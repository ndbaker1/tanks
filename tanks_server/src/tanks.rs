use crate::{
    map::{parse_maps, MapData},
    utils::{process_collisions, VecOps},
};
use lazy_static::lazy_static;
use nanoid::nanoid;
use serde_json::from_str;
use std::{collections::HashMap, time::Duration};
use tanks_core::{
    server_types::{ClientEvent, ServerEvent},
    shared_types::{Bullet, PlayerData, PlayerState, ServerGameState, Tickable, Vec2d},
};
use tokio::time::delay_for;
use websocket_server::{
    cleanup_session, message_client, sessions::Session, SafeClients, SafeSessions,
};

lazy_static! {
    /// Global Reference to MapData loaded at the beginning of the server
    static ref MAPS: Vec<MapData> = parse_maps("assets/mapdata.mf");
}

/// The handler for game logic on the server
///
/// This will take a lot of bandwidth if the rate is too high
pub async fn tick_handler(clients: SafeClients, sessions: SafeSessions<ServerGameState>) {
    loop {
        for session in sessions.write().await.values_mut() {
            let game_data = &mut session.data;

            // Tick all bullets
            for bullet in &mut game_data.bullets {
                bullet.tick();
            }

            // Check Bullet Wall Collisions
            game_data
                .bullets
                .drain_remove_if(|bullet| {
                    bullet.pos.x < 0.0
                        || bullet.pos.x > 1000.0
                        || bullet.pos.y < 0.0
                        || bullet.pos.y > 1000.0
                })
                .into_iter()
                .for_each(|bullet| {
                    if let Some(player) = game_data.players.get_mut(&bullet.player_id) {
                        player.bullets_left += 1;
                    }
                });

            for index in process_collisions(&game_data.bullets) {
                let removed = game_data.bullets.remove(index);
                if let Some(player) = game_data.players.get_mut(&removed.player_id) {
                    player.bullets_left += 1;
                }
                for client_id in game_data.get_player_ids() {
                    if let Some(client) = clients.read().await.get(client_id) {
                        message_client(client, &ServerEvent::BulletExplode(removed.pos));
                    }
                }
            }

            // Relay Bullet data to each Player
            for client_id in game_data.get_player_ids() {
                let bullets = game_data
                    .bullets
                    .iter()
                    .map(|bullet| (bullet.pos, bullet.angle))
                    .collect::<Vec<_>>();

                if let Some(client) = clients.read().await.get(client_id) {
                    message_client(client, &ServerEvent::BulletData(bullets.clone()));
                }
            }

            // Update
            let update_list: Vec<(String, Vec2d)> = game_data
                .players
                .iter_mut()
                .filter_map(|(player_id, player_data)| match player_data.tick() {
                    true => Some((player_id.clone(), player_data.position.clone())),
                    false => None,
                })
                .collect();

            // Update
            for (player_id, player_data) in update_list {
                for client_id in game_data.get_player_ids() {
                    if let Some(client) = clients.read().await.get(client_id) {
                        message_client(
                            client,
                            &ServerEvent::PlayerPosUpdate {
                                player: player_id.clone(),
                                coord: player_data.clone(),
                            },
                        );
                    }
                }
            }
        }

        // wait the tick on the server
        delay_for(Duration::from_millis(1000 / 60)).await;
    }
}

/// Handle the Client events from a given Session
pub async fn handle_event(
    client_id: String,
    event: String,
    clients: SafeClients,
    sessions: SafeSessions<ServerGameState>,
) {
    //======================================================
    // Deserialize into Session Event object
    //======================================================
    let client_event = match from_str::<ClientEvent>(&event) {
        Ok(obj) => obj,
        Err(_) => return log::error!("failed to parse ClientEvent struct from string: {}", event),
    };

    match client_event {
        ClientEvent::PlayerControlUpdate { key, press } => {
            let session_id = get_client_session_id(&client_id, &clients).await.unwrap();
            if let Some(session) = sessions.write().await.get_mut(&session_id) {
                if let Some(player_data) = session.data.players.get_mut(&client_id) {
                    match press {
                        true => player_data.keys_down.insert(key),
                        false => player_data.keys_down.remove(&key),
                    };
                }
            }
        }
        ClientEvent::PlayerShoot { angle } => {
            let session_id = get_client_session_id(&client_id, &clients).await.unwrap();

            if let Some(session) = sessions.write().await.get_mut(&session_id) {
                if let Some(player) = session.data.players.get_mut(&client_id) {
                    if let PlayerState::Idle = player.state {
                        if player.bullets_left > 0 {
                            let bullet_speed = 10.0;

                            player.bullets_left -= 1;

                            session.data.bullets.push(Bullet {
                                player_id: client_id.clone(),
                                angle,
                                pos: player.position.clone(),
                                velocity: Vec2d {
                                    x: bullet_speed * angle.cos(),
                                    y: bullet_speed * angle.sin(),
                                },
                            });

                            player.state = PlayerState::Shooting(4);
                        }
                    }
                }
            }
        }
        ClientEvent::CreateSession => {
            log::info!("request from <{}> to create new session", client_id);
            create_session(&client_id, None, &sessions, &clients).await;
        }
        ClientEvent::JoinSession => {
            log::info!("request from <{}> to join a session", client_id);
            // place player in first valid session
            for session in sessions.write().await.values_mut() {
                return insert_client_into_given_session(&client_id, &clients, session).await;
            }

            create_session(&client_id, None, &sessions, &clients).await;
        }
        ClientEvent::LeaveSession => {
            remove_client_from_current_session(&client_id, &clients, &sessions).await;
        }
    }
}

/// Creates a Session with a given Client as its creator / first member
pub async fn create_session(
    client_id: &str,
    session_id: Option<&str>,
    sessions: &SafeSessions<ServerGameState>,
    clients: &SafeClients,
) {
    log::info!("creating session..");
    let session = &mut Session {
        client_statuses: HashMap::new(),
        owner: String::from(client_id),
        id: match session_id {
            Some(id) => String::from(id),
            None => get_rand_session_id(),
        },
        data: ServerGameState {
            players: [(String::from(client_id), PlayerData::new(client_id))]
                .into_iter()
                .collect(),
            map: HashMap::new(),
            bullets: Vec::new(),
        },
    };

    // insert the host client into the session
    session.insert_client(&String::from(client_id), true);

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

    log::info!("finished creating session {}", session.id);
    log::info!("sessions live: {}", sessions.read().await.len());
}

/// Removes a client from the session that they currently exist under
async fn remove_client_from_current_session<T>(
    client_id: &str,
    clients: &SafeClients,
    sessions: &SafeSessions<T>,
) {
    log::info!(
        "attempting to remove client {} from their current session",
        client_id
    );

    let session_id = match get_client_session_id(client_id, clients).await {
        Some(session_id) => session_id,
        None => return log::warn!("client {} was not in a session", client_id),
    };

    let session_empty = match sessions.write().await.get_mut(&session_id) {
        Some(session) => {
            // remove the client from the session
            session.remove_client(client_id);

            log::info!("removed client {} from session {}", client_id, session_id);

            // revoke the client's copy of the session_id
            if let Some(client) = clients.write().await.get_mut(client_id) {
                client.session_id = None;
            }

            session.get_clients_with_active_status(true).is_empty()
        }
        None => false,
    };

    // clean up the session from the map if it is empty
    if session_empty {
        cleanup_session(&session_id, sessions).await;
    }
}

/// Takes a mutable session reference in order to add a client to a given session
///
/// Uses a Read lock for Clients
async fn insert_client_into_given_session(
    client_id: &str,
    clients: &SafeClients,
    session: &mut Session<ServerGameState>,
) {
    // add client to session
    session.insert_client(client_id, true);
    // add client to gamedata
    session
        .data
        .players
        .insert(String::from(client_id), PlayerData::new(client_id));
    // update session_id of client
    if let Some(client) = clients.write().await.get_mut(client_id) {
        client.session_id = Some(session.id.clone());
    }

    log::info!("client <{}> joined session: <{}>", client_id, session.id);
}

/// Gets a random new session 1 that is 5 characters long
/// This should almost ensure session uniqueness when dealing with a sizeable number of sessions
fn get_rand_session_id() -> String {
    let alphabet = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    nanoid!(5, &alphabet)
}

/// pull the session id off of a client
async fn get_client_session_id(client_id: &str, clients: &SafeClients) -> Option<String> {
    match clients.read().await.get(client_id) {
        Some(client) => client.session_id.clone(),
        None => None,
    }
}
