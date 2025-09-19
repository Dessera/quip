use crate::{
    TcError, TcResult,
    request::{Request, RequestBody},
    response::{Response, ResponseError},
    server::{backend::Backend, user::User},
};

/// Serve `Login` command when not authenticated.
///
/// If a connection is not authenticated, `Login` command will create a new
/// [`User`] for it.
pub async fn serve_unauth<S>(server: &S, request: Request) -> TcResult<Response>
where
    S: Backend,
{
    let name = match &request.body {
        RequestBody::Login(name) | RequestBody::SetName(name) => name.clone(),
        _ => return Ok(Response::error(Some(request), ResponseError::BadCommand)),
    };

    let resp = match server.add_user(User::new(name.clone())).await {
        Ok(_) => Response::success(Some(request), Some(name)),
        Err(TcError::Duplicate(_)) => Response::error(Some(request), ResponseError::Duplicate),
        Err(err) => return Err(err),
    };

    Ok(resp)
}

/// Serve `Login` command when authenticated.
///
/// If a connection is authenticated, `Login` command will change its name.
pub async fn serve<S>(server: &S, request: Request, user: &User) -> TcResult<Response>
where
    S: Backend,
{
    let name = match &request.body {
        RequestBody::Login(name) | RequestBody::SetName(name) => name.clone(),
        _ => return Ok(Response::error(Some(request), ResponseError::BadCommand)),
    };

    let mut user_data = user.data.lock().await;

    if user_data.name == name {
        return Ok(Response::success(Some(request), Some(name)));
    }

    let resp = match server.rename_user(&user_data.name, &name).await {
        Ok(_) => {
            user_data.name = name.clone();
            Response::success(Some(request), Some(name))
        }
        Err(TcError::Duplicate(_)) => Response::error(Some(request), ResponseError::Duplicate),
        Err(TcError::NotFound(_)) => Response::error(Some(request), ResponseError::NotFound),
        Err(err) => return Err(err),
    };

    Ok(resp)
}
