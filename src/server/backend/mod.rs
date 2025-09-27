pub mod memory;

#[cfg(test)]
pub(crate) mod test;

pub use self::memory::*;
use crate::{QuipResult, server::connection::ConnectionRef};
use std::future::Future;

/// Server backend interface, which implements storage of user.
pub trait Backend {
    /// Load a user in backend.
    fn load_user(&self, name: &str) -> impl Future<Output = QuipResult<ConnectionRef>> + Send;

    // Unload a user in backend.
    fn unload_user(&self, name: &str) -> impl Future<Output = QuipResult<()>> + Send;

    /// Find a user from backend.
    fn find_user(&self, name: &str) -> impl Future<Output = QuipResult<ConnectionRef>> + Send;

    /// Find or create a user in backend.
    fn ensure_user(&self, name: &str) -> impl Future<Output = QuipResult<ConnectionRef>> + Send;
}
