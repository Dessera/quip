use crate::{
    QuipError, QuipResult,
    request::RequestBody,
    response::{Response, ResponseError},
    server::{
        backend::Backend,
        connection::{ConnectionReader, ConnectionWriter},
        service::login,
        user::User,
    },
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

/// Serve entry for unauthenticated connection, which waits for `Login` command
/// and go to next step.
pub async fn serve<S, R, W>(
    server: &S,
    reader: &mut ConnectionReader<R>,
    writer: &mut ConnectionWriter<W>,
) -> QuipResult<User>
where
    S: Backend,
    R: AsyncBufReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let (name, resp) = loop {
        let request = match reader.get_request().await {
            Ok(request) => request,
            Err(QuipError::Parse(_)) => {
                writer
                    .write_response(Response::error(None, ResponseError::BadCommand))
                    .await?;
                continue;
            }
            Err(err) => return Err(err),
        };

        let resp = match &request.body {
            RequestBody::Login(name) | RequestBody::SetName(name) => {
                let name = name.clone();
                let resp = login::serve_unauth(server, request).await?;
                match &resp.body {
                    crate::response::ResponseBody::Success(_) => break (name, resp),
                    _ => resp,
                }
            }
            RequestBody::Logout => return Err(QuipError::Disconnect),
            RequestBody::Nop => Response::success(Some(request), None),
            _ => Response::error(Some(request), ResponseError::Unauthorized),
        };

        writer.write_response(resp).await?;
    };

    let user = server.find_user(&name).await?;
    user.push_resp(resp).await;

    Ok(user)
}
