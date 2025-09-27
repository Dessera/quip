use crate::response::Response;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{Mutex, Notify};

/// Connection status to cache message before login.
#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    Cache,
    Auth,
}

/// Connection handler for server.
#[derive(Debug)]
pub struct Connection {
    pub queue: Arc<Mutex<VecDeque<Response>>>,
    pub notify: Arc<Notify>,
    pub name: String,
    pub status: ConnectionStatus,
}

pub type ConnectionRef = Arc<Mutex<Connection>>;

impl Connection {
    pub fn new(name: impl Into<String>, status: ConnectionStatus) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
            name: name.into(),
            status,
        }
    }
}
