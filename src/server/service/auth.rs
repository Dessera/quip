use crate::{
    QuipError, QuipResult,
    io::{
        QuipInput, QuipOutput,
        buffer::{QuipBufReader, QuipBufWriter},
    },
    request::RequestBody,
    response::{Response, ResponseBody, ResponseError},
    server::{
        backend::Backend,
        connection::{ConnectionRef, ConnectionStatus},
    },
};
use log::{debug, warn};

/// Write task for a connection.
///
/// All responses should be written in this, otherwise client may not be able
/// to process the response correctly.
pub async fn serve_write<S: Backend, W: QuipOutput>(
    _server: &S,
    conn: ConnectionRef,
    writer: &mut QuipBufWriter<W>,
) -> QuipResult<()> {
    let (notify, queue, name) = {
        let conn = conn.lock().await;
        (conn.notify.clone(), conn.queue.clone(), conn.name.clone())
    };

    loop {
        notify.notified().await;

        let mut queue = queue.lock().await;
        let mut cnt: usize = 0;
        while !queue.is_empty() {
            let resp = match queue.pop_front() {
                Some(resp) => resp,
                None => continue,
            };

            writer.write_response(resp).await?;
            cnt += 1;
        }

        debug!("Sync {} message to user {}", cnt, name);
    }
}

/// Read task for a connection.
///
/// This task reads and parse all requests and push responses to write task.
pub async fn serve_read<S: Backend, R: QuipInput>(
    server: &S,
    conn: ConnectionRef,
    reader: &mut QuipBufReader<R>,
) -> QuipResult<()> {
    let (notify, queue, name) = {
        let conn = conn.lock().await;
        (conn.notify.clone(), conn.queue.clone(), conn.name.clone())
    };

    loop {
        let resp = match reader.read_request().await {
            Ok(request) => {
                let body = match request.body {
                    RequestBody::Send(name, msg) => serve_send(server, &conn, name, msg).await?,
                    RequestBody::Login(_, _) => ResponseBody::Error(ResponseError::Authorized),
                    RequestBody::Logout => return Err(QuipError::Disconnect),
                    RequestBody::Nop => ResponseBody::Success(None),
                };

                debug!("{}: {}", name, request.tag);
                Response::new(Some(request.tag), body)
            }
            Err(QuipError::Parse(msg)) => {
                warn!("{}: {}", name, msg);
                Response::error(None, ResponseError::BadCommand)
            }
            Err(err) => return Err(err),
        };

        queue.lock().await.push_back(resp);
        notify.notify_one();
    }
}

/// Serve `Send` command.
async fn serve_send<S: Backend>(
    server: &S,
    conn: &ConnectionRef,
    receiver: String,
    msg: String,
) -> QuipResult<ResponseBody> {
    let sender = {
        let conn = conn.lock().await;
        conn.name.clone()
    };

    let recv_conn = match server.ensure_conn(&receiver).await {
        Ok(target) => target,
        Err(_) => return Ok(ResponseBody::Error(ResponseError::NotFound)),
    };

    let recv_conn = recv_conn.lock().await;

    recv_conn
        .queue
        .lock()
        .await
        .push_back(Response::recv(None, sender, msg));

    if recv_conn.status != ConnectionStatus::Cache {
        recv_conn.notify.notify_one();
    }

    Ok(ResponseBody::Success(Some(receiver)))
}
