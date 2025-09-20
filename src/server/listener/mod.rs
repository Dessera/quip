pub mod tcp;

pub use tcp::*;

use crate::{QuipResult, server::connection::Connection};
use std::net::SocketAddr;

/// Server listener interface.
pub trait Listener {
    /// Accept a connection from listener.
    fn accept(&self) -> impl Future<Output = QuipResult<Connection>> + Send;

    /// Get listener address.
    fn address(&self) -> QuipResult<SocketAddr>;
}
