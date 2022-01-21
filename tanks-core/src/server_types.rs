use crate::shared_types::Coord;
/**
 * This file contains type defintions which are shared between the front and back end applications
 */
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    PlayerPosUpdate { player: String, coord: Coord },
    PlayerDisconnect { player: String },
}

#[derive(Serialize, Deserialize)]
pub enum ClientEvent {
    /// Store keys in UPPERCASE
    PlayerControlUpdate {
        key: String,
        press: bool,
    },
    JoinSession,
    CreateSession,
    LeaveSession,
}
