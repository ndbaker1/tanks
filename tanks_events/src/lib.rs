//! This file contains type defintions which are shared between the front and back end applications

use serde::{Deserialize, Serialize};

use tanks_core::utils::Vector2;

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerEvent {
    // Game Related Events
    GameState {
        bullets: Vec<BulletWrapper>,
        tanks: Vec<TankWrapper>,
    },
    BulletExplode(Vector2),
    // Session Related Rvents
    PlayerDisconnect {
        player: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulletWrapper {
    pub position: Vector2,
    pub angle: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TankWrapper {
    pub id: String,
    pub position: Vector2,
    pub movement: Vector2,
    pub angle: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientEvent {
    // Game Related Events
    MovementUpdate {
        direction: Vector2,
    },
    AimUpdate {
        angle: f64,
    },
    Shoot,
    // Session Related Events
    /// Join a Session with a Given ID
    JoinSession(String),
    CreateSession,
    LeaveSession,
}
