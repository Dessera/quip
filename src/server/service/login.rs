use crate::{
    QuipError, QuipResult,
    request::{Request, RequestBody},
    response::{Response, ResponseError},
    server::{backend::Backend, user::User},
};

/// Serve `Login` command when not authenticated.
///
/// If a connection is not authenticated, `Login` command will create a new
/// [`User`] for it.
pub async fn serve_unauth<S>(server: &S, request: Request) -> QuipResult<Response>
where
    S: Backend,
{
    let name = match &request.body {
        RequestBody::Login(name) | RequestBody::SetName(name) => name.clone(),
        _ => {
            return Ok(Response::error(
                Some(request.tag),
                ResponseError::BadCommand,
            ));
        }
    };

    let resp = match server.add_user(&name).await {
        Ok(_) => Response::success(Some(request.tag), Some(name)),
        Err(QuipError::Duplicate(_)) => {
            Response::error(Some(request.tag), ResponseError::Duplicate)
        }
        Err(err) => return Err(err),
    };

    Ok(resp)
}

/// Serve `Login` command when authenticated.
///
/// If a connection is authenticated, `Login` command will change its name.
pub async fn serve<S>(server: &S, request: Request, user: &User) -> QuipResult<Response>
where
    S: Backend,
{
    let name = match &request.body {
        RequestBody::Login(name) | RequestBody::SetName(name) => name.clone(),
        _ => {
            return Ok(Response::error(
                Some(request.tag),
                ResponseError::BadCommand,
            ));
        }
    };

    let mut user_data = user.data.lock().await;

    if user_data.name == name {
        return Ok(Response::success(Some(request.tag), Some(name)));
    }

    let resp = match server.rename_user(&user_data.name, &name).await {
        Ok(_) => {
            user_data.name = name.clone();
            Response::success(Some(request.tag), Some(name))
        }
        Err(QuipError::Duplicate(_)) => {
            Response::error(Some(request.tag), ResponseError::Duplicate)
        }
        Err(QuipError::NotFound(_)) => Response::error(Some(request.tag), ResponseError::NotFound),
        Err(err) => return Err(err),
    };

    Ok(resp)
}
