use crate::{QuipError, QuipResult, request::Request, token::detokenize};
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

impl TryFrom<String> for ResponseError {
    type Error = QuipError;

    fn try_from(value: String) -> QuipResult<Self> {
        ResponseError::try_from(value.as_str())
    }
}

impl TryFrom<&str> for ResponseError {
    type Error = QuipError;

    fn try_from(value: &str) -> QuipResult<Self> {
        let err = match value {
            "BadCommand" => ResponseError::BadCommand,
            "Unauthorized" => ResponseError::Unauthorized,
            "Duplicate" => ResponseError::Duplicate,
            "NotFound" => ResponseError::NotFound,
            _ => {
                return Err(QuipError::Parse(format!(
                    "{} is not a valid ResponseError",
                    value
                )));
            }
        };

        Ok(err)
    }
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
/// - `Recv`: Received command from other users, i.e. `* Recv (<GROUP>:)<USER> <MSG>`.
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
        let tag = match &self.request {
            Some(request) => &request.tag,
            None => "*",
        }
        .to_string();

        let tokens = match &self.body {
            ResponseBody::Success(msg) => match msg {
                Some(msg) => vec![tag, "Success".to_string(), msg.clone()],
                None => vec![tag, "Success".to_string()],
            },
            ResponseBody::Error(msg) => vec![tag, "Error".to_string(), msg.to_string()],
            ResponseBody::Recv(name, msg) => {
                vec![tag, "Recv".to_string(), name.clone(), msg.clone()]
            }
        };

        f.write_str(detokenize(&tokens).as_str())
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
        let req = Request::new("A000", RequestBody::Nop);
        let res = Response::success(Some(req), None);
        assert_eq!(res.to_string(), "A000 Success");

        let req = Request::new("A000", RequestBody::Nop);
        let res = Response::success(Some(req), Some("AdditionalInfo".to_string()));
        assert_eq!(res.to_string(), "A000 Success AdditionalInfo");

        let res = Response::success(None, None);
        assert_eq!(res.to_string(), "* Success");
    }

    #[test]
    fn test_response_display_error() {
        let req = Request::new("A000", RequestBody::Nop);
        let res = Response::error(Some(req), ResponseError::BadCommand);
        assert_eq!(res.to_string(), "A000 Error BadCommand");

        let res = Response::error(None, ResponseError::Unauthorized);
        assert_eq!(res.to_string(), "* Error Unauthorized");
    }

    #[test]
    fn test_response_display_recv() {
        let req = Request::new("A000", RequestBody::Nop);
        let res = Response::recv(Some(req), "Sender", "Message");
        assert_eq!(res.to_string(), "A000 Recv Sender Message");

        let res = Response::recv(None, "Sender", "Message");
        assert_eq!(res.to_string(), "* Recv Sender Message");

        let res = Response::recv(None, "Sender", "Complex  Message");
        assert_eq!(res.to_string(), "* Recv Sender \"Complex  Message\"");
    }
}
