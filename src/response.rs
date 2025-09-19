use crate::request::Request;
use std::fmt;

/// Error type of response.
///
/// All errors returned by server should have the following format:
///
/// ```plaintext
/// <TAG> Error <CODE>
/// ```
///
/// # Examples:
///
/// ```plaintext
/// A000 Error BadCommand
/// A001 Error Unauthorized
/// ```
#[derive(Debug)]
pub enum ResponseError {
    BadCommand,
    Unauthorized,
    Duplicate,
    NotFound,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ResponseError::BadCommand => "BadCommand",
            ResponseError::Unauthorized => "Unauthorized",
            ResponseError::Duplicate => "Duplicate",
            ResponseError::NotFound => "NotFound",
        })
    }
}

/// General response body.
///
/// A general response body may be one of the following responses:
///
/// - `Success`: Command was processed successfully, i.e. `<TAG> Success <OPTIONAL STRING>`.
/// - `Error`: Error occurred when peocessing command, i.e. `<TAG> Error <CODE>`.
/// - `Recv`: Received command from other users, i.e. `* Recv <MSG>`.
#[derive(Debug)]
pub enum ResponseBody {
    Success(Option<String>),
    Error(ResponseError),
    Recv(String, String),
}

/// General response, with optional request info.
#[derive(Debug)]
pub struct Response {
    pub request: Option<Request>,
    pub body: ResponseBody,
}

impl Response {
    pub fn new(request: Option<Request>, body: ResponseBody) -> Self {
        Self { request, body }
    }

    pub fn success(request: Option<Request>, msg: Option<String>) -> Self {
        Response::new(request, ResponseBody::Success(msg))
    }

    pub fn error(request: Option<Request>, err: ResponseError) -> Self {
        Response::new(request, ResponseBody::Error(err))
    }

    pub fn recv(
        request: Option<Request>,
        sender: impl Into<String>,
        msg: impl Into<String>,
    ) -> Self {
        Response::new(request, ResponseBody::Recv(sender.into(), msg.into()))
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            match &self.request {
                Some(request) => &request.tag,
                None => "*",
            },
            match &self.body {
                ResponseBody::Success(msg) => match msg {
                    Some(msg) => format!("Success {}", msg),
                    None => format!("Success"),
                },
                ResponseBody::Error(msg) => format!("Error {}", msg),
                ResponseBody::Recv(sender, msg) => format!("Recv {} {}", sender, msg),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{Request, RequestBody};

    #[test]
    fn test_response_error_display() {
        assert_eq!(ResponseError::BadCommand.to_string(), "BadCommand");
        assert_eq!(ResponseError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(ResponseError::Duplicate.to_string(), "Duplicate");
        assert_eq!(ResponseError::NotFound.to_string(), "NotFound");
    }

    #[test]
    fn test_response_display_success() {
        let req = Request::new("A000".to_string(), RequestBody::Nop);
        let res = Response::success(Some(req), None);
        assert_eq!(res.to_string(), "A000 Success");

        let req = Request::new("A000".to_string(), RequestBody::Nop);
        let res = Response::success(Some(req), Some("AdditionalInfo".to_string()));
        assert_eq!(res.to_string(), "A000 Success AdditionalInfo");

        let res = Response::success(None, None);
        assert_eq!(res.to_string(), "* Success");
    }

    #[test]
    fn test_response_display_error() {
        let req = Request::new("A000".to_string(), RequestBody::Nop);
        let res = Response::error(Some(req), ResponseError::BadCommand);
        assert_eq!(res.to_string(), "A000 Error BadCommand");

        let res = Response::error(None, ResponseError::Unauthorized);
        assert_eq!(res.to_string(), "* Error Unauthorized");
    }

    #[test]
    fn test_response_display_recv() {
        let req = Request::new("A000".to_string(), RequestBody::Nop);
        let res = Response::recv(Some(req), "Sender", "Message");
        assert_eq!(res.to_string(), "A000 Recv Sender Message");

        let res = Response::recv(None, "Sender", "Message");
        assert_eq!(res.to_string(), "* Recv Sender Message");
    }
}
