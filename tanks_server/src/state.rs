use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use tokio::sync::Mutex;

pub type SharedServerState<T> = Arc<ServerState<T>>;

#[derive(Debug, Default)]
pub struct ServerState<T> {
    pub clients: Arc<Mutex<HashMap<String, Client>>>,
    pub sessions: Arc<Mutex<HashMap<String, Session<T>>>>,
}

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

#[derive(Debug)]
pub struct Session<T> {
    pub id: String,
    pub client_statuses: HashMap<String, bool>,
    pub data: T,
}

impl<T: Default> Session<T> {
    pub fn new(id: String) -> Self {
        Self {
            id,
            client_statuses: HashMap::default(),
            data: T::default(),
        }
    }

    pub fn set_client_status(&mut self, client_id: &str, active: bool) {
        if self.client_statuses.contains_key(client_id) {
            self.client_statuses.insert(client_id.to_string(), active);
        }
    }

    pub fn active_client_set(&self) -> Vec<&String> {
        self.client_statuses
            .iter()
            .filter_map(|(client, status)| if *status { Some(client) } else { None })
            .collect()
    }
}

/// Generates a String of given length using characters that are valid for Session IDs
///
/// This should effectively resolve to Session uniqueness when the length is
/// greater than a value like 4 for a plausable number of concurrent sessions
pub(crate) fn generate_session_id() -> String {
    /// The Chosen Length of a Session ID
    const SESSION_ID_LENGTH: usize = 5;

    nanoid::nanoid!(
        SESSION_ID_LENGTH,
        &[
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ]
    )
}
