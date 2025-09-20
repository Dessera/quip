use crate::TcError;

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
    type Error = TcError;

    // TODO: Need a parser.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.trim().split(' ').collect();

        if parts.len() < 1 {
            return Err(TcError::Parse(value));
        }

        let tag = parts[0];
        let body = match parts[1..] {
            ["Send", name, ..] => RequestBody::Send(name.to_string(), parts[3..].join(" ")),
            ["Login", name] => RequestBody::Login(name.to_string()),
            ["SetName", name] => RequestBody::SetName(name.to_string()),
            ["Logout"] => RequestBody::Logout,
            ["Nop"] => RequestBody::Nop,
            _ => return Err(TcError::Parse(value)),
        };

        Ok(Request::new(tag, body))
    }
}

impl TryFrom<&str> for Request {
    type Error = TcError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Request::try_from(value.to_string())
    }
}
