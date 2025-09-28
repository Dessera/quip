use crate::{
    QuipError, QuipResult,
    data::{BackendData, BackendQueryData},
    server::{
        backend::Backend,
        connection::{Connection, ConnectionRef, ConnectionStatus},
    },
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Memory backend implementation.
///
/// All users are stored in memory with a [`HashMap`].
pub struct MemoryBackend {
    data: BackendQueryData,
    conns: Arc<Mutex<HashMap<String, Arc<Mutex<Connection>>>>>,
}

impl MemoryBackend {
    pub fn new(data: BackendQueryData) -> Self {
        Self {
            data,
            conns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create [`MemoryBackend`] from raw data.
    pub fn from_data(data: BackendData) -> QuipResult<Self> {
        let data = data.try_into()?;
        Ok(Self::new(data))
    }
}

impl Backend for MemoryBackend {
    async fn load_conn(&self, name: &str, password: &str) -> QuipResult<ConnectionRef> {
        match self.data.users.get(name) {
            Some(user) => {
                if user.password != password {
                    return Err(QuipError::Authorize(format!(
                        "Incorrect password for user {}",
                        name
                    )));
                }
            }
            None => return Err(QuipError::NotFound(format!("No user named {}", name))),
        };

        let mut conns = self.conns.lock().await;

        let conn = match conns.get(name) {
            Some(conn) => {
                let mut conn_handle = conn.lock().await;

                if conn_handle.status != ConnectionStatus::Cache {
                    return Err(QuipError::Duplicate(format!("User {} exists", name)));
                }

                conn_handle.status = ConnectionStatus::Auth;
                conn.clone()
            }
            None => {
                let conn = Arc::new(Mutex::new(Connection::new(name, ConnectionStatus::Auth)));
                conns.insert(name.into(), conn.clone());
                conn
            }
        };

        Ok(conn)
    }

    async fn unload_conn(&self, name: &str) -> QuipResult<()> {
        let mut conns = self.conns.lock().await;
        conns.remove(name);

        Ok(())
    }

    async fn find_conn(&self, name: &str) -> QuipResult<ConnectionRef> {
        let conns = self.conns.lock().await;
        match conns.get(name) {
            Some(user) => Ok(user.clone()),
            None => return Err(QuipError::NotFound(format!("No user named {}", name))),
        }
    }

    async fn ensure_conn(&self, name: &str) -> QuipResult<ConnectionRef> {
        if let None = self.data.users.get(name) {
            return Err(QuipError::NotFound(format!("No user named {}", name)));
        }

        let mut conns = self.conns.lock().await;

        let conn = match conns.get(name) {
            Some(conn) => conn.clone(),
            None => {
                let conn = Arc::new(Mutex::new(Connection::new(name, ConnectionStatus::Cache)));
                conns.insert(name.into(), conn.clone());
                conn
            }
        };

        Ok(conn)
    }
}
