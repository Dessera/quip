mod auth;
mod unauth;

use crate::{
    QuipError, QuipResult,
    io::{
        DynamicQuipIO, QuipInput, QuipOutput,
        buffer::{QuipBufReader, QuipBufWriter},
    },
    server::backend::Backend,
};
use log::info;

/// General serve entry, which represents the entire lifetime of a connection.
pub async fn serve<S: Backend>(server: &S, conn: DynamicQuipIO) -> QuipResult<()> {
    let (rx, tx) = {
        let conns = conn.duplex();
        (QuipBufReader::new(conns.0), QuipBufWriter::new(conns.1))
    };

    match serve_inner(server, rx, tx).await {
        Ok(_) | Err(QuipError::Disconnect) => Ok(()),
        Err(err) => Err(err),
    }
}

async fn serve_inner<S: Backend, R: QuipInput, W: QuipOutput>(
    server: &S,
    mut rx: QuipBufReader<R>,
    mut tx: QuipBufWriter<W>,
) -> QuipResult<()> {
    let conn = unauth::serve(server, &mut rx, &mut tx).await?;
    let conn_name = {
        let conn = conn.lock().await;
        conn.name.clone()
    };

    info!("User {} login", conn_name);

    // TODO: Use flag rather than `try_join`.
    let res = tokio::try_join!(
        auth::serve_read(server, conn.clone(), &mut rx),
        auth::serve_write(server, conn.clone(), &mut tx)
    );

    server.unload_conn(&conn_name).await?;

    info!("User {} logout", conn_name);

    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}
