use crate::{QuipError, QuipResult, request::Request, response::Response};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufStream, BufWriter},
    net::TcpStream,
};

/// Wrapper for real socket.
///
/// TODO: Support more protocol?
pub struct Connection {
    pub socket: TcpStream,
    pub addr: SocketAddr,
}

impl Connection {
    pub fn new(socket: TcpStream, addr: SocketAddr) -> Self {
        Self { socket, addr }
    }
}

/// Stream for read [`Request`] and write [`Response`] with any IO port.
pub struct ConnectionStream<RW> {
    stream: RW,
}

/// Reader for read [`Request`] from any IO port.
///
/// Actually, there is no difference between [`ConnectionStream`] and
/// [`ConnectionReader`]
pub type ConnectionReader<R> = ConnectionStream<R>;

/// Reader for write [`Response`] to any IO port.
///
/// Actually, there is no difference between [`ConnectionStream`] and
/// [`ConnectionWriter`]
pub type ConnectionWriter<W> = ConnectionStream<W>;

impl<RW> ConnectionStream<RW> {
    pub fn new(stream: RW) -> Self {
        Self { stream: stream }
    }
}

impl<RW> From<RW> for ConnectionStream<BufStream<RW>>
where
    RW: AsyncRead + AsyncWrite,
{
    fn from(value: RW) -> Self {
        Self::new(BufStream::new(value))
    }
}

impl<R> ConnectionReader<BufReader<R>>
where
    R: AsyncRead,
{
    pub fn from_read(value: R) -> Self {
        Self::new(BufReader::new(value))
    }
}

impl<W> ConnectionWriter<BufWriter<W>>
where
    W: AsyncWrite,
{
    pub fn from_write(value: W) -> Self {
        Self::new(BufWriter::new(value))
    }
}

impl<RW> ConnectionStream<RW>
where
    RW: AsyncBufReadExt + Unpin,
{
    /// Get [`Request`] from socket, terminate with `\n`.
    pub async fn get_request(&mut self) -> QuipResult<Request> {
        let mut buffer = String::new();
        let mut zero_cnt: usize = 0;
        loop {
            match self.stream.read_line(&mut buffer).await? {
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

impl<RW> ConnectionStream<RW>
where
    RW: AsyncWriteExt + Unpin,
{
    /// Write [`Response`] to socket, end with `\n`.
    pub async fn write_response(&mut self, resp: Response) -> QuipResult<()> {
        self.stream.write_all(resp.to_string().as_bytes()).await?;
        self.stream.write_all("\n".as_bytes()).await?;
        Ok(self.stream.flush().await?)
    }
}
