/**
 * This file contains type defintions which are shared between the front and back end applications
 */
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::shared_types::{Coord, ServerGameState};

#[derive(Deserialize, Serialize, Builder)]
#[builder(pattern = "immutable")]
pub struct Event<Code, PayloadType> {
    pub event_code: Code,
    #[builder(setter(strip_option), default)]
    pub message: Option<String>,
    #[builder(setter(strip_option), default)]
    pub data: Option<PayloadType>,
}

#[derive(Serialize, Deserialize, Clone, Builder)]
pub struct ServerEventData {
    #[builder(setter(strip_option), default)]
    pub session_id: Option<String>,
    #[builder(setter(strip_option), default)]
    pub client_id: Option<String>,
    #[builder(setter(strip_option), default)]
    pub session_client_ids: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    pub game_data: Option<ServerGameState>,
}

#[derive(Serialize, Deserialize, Builder)]
pub struct ClientEventData {
    #[builder(setter(strip_option), default)]
    pub target_ids: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    pub session_id: Option<String>,
    #[builder(setter(strip_option), default)]
    pub column: Option<usize>,
}

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
