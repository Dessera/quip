use crate::{
    QuipError, QuipResult,
    io::{
        QuipInput, QuipOutput,
        buffer::{QuipBufReader, QuipBufWriter},
    },
    request::RequestBody,
    response::{Response, ResponseError},
    server::{backend::Backend, service::login, user::User},
};

/// Serve entry for unauthenticated connection, which waits for `Login` command
/// and go to next step.
pub async fn serve<S, R, W>(
    server: &S,
    reader: &mut QuipBufReader<R>,
    writer: &mut QuipBufWriter<W>,
) -> QuipResult<User>
where
    S: Backend,
    R: QuipInput,
    W: QuipOutput,
{
    let (name, resp) = loop {
        let request = match reader.read_request().await {
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
            RequestBody::Nop => Response::success(Some(request.tag), None),
            _ => Response::error(Some(request.tag), ResponseError::Unauthorized),
        };

        writer.write_response(resp).await?;
    };

    let user = server.find_user(&name).await?;
    user.push_resp(resp).await;

    Ok(user)
}
