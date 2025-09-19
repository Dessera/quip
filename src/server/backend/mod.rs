pub mod memory;

#[cfg(test)]
pub(crate) mod test;

pub use self::memory::*;
use crate::{TcResult, server::user::User};
use std::future::Future;

/// Server backend interface, which implements storage of user.
pub trait Backend {
    /// Add a user to backend.
    fn add_user(&self, user: User) -> impl Future<Output = TcResult<()>> + Send;

    // Remove a user from backend.
    fn remove_user(&self, user: User) -> impl Future<Output = TcResult<()>> + Send;

    /// Rename a user in backend.
    fn rename_user(&self, original: &str, name: &str) -> impl Future<Output = TcResult<()>> + Send;

    /// Find a user from backend.
    fn find_user(&self, name: &str) -> impl Future<Output = TcResult<User>> + Send;
}
