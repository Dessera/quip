use crate::{
    QuipResult,
    server::{
        listener::Listener,
        stream::{AsyncReadHalf, AsyncStream, AsyncWriteHalf, QuipStream},
    },
};
use log::info;
use native_tls::{Identity, TlsAcceptor as NativeTlsAcceptor};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_native_tls::{TlsAcceptor, TlsStream};

impl AsyncStream for TlsStream<TcpStream> {
    // ! Unstable implementation !
    fn duplex(self: Box<Self>) -> (Box<dyn AsyncReadHalf>, Box<dyn AsyncWriteHalf>) {
        let (rx, tx) = tokio::io::split(self);
        (Box::new(rx), Box::new(tx))
    }
}

/// Wrapper of [`TlsAcceptor`].
pub struct TlsListener {
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsListener {
    pub async fn bind<T: ToSocketAddrs>(addr: T, identity: Identity) -> QuipResult<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr).await?,
            acceptor: TlsAcceptor::from(NativeTlsAcceptor::builder(identity).build()?),
        })
    }
}

impl Listener for TlsListener {
    async fn accept(&self) -> QuipResult<QuipStream> {
        let (socket, addr) = self.listener.accept().await?;
        let tls_stream = self.acceptor.accept(socket).await?;

        info!("TLS socket {} accepted", addr);

        Ok(QuipStream::new(tls_stream))
    }

    fn address(&self) -> QuipResult<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }
}
