//! Quip listener interface.

pub mod tcp;
pub mod tls;

use crate::{QuipResult, io::DynamicQuipIO};
use std::net::SocketAddr;

/// Server listener interface.
pub trait Listener {
    /// Accept a connection from listener.
    fn accept(&self) -> impl Future<Output = QuipResult<DynamicQuipIO>> + Send;

    /// Get listener address.
    fn address(&self) -> QuipResult<SocketAddr>;
}
