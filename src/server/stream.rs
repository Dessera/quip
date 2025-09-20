use crate::{QuipError, QuipResult, request::Request, response::Response};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, ReadBuf,
};

/// Handle for async reade half.
pub trait AsyncReadHalf: AsyncRead + Send + Unpin {}

impl<T> AsyncReadHalf for T where T: AsyncRead + Send + Unpin {}

/// Handle for async write half.
pub trait AsyncWriteHalf: AsyncWrite + Send + Unpin {}

impl<T> AsyncWriteHalf for T where T: AsyncWrite + Send + Unpin {}

/// Handle for async read and write half.
pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin {
    /// Extra util to support full duplex of io stream.
    ///
    /// Unfortunately, both `tokio-native-tls` and [`tokio-rustls`] do not
    /// support it. So this part of the design has no effect :(
    fn duplex(self: Box<Self>) -> (Box<dyn AsyncReadHalf>, Box<dyn AsyncWriteHalf>);
}

/// Dynamic wrapper for any async stream.
pub struct QuipStream(Box<dyn AsyncStream>);

impl QuipStream {
    pub fn new<T: AsyncStream + 'static>(socket: T) -> Self {
        Self(Box::new(socket))
    }

    #[inline]
    pub fn duplex(self) -> (Box<dyn AsyncReadHalf>, Box<dyn AsyncWriteHalf>) {
        self.0.duplex()
    }
}

impl AsyncRead for QuipStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let ptr = &mut self.get_mut().0;
        Pin::new(ptr).poll_read(cx, buf)
    }
}

impl AsyncWrite for QuipStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let ptr = &mut self.get_mut().0;
        Pin::new(ptr).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let ptr = &mut self.get_mut().0;
        Pin::new(ptr).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let ptr = &mut self.get_mut().0;
        Pin::new(ptr).poll_shutdown(cx)
    }
}

/// Reader for read [`Request`] from any IO port.
pub struct QuipBufReader<R>(BufReader<R>);

impl<R> QuipBufReader<R>
where
    R: AsyncRead,
{
    pub fn new(socket: R) -> Self {
        Self(BufReader::new(socket))
    }
}

impl<R> QuipBufReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Get [`Request`] from socket, terminate with `\n`.
    pub async fn get_request(&mut self) -> QuipResult<Request> {
        let mut buffer = String::new();
        let mut zero_cnt: usize = 0;
        loop {
            match self.0.read_line(&mut buffer).await? {
                0 => {
                    zero_cnt += 1;
                    match zero_cnt {
                        1 => continue,
                        _ => return Err(QuipError::Disconnect),
                    }
                }
                _ => break,
            };
        }

        Request::try_from(buffer)
    }
}

/// Writer for write [`Response`] to any IO port.
pub struct QuipBufWriter<W>(BufWriter<W>);

impl<W> QuipBufWriter<W>
where
    W: AsyncWrite,
{
    pub fn new(socket: W) -> Self {
        Self(BufWriter::new(socket))
    }
}

impl<W> QuipBufWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Write [`Response`] to socket, end with `\n`.
    pub async fn write_response(&mut self, resp: Response) -> QuipResult<()> {
        self.0.write_all(resp.to_string().as_bytes()).await?;
        self.0.write_all("\n".as_bytes()).await?;
        Ok(self.0.flush().await?)
    }
}
