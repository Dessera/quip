//! Quip listener based on TCP stream.

use crate::{
    QuipResult,
    io::{DynamicQuipIO, tcp::QuipTcpStream},
    server::listener::Listener,
};
use log::info;
use tokio::net::{TcpListener as TokioTcpListener, ToSocketAddrs};

/// Wrapper for [`TcpListener`].
pub struct TcpListener {
    listener: TokioTcpListener,
}

impl TcpListener {
    /// Create a new [`TcpListener`] with specific address.
    pub async fn bind<T: ToSocketAddrs>(addr: T) -> QuipResult<Self> {
        let listener = TokioTcpListener::bind(addr).await?;

        if let Ok(local_addr) = listener.local_addr() {
            info!("Tcp listener was binded to {}", local_addr);
        }

        Ok(Self { listener })
    }
}

impl Listener for TcpListener {
    async fn accept(&self) -> QuipResult<DynamicQuipIO> {
        let (socket, addr) = self.listener.accept().await?;
        info!("Tcp socket {} accepted", addr);

        Ok(Box::new(QuipTcpStream::new(socket)))
    }
}
