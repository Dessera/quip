use crate::{
    QuipResult,
    server::{
        listener::Listener,
        stream::{AsyncReadHalf, AsyncStream, AsyncWriteHalf, QuipStream},
    },
};
use log::info;
use std::net::SocketAddr;
use tokio::net::{TcpListener as TokioTcpListener, TcpStream, ToSocketAddrs};

impl AsyncStream for TcpStream {
    fn duplex(self: Box<Self>) -> (Box<dyn AsyncReadHalf>, Box<dyn AsyncWriteHalf>) {
        let (rx, tx) = self.into_split();
        (Box::new(rx), Box::new(tx))
    }
}

/// Wrapper for [`tokio::net::TcpListener`].
pub struct TcpListener(TokioTcpListener);

impl TcpListener {
    /// Create a new [`TcpListener`] with specific address.
    pub async fn bind<T: ToSocketAddrs>(addr: T) -> QuipResult<Self> {
        Ok(Self(TokioTcpListener::bind(addr).await?))
    }
}

impl Listener for TcpListener {
    async fn accept(&self) -> QuipResult<QuipStream> {
        let (socket, addr) = self.0.accept().await?;
        info!("Tcp socket {} accepted", addr);

        Ok(QuipStream::new(socket))
    }

    fn address(&self) -> QuipResult<SocketAddr> {
        Ok(self.0.local_addr()?)
    }
}
