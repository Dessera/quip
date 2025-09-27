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
    users: Arc<Mutex<HashMap<String, Arc<Mutex<Connection>>>>>,
}

impl MemoryBackend {
    pub fn new(data: BackendQueryData) -> Self {
        Self {
            data,
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create [`MemoryBackend`] from raw data.
    pub fn from_data(data: BackendData) -> QuipResult<Self> {
        let data = data.try_into()?;
        Ok(Self::new(data))
    }
}

impl Backend for MemoryBackend {
    async fn load_user(&self, name: &str) -> QuipResult<ConnectionRef> {
        if let None = self.data.users.get(name) {
            return Err(QuipError::NotFound(format!("No user named {}", name)));
        }

        let mut users = self.users.lock().await;

        let user = match users.get(name) {
            Some(user) => {
                let mut user_handle = user.lock().await;

                if user_handle.status != ConnectionStatus::Cache {
                    return Err(QuipError::Duplicate(format!("User {} exists", name)));
                }

                user_handle.status = ConnectionStatus::Auth;
                user.clone()
            }
            None => {
                let user = Arc::new(Mutex::new(Connection::new(name, ConnectionStatus::Auth)));
                users.insert(name.into(), user.clone());
                user
            }
        };

        Ok(user)
    }

    async fn unload_user(&self, name: &str) -> QuipResult<()> {
        let mut users = self.users.lock().await;
        users.remove(name);

        Ok(())
    }

    async fn find_user(&self, name: &str) -> QuipResult<ConnectionRef> {
        let users = self.users.lock().await;
        match users.get(name) {
            Some(user) => Ok(user.clone()),
            None => return Err(QuipError::NotFound(format!("No user named {}", name))),
        }
    }

    async fn ensure_user(&self, name: &str) -> QuipResult<ConnectionRef> {
        if let None = self.data.users.get(name) {
            return Err(QuipError::NotFound(format!("No user named {}", name)));
        }

        let mut users = self.users.lock().await;

        let user = match users.get(name) {
            Some(user) => user.clone(),
            None => {
                let user = Arc::new(Mutex::new(Connection::new(name, ConnectionStatus::Cache)));
                users.insert(name.into(), user.clone());
                user
            }
        };

        Ok(user)
    }
}
