//! Quip listener based on TCP stream.

use crate::{
    QuipResult,
    io::{DynamicQuipIO, tcp::QuipTcpStream},
    server::listener::Listener,
};
use log::info;
use std::net::SocketAddr;
use tokio::net::{TcpListener as TokioTcpListener, ToSocketAddrs};

/// Wrapper for [`TcpListener`].
pub struct TcpListener(TokioTcpListener);

impl TcpListener {
    /// Create a new [`TcpListener`] with specific address.
    pub async fn bind<T: ToSocketAddrs>(addr: T) -> QuipResult<Self> {
        Ok(Self(TokioTcpListener::bind(addr).await?))
    }
}

impl Listener for TcpListener {
    async fn accept(&self) -> QuipResult<DynamicQuipIO> {
        let (socket, addr) = self.0.accept().await?;
        info!("Tcp socket {} accepted", addr);

        Ok(Box::new(QuipTcpStream::new(socket)))
    }

    fn address(&self) -> QuipResult<SocketAddr> {
        Ok(self.0.local_addr()?)
    }
}
