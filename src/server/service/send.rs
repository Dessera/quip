use crate::{
    TcResult,
    request::{Request, RequestBody},
    response::{Response, ResponseError},
    server::{backend::Backend, user::User},
};

/// Serve `Send` command.
pub async fn serve<S>(server: &S, request: Request, user: &User) -> TcResult<Response>
where
    S: Backend,
{
    let (receiver, msg) = match &request.body {
        RequestBody::Send(receiver, msg) => (receiver.clone(), msg.clone()),
        _ => return Ok(Response::error(Some(request), ResponseError::BadCommand)),
    };

    let sender = {
        let user_data = user.data.lock().await;
        user_data.name.clone()
    };

    let target = match server.find_user(receiver.as_str()).await {
        Ok(target) => target,
        Err(_) => return Ok(Response::error(Some(request), ResponseError::NotFound)),
    };

    target.push_resp(Response::recv(None, sender, msg)).await;

    Ok(Response::success(Some(request), Some(receiver)))
}
