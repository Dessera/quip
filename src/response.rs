use crate::request::Request;
use std::fmt;

#[derive(Debug)]
pub enum ResponseError {
    BadCommand,
    Unauthorized,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ResponseError::BadCommand => "BadCommand",
                ResponseError::Unauthorized => "Unauthorized",
            }
        )
    }
}

#[derive(Debug)]
pub enum ResponseBody {
    Success(String),
    Error(ResponseError),
}

#[derive(Debug)]
pub struct Response {
    pub request: Option<Request>,
    pub body: ResponseBody,
}

impl Response {
    pub fn new(request: Option<Request>, body: ResponseBody) -> Self {
        Self { request, body }
    }

    pub fn success(request: Option<Request>, msg: String) -> Self {
        Response::new(request, ResponseBody::Success(msg))
    }

    pub fn error(request: Option<Request>, err: ResponseError) -> Self {
        Response::new(request, ResponseBody::Error(err))
    }
}

impl ToString for Response {
    fn to_string(&self) -> String {
        match &self.body {
            ResponseBody::Success(msg) => format!("Success {}", msg),
            ResponseBody::Error(msg) => format!("Error {}", msg),
        }
    }
}
