//! Buffered IO utils.

use crate::{
    QuipError, QuipResult,
    io::{QuipInput, QuipOutput},
    request::Request,
    response::Response,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

/// Reader for read [`Request`] from any [`QuipInput`].
pub struct QuipBufReader<R>(BufReader<R>);

impl<R> QuipBufReader<R>
where
    R: QuipInput,
{
    pub fn new(socket: R) -> Self {
        Self(BufReader::new(socket))
    }

    /// Get [`Request`] from socket, terminate with `\n`.
    pub async fn read_request(&mut self) -> QuipResult<Request> {
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

/// Writer for write [`Response`] to any [`QuipOutput`].
pub struct QuipBufWriter<W>(BufWriter<W>);

impl<W> QuipBufWriter<W>
where
    W: QuipOutput,
{
    pub fn new(socket: W) -> Self {
        Self(BufWriter::new(socket))
    }

    /// Write [`Response`] to socket, end with `\n`.
    pub async fn write_response(&mut self, resp: Response) -> QuipResult<()> {
        self.0.write_all(resp.to_string().as_bytes()).await?;
        self.0.write_all("\n".as_bytes()).await?;
        Ok(self.0.flush().await?)
    }
}
