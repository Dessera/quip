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
        write!(
            f,
            "{}",
            match self {
                ResponseError::BadCommand => "BadCommand",
                ResponseError::Unauthorized => "Unauthorized",
                ResponseError::Duplicate => "Duplicate",
                ResponseError::NotFound => "NotFound",
            }
        )
    }
}

/// Generic response body.
///
/// A generic response body may be one of the following responses:
///
/// - `Success`: Command was processed successfully, usually likes `<TAG> Success <OPTIONAL STRING>`
/// - `Error`: Error occurred when peocessing command, usually likes `<TAG> Error <CODE>`
/// - `Recv`: Received command from other users, usually likes `* Recv <MSG>`
#[derive(Debug)]
pub enum ResponseBody {
    Success(Option<String>),
    Error(ResponseError),
    Recv(String, String),
}

/// Generic response, with optional request info.
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

    pub fn recv(request: Option<Request>, sender: String, msg: String) -> Self {
        Response::new(request, ResponseBody::Recv(sender, msg))
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            match &self.request {
                Some(request) => request.tag.as_str(),
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
