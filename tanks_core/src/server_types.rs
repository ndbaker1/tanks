use crate::shared_types::Vec2d;
/**
 * This file contains type defintions which are shared between the front and back end applications
 */
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    PlayerPosUpdate { player: String, coord: Vec2d },
    PlayerDisconnect { player: String },
    BulletExplode(Vec2d),
    BulletData(Vec<(Vec2d, f64)>),
}

#[derive(Serialize, Deserialize)]
pub enum ClientEvent {
    /// Store keys in UPPERCASE
    PlayerControlUpdate {
        key: String,
        press: bool,
    },
    PlayerShoot {
        angle: f64,
    },
    JoinSession,
    CreateSession,
    LeaveSession,
}
