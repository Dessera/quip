//! Quip listener interface.

pub mod tcp;
pub mod tls;

pub use tcp::*;
pub use tls::*;

use crate::{QuipResult, io::DynamicQuipIO};

/// Server listener interface.
pub trait Listener {
    /// Accept a connection from listener.
    fn accept(&self) -> impl Future<Output = QuipResult<DynamicQuipIO>> + Send;
}
