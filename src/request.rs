use crate::{QuipError, QuipResult, token::tokenize};
use std::vec::IntoIter as VecIntoIter;

macro_rules! unwrap_token {
    ($iter:expr, $msg:expr) => {
        match $iter.next() {
            Some(value) => value,
            None => return Err(QuipError::Parse($msg)),
        }
    };
}

/// General request body.
///
/// A general request body may be one of the following requests:
///
/// - `Send`: Send message to another user or group, i.e.
///   `<TAG> Send <USER> <MESSAGE>` or `<TAG> Send G:<GROUP> <MESSAGE>`.
/// - `Login`/`SetName`: Authenticate connection with a user name, i.e.
///   `<TAG> Login|SetName <NAME>`.
/// - `Logout`: Disconnect immediately, i.e. `<TAG> Logout`.
/// - `Nop`: Do nothing, i.e. `<TAG> Nop`.
#[derive(Debug)]
pub enum RequestBody {
    Send(String, String),
    Login(String),
    SetName(String),
    Logout,
    Nop,
}

/// General request, with tag for responses.
#[derive(Debug)]
pub struct Request {
    pub tag: String,
    pub body: RequestBody,
}

impl Request {
    pub fn new(tag: impl Into<String>, body: RequestBody) -> Self {
        Self {
            tag: tag.into(),
            body,
        }
    }
}

impl TryFrom<String> for Request {
    type Error = QuipError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Request::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Request {
    type Error = QuipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens = tokenize(value)?;
        let mut tokens_iter = tokens.into_iter();

        let tag = unwrap_token!(tokens_iter, "No tag found".to_string());

        let cmd = unwrap_token!(tokens_iter, "No command found".to_string());
        let body = match cmd.as_str() {
            "Send" => parse_send_body(tokens_iter)?,
            "Login" => parse_login_body(tokens_iter)?,
            "SetName" => parse_setname_body(tokens_iter)?,
            "Logout" => RequestBody::Logout,
            "Nop" => RequestBody::Nop,
            _ => return Err(QuipError::Parse(format!("Unexpected command {}", cmd))),
        };

        Ok(Request::new(tag, body))
    }
}

fn parse_send_body(mut tokens: VecIntoIter<String>) -> QuipResult<RequestBody> {
    let name = unwrap_token!(tokens, "No name found for command Send".to_string());
    let msg = unwrap_token!(tokens, "No message found for command Send".to_string());

    Ok(RequestBody::Send(name, msg))
}

fn parse_login_body(mut tokens: VecIntoIter<String>) -> QuipResult<RequestBody> {
    let name = unwrap_token!(tokens, "No name found for command Login".to_string());

    Ok(RequestBody::Login(name))
}

fn parse_setname_body(mut tokens: VecIntoIter<String>) -> QuipResult<RequestBody> {
    let name = unwrap_token!(tokens, "No name found for command SetName".to_string());

    Ok(RequestBody::SetName(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_login() {
        let request = Request::try_from("A000 Login Dessera").unwrap();

        assert_eq!(request.tag, "A000");

        match request.body {
            RequestBody::Login(name) => assert_eq!(name, "Dessera"),
            _ => panic!("Mismatched command, need Login but others found"),
        }
    }

    #[test]
    fn test_request_send() {
        let request = Request::try_from("A000 Send Dessera \"How are you today?\"").unwrap();

        assert_eq!(request.tag, "A000");

        match request.body {
            RequestBody::Send(name, msg) => {
                assert_eq!(name, "Dessera");
                assert_eq!(msg, "How are you today?");
            }
            _ => panic!("Mismatched command, need Send but others found"),
        }
    }

    #[test]
    fn test_request_setname() {
        let request = Request::try_from("A000 SetName Dessera").unwrap();

        assert_eq!(request.tag, "A000");

        match request.body {
            RequestBody::SetName(name) => assert_eq!(name, "Dessera"),
            _ => panic!("Mismatched command, need SetName but others found"),
        }
    }

    #[test]
    fn test_request_logout() {
        let request = Request::try_from("A000 Logout").unwrap();

        assert_eq!(request.tag, "A000");

        match request.body {
            RequestBody::Logout => (),
            _ => panic!("Mismatched command, need Logout but others found"),
        }
    }

    #[test]
    fn test_request_nop() {
        let request = Request::try_from("A000 Nop").unwrap();

        assert_eq!(request.tag, "A000");

        match request.body {
            RequestBody::Nop => (),
            _ => panic!("Mismatched command, need Nop but others found"),
        }
    }

    #[test]
    fn test_request_failed() {
        let request = Request::try_from("A000 Invalid Command");
        assert!(request.is_err());
    }
}
