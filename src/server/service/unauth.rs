use log::{debug, warn};

use crate::{
    QuipError, QuipResult,
    io::{
        QuipInput, QuipOutput,
        buffer::{QuipBufReader, QuipBufWriter},
    },
    request::RequestBody,
    response::{Response, ResponseBody, ResponseError},
    server::{backend::Backend, connection::ConnectionRef},
};

/// Serve entry for unauthenticated connection, which waits for `Login` command
/// and go to next step.
pub async fn serve<S: Backend, R: QuipInput, W: QuipOutput>(
    server: &S,
    reader: &mut QuipBufReader<R>,
    writer: &mut QuipBufWriter<W>,
) -> QuipResult<ConnectionRef> {
    let (name, resp) = serve_inner(server, reader, writer).await?;
    let conn = server.find_conn(&name).await?;

    {
        let conn = conn.lock().await;
        conn.queue.lock().await.push_back(resp);
        conn.notify.notify_one();
    }

    Ok(conn)
}

async fn serve_inner<S: Backend, R: QuipInput, W: QuipOutput>(
    server: &S,
    reader: &mut QuipBufReader<R>,
    writer: &mut QuipBufWriter<W>,
) -> QuipResult<(String, Response)> {
    Ok(loop {
        let resp = match reader.read_request().await {
            Ok(request) => {
                let body = match request.body {
                    RequestBody::Login(name, password) => {
                        let body = serve_login(server, &name, &password).await?;

                        match body {
                            ResponseBody::Success(_) => {
                                break (name, Response::new(Some(request.tag), body));
                            }
                            _ => body,
                        }
                    }
                    RequestBody::Logout => return Err(QuipError::Disconnect),
                    RequestBody::Nop => ResponseBody::Success(None),
                    _ => ResponseBody::Error(ResponseError::Unauthorized),
                };

                debug!("Unknown: {}", request.tag);
                Response::new(Some(request.tag), body)
            }
            Err(QuipError::Parse(msg)) => {
                warn!("Unknown: {}", msg);
                Response::error(None, ResponseError::BadCommand)
            }
            Err(err) => return Err(err), // Unexpected
        };

        writer.write_response(resp).await?;
    })
}

/// Serve `Login` command.
async fn serve_login<S: Backend>(
    server: &S,
    name: &str,
    password: &str,
) -> QuipResult<ResponseBody> {
    let resp = match server.load_conn(name, password).await {
        Ok(_) => ResponseBody::Success(Some(name.to_string())),
        Err(QuipError::Duplicate(_)) => ResponseBody::Error(ResponseError::Duplicate),
        Err(QuipError::NotFound(_)) => ResponseBody::Error(ResponseError::NotFound),
        Err(QuipError::Authorize(_)) => ResponseBody::Error(ResponseError::Unauthorized),
        Err(err) => return Err(err),
    };

    Ok(resp)
}
