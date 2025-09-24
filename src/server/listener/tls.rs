//! Quip listener based on TCP stream with SSL/TLS.

use crate::{
    QuipResult,
    io::{DynamicQuipIO, tls::QuipTlsStream},
    server::listener::Listener,
};
use log::info;
use native_tls::{Identity, TlsAcceptor as NativeTlsAcceptor};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_native_tls::{TlsAcceptor, TlsStream};

/// Wrapper of [`TlsAcceptor`].
pub struct TlsListener {
    rx_listener: TcpListener,
    tx_listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsListener {
    pub async fn bind(
        read_addr: impl ToSocketAddrs,
        write_addr: impl ToSocketAddrs,
        identity: Identity,
    ) -> QuipResult<Self> {
        Ok(Self {
            rx_listener: TcpListener::bind(read_addr).await?,
            tx_listener: TcpListener::bind(write_addr).await?,
            acceptor: TlsAcceptor::from(NativeTlsAcceptor::builder(identity).build()?),
        })
    }

    pub(self) async fn accept_tls(
        &self,
        listener: &TcpListener,
    ) -> QuipResult<(TlsStream<TcpStream>, SocketAddr)> {
        let (socket, addr) = listener.accept().await?;
        Ok((self.acceptor.accept(socket).await?, addr))
    }
}

impl Listener for TlsListener {
    async fn accept(&self) -> QuipResult<DynamicQuipIO> {
        let ((rx, rx_addr), (tx, tx_addr)) = tokio::try_join!(
            self.accept_tls(&self.rx_listener),
            self.accept_tls(&self.tx_listener)
        )?;

        info!(
            "SSL/TLS socket accepted, read {}, write {}",
            rx_addr, tx_addr
        );

        Ok(Box::new(QuipTlsStream::new(rx, tx)))
    }

    fn address(&self) -> QuipResult<SocketAddr> {
        Ok(self.rx_listener.local_addr()?) // TODO: rx and tx
    }
}
