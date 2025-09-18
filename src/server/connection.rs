use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{TcError, TcResult, request::Request, response::Response};

pub struct Connection {
    pub socket: TcpStream,
    pub addr: SocketAddr,
}

impl Connection {
    pub fn new(socket: TcpStream, addr: SocketAddr) -> Self {
        Self { socket, addr }
    }
}

pub struct ConnectionStream<RW> {
    stream: RW,
}

impl<RW> ConnectionStream<RW> {
    pub fn new(stream: RW) -> Self {
        Self { stream: stream }
    }
}

impl<RW: AsyncBufReadExt + Unpin> ConnectionStream<RW> {
    pub async fn get_request(&mut self) -> TcResult<Request> {
        let mut buffer = String::new();
        let mut zero_cnt: usize = 0;
        loop {
            match self.stream.read_line(&mut buffer).await? {
                0 => {
                    zero_cnt += 1;
                    match zero_cnt {
                        1 => continue,
                        _ => return Err(TcError::Disconnect),
                    }
                }
                _ => break,
            };
        }

        Request::try_from(buffer.as_str())
    }
}

impl<RW: AsyncWriteExt + Unpin> ConnectionStream<RW> {
    pub async fn write_response(&mut self, resp: Response) -> TcResult<()> {
        self.stream.write_all(resp.to_string().as_bytes()).await?;
        self.stream.write_all("\n".as_bytes()).await?;
        Ok(self.stream.flush().await?)
    }
}
