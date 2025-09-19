use crate::{
    TcError, TcResult,
    request::RequestBody,
    response::{Response, ResponseError},
    server::{
        backend::Backend,
        connection::{ConnectionReader, ConnectionWriter},
        service::{login, send},
        user::User,
    },
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

pub async fn serve_write<S, W>(
    _server: &S,
    user: User,
    writer: &mut ConnectionWriter<W>,
) -> TcResult<()>
where
    S: Backend,
    W: AsyncWriteExt + Unpin,
{
    loop {
        user.notify.notified().await;
        user.write_all(writer).await?;
    }
}

pub async fn serve_read<S, R>(
    server: &S,
    user: User,
    reader: &mut ConnectionReader<R>,
) -> TcResult<()>
where
    S: Backend,
    R: AsyncBufReadExt + Unpin,
{
    loop {
        let request = {
            match reader.get_request().await {
                Ok(request) => request,
                Err(TcError::Parse(_)) => {
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
            RequestBody::Logout => return Err(TcError::Disconnect),
            RequestBody::Nop => Response::success(Some(request), None),
        };

        user.push_resp(resp).await;
    }
}
