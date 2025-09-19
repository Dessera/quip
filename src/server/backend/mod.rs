pub mod simple;

pub use self::simple::*;
use crate::{
    TcResult,
    server::{connection::Connection, user::User},
};
use std::{future::Future, net::SocketAddr};

/// Server backend interface.
pub trait Backend {
    /// Accept a connection from any source.
    fn accept(&self) -> impl Future<Output = TcResult<Connection>> + Send;

    /// Serve a connection.
    fn serve(&self, conn: Connection) -> impl Future<Output = TcResult<()>> + Send;

    /// Get server address.
    fn address(&self) -> TcResult<SocketAddr>;

    fn add_user(&self, user: User) -> impl Future<Output = TcResult<()>> + Send;
    fn remove_user(&self, user: User) -> impl Future<Output = TcResult<()>> + Send;
    fn rename_user(&self, original: &str, name: &str) -> impl Future<Output = TcResult<()>> + Send;
    fn find_user(&self, name: &str) -> impl Future<Output = TcResult<User>> + Send;
}
