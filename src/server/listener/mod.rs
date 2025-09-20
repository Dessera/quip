pub mod tcp;
pub mod tls;

pub use tcp::*;
pub use tls::*;

use crate::{QuipResult, server::stream::QuipStream};
use std::net::SocketAddr;

/// Server listener interface.
pub trait Listener {
    /// Accept a connection from listener.
    fn accept(&self) -> impl Future<Output = QuipResult<QuipStream>> + Send;

    /// Get listener address.
    fn address(&self) -> QuipResult<SocketAddr>;
}
