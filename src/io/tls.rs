//! TCP wrapper for Quip connection with SSL/TLS support.

use crate::io::{DynamicQuipInput, DynamicQuipOutput, QuipIO};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};
use tokio_native_tls::TlsStream;

/// Quip stream implementation with [`TlsStream`].
///
/// Due to the [`TlsStream`] from [`tokio_native_tls`] does not support the
/// full duplex mode, so we need two sockets for both read and write at the
/// same time.
#[derive(Debug)]
pub struct QuipTlsStream {
    rx: TlsStream<TcpStream>,
    tx: TlsStream<TcpStream>,
}

impl QuipTlsStream {
    pub fn new(rx: TlsStream<TcpStream>, tx: TlsStream<TcpStream>) -> Self {
        Self { rx, tx }
    }
}

impl AsyncRead for QuipTlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let ptr = &mut self.get_mut().rx;
        Pin::new(ptr).poll_read(cx, buf)
    }
}

impl AsyncWrite for QuipTlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let ptr = &mut self.get_mut().tx;
        Pin::new(ptr).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let ptr = &mut self.get_mut().tx;
        Pin::new(ptr).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let ptr = &mut self.get_mut().tx;
        Pin::new(ptr).poll_shutdown(cx)
    }
}

impl QuipIO for QuipTlsStream {
    fn duplex(self: Box<Self>) -> (DynamicQuipInput, DynamicQuipOutput) {
        (Box::new(self.rx), Box::new(self.tx))
    }
}
