use crate::{
    TcError, TcResult,
    server::{
        backend::Backend,
        user::{User, UserStatus},
    },
};
use log::info;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Memory backend implementation.
///
/// All users are stored in memory with a [`HashMap`].
pub struct MemoryBackend {
    users: Arc<Mutex<HashMap<String, User>>>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Backend for MemoryBackend {
    async fn add_user(&self, name: impl AsRef<str> + Into<String> + Send) -> TcResult<User> {
        let mut users = self.users.lock().await;
        let name_ref = name.as_ref();

        if let Some(user) = users.get(name_ref) {
            let mut user_data = user.data.lock().await;

            return match user_data.status {
                UserStatus::Cache => {
                    user_data.status = UserStatus::Auth;
                    Ok(user.clone())
                }
                _ => Err(TcError::Duplicate(format!(
                    "User '{}' exists.",
                    user_data.name
                ))),
            };
        }

        info!("User {} has been authenticated.", name_ref);

        let user = User::new(name_ref, UserStatus::Auth);
        users.insert(name.into(), user.clone());

        Ok(user)
    }

    async fn remove_user(&self, user: User) -> TcResult<()> {
        let mut users = self.users.lock().await;
        let user_data = user.data.lock().await;

        users.remove(&user_data.name);

        info!("User {} has left.", user_data.name);

        Ok(())
    }

    async fn rename_user(&self, original: &str, name: &str) -> TcResult<()> {
        let mut users = self.users.lock().await;
        let user = match users.get(original) {
            Some(user) => user.clone(),
            None => return Err(TcError::NotFound(format!("No user named '{}'.", original))),
        };

        if users.contains_key(name) {
            return Err(TcError::Duplicate(format!("User '{}' exists.", name)));
        }

        users.insert(name.to_string(), user);
        users.remove(original);

        info!("User {} has renamed to {}.", original, name);

        Ok(())
    }

    async fn find_user(&self, name: &str) -> TcResult<User> {
        let users = self.users.lock().await;
        match users.get(name) {
            Some(user) => Ok(user.clone()),
            None => return Err(TcError::NotFound(format!("No user named '{}'.", name))),
        }
    }

    async fn ensure_user(&self, name: impl AsRef<str> + Into<String> + Send) -> TcResult<User> {
        let mut users = self.users.lock().await;
        let name_ref = name.as_ref();

        match users.get(name_ref) {
            Some(user) => Ok(user.clone()),
            None => {
                let user = User::new(name_ref, UserStatus::Cache);
                users.insert(name.into(), user.clone());
                Ok(user)
            }
        }
    }
}
