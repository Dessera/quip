use crate::{
    QuipResult,
    server::{connection::Connection, listener::Listener},
};
use log::info;
use std::net::SocketAddr;
use tokio::net::{TcpListener as TokioTcpListener, ToSocketAddrs};

/// Wrapper for [`tokio::net::TcpListener`].
pub struct TcpListener(TokioTcpListener);

impl TcpListener {
    /// Create a new [`TcpListener`] with specific address.
    pub async fn bind<T: ToSocketAddrs>(addr: T) -> QuipResult<Self> {
        Ok(Self(TokioTcpListener::bind(addr).await?))
    }
}

impl Listener for TcpListener {
    async fn accept(&self) -> QuipResult<Connection> {
        let (socket, addr) = self.0.accept().await?;
        info!("Tcp socket {} accepted", addr);

        Ok(Connection::new(socket, addr))
    }

    fn address(&self) -> QuipResult<SocketAddr> {
        Ok(self.0.local_addr()?)
    }
}
