pub mod simple;

pub use self::simple::*;
use crate::{TcResult, server::connection::Connection};
use std::{future::Future, net::SocketAddr};

pub trait Backend {
    fn accept(&self) -> impl Future<Output = TcResult<Connection>> + Send;
    fn serve(&self, conn: Connection) -> impl Future<Output = TcResult<()>> + Send;
    fn address(&self) -> TcResult<SocketAddr>;
}
