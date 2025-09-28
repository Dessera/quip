pub mod memory;

pub use memory::*;

use crate::{QuipResult, server::connection::ConnectionRef};
use std::future::Future;

/// Server backend interface, which implements storage of connections.
pub trait Backend {
    /// Load a connection in backend.
    fn load_conn(
        &self,
        name: &str,
        password: &str,
    ) -> impl Future<Output = QuipResult<ConnectionRef>> + Send;

    // Unload a connection in backend.
    fn unload_conn(&self, name: &str) -> impl Future<Output = QuipResult<()>> + Send;

    /// Find a connection from backend.
    fn find_conn(&self, name: &str) -> impl Future<Output = QuipResult<ConnectionRef>> + Send;

    /// Find or create a connection in backend.
    fn ensure_conn(&self, name: &str) -> impl Future<Output = QuipResult<ConnectionRef>> + Send;
}
