//! This file contains type defintions which are shared between the front and back end applications

use serde::{Deserialize, Serialize};

use tanks_core::utils::Vector2;

/// Server Events
#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    Game(ServerGameEvent),
    Session(ServerSessionEvent),
}

#[derive(Serialize, Deserialize)]
pub enum ServerSessionEvent {
    PlayerDisconnect { player: String },
}

#[derive(Serialize, Deserialize)]
pub enum ServerGameEvent {
    ClientPosUpdate { player: String, coord: Vector2 },
    BulletExplode(Vector2),
    BulletData(Vec<(Vector2, f64)>),
}

/// Client Events
#[derive(Serialize, Deserialize)]
pub enum ClientEvent {
    Game(ClientGameEvent),
    Session(ClientSessionEvent),
}

#[derive(Serialize, Deserialize)]
pub enum ClientSessionEvent {
    /// Join a Session with a Given ID
    JoinSession(String),
    CreateSession,
    LeaveSession,
}

#[derive(Serialize, Deserialize)]
pub enum ClientGameEvent {
    MovementUpdate { direction: Vector2 },
    Shoot,
}
