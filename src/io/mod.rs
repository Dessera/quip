//! Quip stream interfaces.

pub mod buffer;
pub mod tcp;
pub mod tls;

use tokio::io::{AsyncRead, AsyncWrite};

/// Quip IO interface.
///
/// All IO socket should implements [`AsyncRead`] and [`AsyncWrite`]. Besides,
/// the `duplex` method should be implemented to read and write at the same
/// time.
pub trait QuipIO: AsyncRead + AsyncWrite + Send + Unpin {
    fn duplex(self: Box<Self>) -> (DynamicQuipInput, DynamicQuipOutput);
}

pub type DynamicQuipIO = Box<dyn QuipIO>;

pub trait QuipInput: AsyncRead + Send + Unpin {}

pub type DynamicQuipInput = Box<dyn QuipInput>;

impl<T> QuipInput for T where T: AsyncRead + Send + Unpin {}

pub trait QuipOutput: AsyncWrite + Send + Unpin {}

pub type DynamicQuipOutput = Box<dyn QuipOutput>;

impl<T> QuipOutput for T where T: AsyncWrite + Send + Unpin {}
