use crate::{
    QuipError, QuipResult,
    io::{
        QuipInput, QuipOutput,
        buffer::{QuipBufReader, QuipBufWriter},
    },
    request::RequestBody,
    response::{Response, ResponseError},
    server::{
        backend::Backend,
        service::{login, send},
        user::User,
    },
};

/// Write task for a connection.
///
/// All responses should be written in this, otherwise client may not be able
/// to process the response correctly.
pub async fn serve_write<S, W>(
    _server: &S,
    user: User,
    writer: &mut QuipBufWriter<W>,
) -> QuipResult<()>
where
    S: Backend,
    W: QuipOutput,
{
    loop {
        user.notify.notified().await;
        user.write_all(writer).await?;
    }
}

/// Read task for a connection.
///
/// This task reads and parse all requests and push responses to write task.
pub async fn serve_read<S, R>(
    server: &S,
    user: User,
    reader: &mut QuipBufReader<R>,
) -> QuipResult<()>
where
    S: Backend,
    R: QuipInput,
{
    loop {
        let request = {
            match reader.read_request().await {
                Ok(request) => request,
                Err(QuipError::Parse(_)) => {
                    user.push_resp(Response::error(None, ResponseError::BadCommand))
                        .await;
                    continue;
                }
                Err(err) => return Err(err),
            }
        };

        let resp = match &request.body {
            RequestBody::Send(_, _) => send::serve(server, request, &user).await?,
            RequestBody::Login(_) | RequestBody::SetName(_) => {
                login::serve(server, request, &user).await?
            }
            RequestBody::Logout => return Err(QuipError::Disconnect),
            RequestBody::Nop => Response::success(Some(request), None),
        };

        user.push_resp(resp).await;
    }
}
