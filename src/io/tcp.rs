//! TCP wrapper for Quip connection.

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

/// Quip stream implementation with [`TcpStream`].
#[derive(Debug)]
pub struct QuipTcpStream {
    io: TcpStream,
}

impl QuipTcpStream {
    pub fn new(io: TcpStream) -> Self {
        Self { io }
    }
}

impl AsyncRead for QuipTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let ptr = &mut self.get_mut().io;
        Pin::new(ptr).poll_read(cx, buf)
    }
}

impl AsyncWrite for QuipTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let ptr = &mut self.get_mut().io;
        Pin::new(ptr).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let ptr = &mut self.get_mut().io;
        Pin::new(ptr).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let ptr = &mut self.get_mut().io;
        Pin::new(ptr).poll_shutdown(cx)
    }
}

impl QuipIO for QuipTcpStream {
    fn duplex(self: Box<Self>) -> (DynamicQuipInput, DynamicQuipOutput) {
        let (rx, tx) = self.io.into_split();
        (Box::new(rx), Box::new(tx))
    }
}
