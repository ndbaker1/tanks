use std::collections::HashMap;
use tokio::sync::mpsc;
use warp::ws::Message;

pub type Clients = HashMap<String, Client>;
pub type Sessions<T> = HashMap<String, Session<T>>;

/// Data Stored for a Single User in a Session
#[derive(Debug, Clone)]
pub struct Client {
    /// Unique ID of the Client
    pub id: String,
    /// Session ID the client belongs to if it exists
    pub session_id: Option<String>,
    /// Sender pipe used to relay messages to the sender socket
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

/// Data Stored for a Game Sessions
#[derive(Debug, Clone)]
pub struct Session<T> {
    /// Unique ID of the Session
    pub id: String,
    /// ID of the Client who owns the Session
    pub owner: String,
    /// Active statuses of every Client in the Session
    pub client_statuses: HashMap<String, bool>,
    /// Data that you want to store in a sessions
    pub data: T,
}

impl<T> Session<T> {
    pub fn get_num_clients(&self) -> usize {
        self.client_statuses.len()
    }

    pub fn contains_client(&self, id: &str) -> bool {
        self.client_statuses.contains_key(id)
    }

    pub fn get_client_ids(&self) -> Vec<String> {
        self.client_statuses
            .clone()
            .into_iter()
            .map(|(id, _)| id)
            .collect::<Vec<String>>()
    }

    pub fn remove_client(&mut self, id: &str) {
        self.client_statuses.remove(id);
    }

    pub fn insert_client(&mut self, id: &str, is_active: bool) {
        self.client_statuses.insert(id.to_string(), is_active);
    }

    pub fn get_clients_with_active_status(&self, active_status: bool) -> Vec<String> {
        self.client_statuses
            .clone()
            .into_iter()
            .filter(|(_, status)| status == &active_status)
            .map(|(id, _)| id)
            .collect::<Vec<String>>()
    }

    pub fn set_client_active_status(&mut self, id: &str, is_active: bool) -> Result<(), String> {
        match self.client_statuses.get(id) {
            Some(_) => {
                self.client_statuses.insert(id.to_string(), is_active);
                Ok(())
            }
            None => Err(format!(
                "tried to set active_status of client: {} but id was not found in session",
                id
            )),
        }
    }
}
