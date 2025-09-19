pub mod auth;
pub mod login;
pub mod send;
pub mod unauth;

use crate::{
    TcError, TcResult,
    server::{
        backend::Backend,
        connection::{Connection, ConnectionReader, ConnectionWriter},
    },
};

/// General serve entry, which represents the entire lifetime of a connection.
pub async fn serve<S: Backend>(server: &S, conn: Connection) -> TcResult<()> {
    let (mut rx, mut tx) = conn.socket.into_split();

    let mut rx = ConnectionReader::from_read(&mut rx);
    let mut tx = ConnectionWriter::from_write(&mut tx);

    let user = match unauth::serve(server, &mut rx, &mut tx).await {
        Ok(user) => user,
        Err(TcError::Disconnect) => return Ok(()),
        Err(err) => return Err(err),
    };

    let res = tokio::try_join!(
        auth::serve_read(server, user.clone(), &mut rx),
        auth::serve_write(server, user.clone(), &mut tx)
    );

    server.remove_user(user).await?;

    match res {
        Ok(_) | Err(TcError::Disconnect) => Ok(()),
        Err(err) => Err(err),
    }
}
