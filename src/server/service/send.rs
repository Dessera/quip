use crate::{
    QuipResult,
    request::{Request, RequestBody},
    response::{Response, ResponseError},
    server::{backend::Backend, user::User},
};

/// Serve `Send` command.
pub async fn serve<S>(server: &S, request: Request, user: &User) -> QuipResult<Response>
where
    S: Backend,
{
    let (receiver, msg) = match &request.body {
        RequestBody::Send(receiver, msg) => (receiver.clone(), msg.clone()),
        _ => {
            return Ok(Response::error(
                Some(request.tag),
                ResponseError::BadCommand,
            ));
        }
    };

    let sender = {
        let user_data = user.data.lock().await;
        user_data.name.clone()
    };

    let target = match server.ensure_user(&receiver).await {
        Ok(target) => target,
        Err(_) => return Ok(Response::error(Some(request.tag), ResponseError::NotFound)),
    };

    target.push_resp(Response::recv(None, sender, msg)).await;

    Ok(Response::success(Some(request.tag), Some(receiver)))
}
