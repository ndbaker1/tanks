/**
 * This file contains type defintions which are shared between the front and back end applications
 */
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Debug, Clone)]
pub struct GameData {
    pub turn_index: usize,
    pub player_order: Vec<String>,
    pub play_indexes: Vec<Vec<usize>>,
}

#[derive(Deserialize, Serialize, Builder)]
pub struct Event<Code, PayloadType> {
    pub event_code: Code,
    #[builder(setter(into, strip_option), default)]
    pub message: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub data: Option<PayloadType>,
}

pub type ServerEvent = Event<ServerEventCode, ServerEventData>;
pub type ClientEvent = Event<ClientEventCode, ClientEventData>;

#[derive(Serialize, Clone, Builder)]
pub struct ServerEventData {
    #[builder(setter(into, strip_option), default)]
    pub session_id: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub client_id: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub session_client_ids: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    pub game_data: Option<GameData>,
}

#[derive(Deserialize, Builder)]
pub struct ClientEventData {
    #[builder(setter(into, strip_option), default)]
    pub target_ids: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    pub session_id: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub column: Option<usize>,
}

#[derive(Serialize_repr, Clone)]
#[repr(u8)]
pub enum ServerEventCode {
    ClientJoined = 1,
    ClientLeft,
    SessionResponse,
    CannotJoinInProgress,
}

#[derive(Deserialize_repr)]
#[repr(u8)]
pub enum ClientEventCode {
    /**
     * Session Related Events
     */
    JoinSession = 1,
    CreateSession,
    LeaveSession,
    SessionRequest,
    /**
     * Game Related Events
     */
    StartGame,
    Play,
}
