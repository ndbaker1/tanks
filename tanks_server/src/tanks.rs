use crate::utils::{process_collisions, VecOps};
use lazy_static::lazy_static;
use nanoid::nanoid;
use serde_json::from_str;
use std::{collections::HashMap, time::Duration};
use tanks_core::{
    map::{parse_maps, MapData},
    server_types::{ClientEvent, ServerEvent},
    shared_types::{Bullet, PlayerData, PlayerState, ServerGameState, Tickable, Vec2d},
    BULLET_SPEED,
};
use tokio::time::delay_for;
use websocket_server::{
    cleanup_session, message_client,
    sessions::{Client, Clients, Session, Sessions},
    SafeClients, SafeSessions,
};

lazy_static! {
    /// Global Reference to MapData loaded at the beginning of the server
    static ref MAPS: HashMap<String, MapData> = parse_maps("assets/mapdata.mf");
}

/// The handler for game logic on the server
///
/// ### Warnings
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
                .drain_remove_if(|bullet| bullet.collide_bounds())
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
            let session_id = {
                let clients = &clients.read().await;
                pull_client_session_id(&client_id, clients).unwrap()
            };

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
            let session_id = {
                let clients = &clients.read().await;
                pull_client_session_id(&client_id, clients).unwrap()
            };

            if let Some(session) = sessions.write().await.get_mut(&session_id) {
                if let Some(player) = session.data.players.get_mut(&client_id) {
                    if let PlayerState::Idle = player.state {
                        if player.bullets_left > 0 {
                            player.bullets_left -= 1;

                            session.data.bullets.push(Bullet {
                                angle,
                                bounces: 1,
                                player_id: client_id.clone(),
                                pos: player.position.clone(),
                                velocity: Vec2d {
                                    x: BULLET_SPEED * angle.cos(),
                                    y: BULLET_SPEED * angle.sin(),
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

            let session_id = {
                let sessions = &mut sessions.write().await;
                match create_session(None, sessions) {
                    Ok(id) => id,
                    Err(_) => return log::error!("failed to create session.."),
                }
            };

            if let Some(client) = clients.write().await.get_mut(&client_id) {
                if let Some(session) = sessions.write().await.get_mut(&session_id) {
                    insert_client_into_session(client, session);
                }
            }
        }
        ClientEvent::JoinSession(session_id) => {
            log::info!(
                "request from <{}> to join session {}",
                client_id,
                session_id
            );

            // If the Session does not exists then we will create it first
            if sessions.read().await.get(&session_id).is_none() {
                let mut_sessions = &mut sessions.write().await;
                create_session(Some(&session_id), mut_sessions)
                    .expect("unable to create a session with a given id.");
            }

            if let Some(client) = clients.write().await.get_mut(&client_id) {
                if let Some(session) = sessions.write().await.get_mut(&session_id) {
                    insert_client_into_session(client, session);
                }
            }

            if let Some(client) = clients.write().await.get_mut(&client_id) {
                log::warn!("sending map data..");
                message_client(
                    client,
                    &ServerEvent::MapUpdate(MAPS.get("first").unwrap().tile_data.clone()),
                );
            }
        }
        ClientEvent::LeaveSession => {
            if let Some(client) = clients.write().await.get_mut(&client_id) {
                let sessions = &mut sessions.write().await;
                remove_client_from_current_session(client, sessions);
            }
        }
    }
}

/// Creates a new empty Session
///
/// Takes a predefined ID to generate, or uses a randomly generated String
pub fn create_session(
    session_id: Option<&str>,
    sessions: &mut Sessions<ServerGameState>,
) -> Result<String, ()> {
    log::info!("creating session..");
    let session = &mut Session {
        client_statuses: HashMap::new(),
        owner: String::new(),
        id: match session_id {
            Some(id) => String::from(id),
            None => generate_session_id(SESSION_ID_LENGTH),
        },
        data: ServerGameState::default(),
    };

    log::info!("writing new session {} to global sessions", session.id);
    // add a new session into the server
    sessions.insert(session.id.clone(), session.clone());

    log::info!("finished creating session {}", session.id);
    log::info!("sessions live: {}", sessions.len());

    Ok(session.id.clone())
}

/// Removes a client from the session that they currently exist under
fn remove_client_from_current_session<T>(client: &mut Client, sessions: &mut Sessions<T>) {
    log::info!(
        "attempting to remove client {} from their current session",
        client.id
    );

    let session_id = match &client.session_id {
        Some(id) => String::from(id),
        None => return log::warn!("client {} was not in a session", client.id),
    };

    match sessions.get_mut(&session_id) {
        Some(session) => {
            // remove the client from the session
            session.remove_client(&client.id);

            log::info!("removed client {} from session {}", client.id, session_id);
            // revoke the client's reference to the current Session ID
            client.session_id = None;
            // clean up the session from the map if it is empty
            if session.get_clients_with_active_status(true).is_empty() {
                cleanup_session(&session_id, sessions);
            }
        }
        None => log::error!(
            "failed to find session {} to remove client {}",
            session_id,
            client.id
        ),
    }
}

/// Takes a mutable session reference in order to add a client to a given session
///
/// Takes a Read lock for Clients
fn insert_client_into_session(client: &mut Client, session: &mut Session<ServerGameState>) {
    // add client to session
    session.insert_client(&client.id, true);
    // add client to gamedata
    session
        .data
        .players
        .insert(client.id.clone(), PlayerData::new(&client.id));

    log::info!("attaching session {} to client <{}>", session.id, client.id);
    // update session_id of client
    client.session_id = Some(session.id.clone());

    log::info!("client <{}> joined session: <{}>", client.id, session.id);
}

/// The Chosen Length of a Session ID
pub const SESSION_ID_LENGTH: usize = 5;

/// Generates a String of given length using characters that are valid for Session IDs
///
/// This should effectively resolve to Session uniqueness when the length is
/// greater than a value like 4 for a plausable number of concurrent sessions
fn generate_session_id(length: usize) -> String {
    nanoid!(
        length,
        &[
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ]
    )
}

/// pull the session id off of a client
fn pull_client_session_id(client_id: &str, clients: &Clients) -> Option<String> {
    match clients.get(client_id) {
        Some(client) => client.session_id.clone(),
        None => None,
    }
}
