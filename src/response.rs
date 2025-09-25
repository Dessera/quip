use crate::{
    QuipError, QuipResult,
    token::{detokenize, tokenize},
    unwrap_token,
};
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
#[derive(Debug, PartialEq, Eq)]
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
    pub tag: Option<String>,
    pub body: ResponseBody,
}

impl Response {
    pub fn new(tag: Option<String>, body: ResponseBody) -> Self {
        Self { tag, body }
    }

    pub fn success(tag: Option<String>, msg: Option<String>) -> Self {
        Response::new(tag, ResponseBody::Success(msg))
    }

    pub fn error(tag: Option<String>, err: ResponseError) -> Self {
        Response::new(tag, ResponseBody::Error(err))
    }

    pub fn recv(tag: Option<String>, sender: impl Into<String>, msg: impl Into<String>) -> Self {
        Response::new(tag, ResponseBody::Recv(sender.into(), msg.into()))
    }
}

impl TryFrom<String> for Response {
    type Error = QuipError;

    fn try_from(value: String) -> QuipResult<Self> {
        Response::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Response {
    type Error = QuipError;

    fn try_from(value: &str) -> QuipResult<Self> {
        let mut tokens = tokenize(value)?.into_iter();

        let tag = unwrap_token!(tokens, "No tag found");
        let tag = match tag.as_str() {
            "*" => None,
            _ => Some(tag),
        };

        let resp_type = unwrap_token!(tokens, "No response status found");
        let body = match resp_type.as_str() {
            "Success" => ResponseBody::Success(tokens.next()),
            "Error" => {
                let code = unwrap_token!(tokens, "No error code found for response Error");
                ResponseBody::Error(code.try_into()?)
            }
            "Recv" => {
                let name = unwrap_token!(tokens, "No name found for response Recv");
                let msg = unwrap_token!(tokens, "No message found for response Recv");

                ResponseBody::Recv(name, msg)
            }
            _ => {
                return Err(QuipError::Parse(format!(
                    "Unexpected response {}",
                    resp_type
                )));
            }
        };

        Ok(Response::new(tag, body))
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = match &self.tag {
            Some(tag) => tag,
            None => "*",
        };

        let err_msg;
        let tokens = match &self.body {
            ResponseBody::Success(msg) => match msg {
                Some(msg) => vec![tag, "Success", msg],
                None => vec![tag, "Success"],
            },
            ResponseBody::Error(msg) => {
                err_msg = msg.to_string();
                vec![tag, "Error", err_msg.as_str()]
            }
            ResponseBody::Recv(name, msg) => {
                vec![tag, "Recv", name, msg]
            }
        };

        f.write_str(detokenize(&tokens).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_error() {
        assert_eq!(
            ResponseError::try_from("BadCommand").unwrap(),
            ResponseError::BadCommand
        );
        assert_eq!(
            ResponseError::try_from("Unauthorized").unwrap(),
            ResponseError::Unauthorized
        );
        assert_eq!(
            ResponseError::try_from("Duplicate").unwrap(),
            ResponseError::Duplicate
        );
        assert_eq!(
            ResponseError::try_from("NotFound").unwrap(),
            ResponseError::NotFound
        );
    }

    #[test]
    fn test_response_success() {
        let resp = Response::try_from("A000 Success").unwrap();
        assert_eq!(resp.tag.unwrap(), "A000");

        match resp.body {
            ResponseBody::Success(None) => (),
            _ => panic!("Mismatched response, need Success with no message but others found"),
        }

        let resp = Response::try_from("A000 Success Message").unwrap();
        assert_eq!(resp.tag.unwrap(), "A000");

        match resp.body {
            ResponseBody::Success(Some(msg)) => assert_eq!(msg, "Message"),
            _ => panic!("Mismatched response, need Success but others found"),
        }
    }

    #[test]
    fn test_response_body_error() {
        let resp = Response::try_from("A000 Error Duplicate").unwrap();
        assert_eq!(resp.tag.unwrap(), "A000");

        match resp.body {
            ResponseBody::Error(ResponseError::Duplicate) => (),
            _ => panic!("Mismatched response, need Error Duplicate but others found"),
        }
    }

    #[test]
    fn test_response_recv() {
        let resp = Response::try_from("* Recv Dessera \"How are you today?\"").unwrap();
        assert!(resp.tag.is_none());

        match resp.body {
            ResponseBody::Recv(name, msg) => {
                assert_eq!(name, "Dessera");
                assert_eq!(msg, "How are you today?");
            }
            _ => panic!("Mismatched response, need Recv but others found"),
        }
    }

    #[test]
    fn test_response_error_display() {
        assert_eq!(ResponseError::BadCommand.to_string(), "BadCommand");
        assert_eq!(ResponseError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(ResponseError::Duplicate.to_string(), "Duplicate");
        assert_eq!(ResponseError::NotFound.to_string(), "NotFound");
    }

    #[test]
    fn test_response_display_success() {
        let res = Response::success(Some("A000".to_string()), None);
        assert_eq!(res.to_string(), "A000 Success");

        let res = Response::success(Some("A000".to_string()), Some("AdditionalInfo".to_string()));
        assert_eq!(res.to_string(), "A000 Success AdditionalInfo");

        let res = Response::success(None, None);
        assert_eq!(res.to_string(), "* Success");
    }

    #[test]
    fn test_response_display_error() {
        let res = Response::error(Some("A000".to_string()), ResponseError::BadCommand);
        assert_eq!(res.to_string(), "A000 Error BadCommand");

        let res = Response::error(None, ResponseError::Unauthorized);
        assert_eq!(res.to_string(), "* Error Unauthorized");
    }

    #[test]
    fn test_response_display_recv() {
        let res = Response::recv(Some("A000".to_string()), "Sender", "Message");
        assert_eq!(res.to_string(), "A000 Recv Sender Message");

        let res = Response::recv(None, "Sender", "Message");
        assert_eq!(res.to_string(), "* Recv Sender Message");

        let res = Response::recv(None, "Sender", "Complex  Message");
        assert_eq!(res.to_string(), "* Recv Sender \"Complex  Message\"");
    }
}
