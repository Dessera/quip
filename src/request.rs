use crate::TcError;

#[derive(Debug)]
pub enum RequestBody {
    Send(String, String),
    Login(String),
    SetName(String),
    Logout,
    Nop,
}

#[derive(Debug)]
pub struct Request {
    pub tag: String,
    pub body: RequestBody,
}

impl Request {
    pub fn new(tag: String, body: RequestBody) -> Self {
        Self { tag, body }
    }

    pub fn label(&self) -> &str {
        match self.body {
            RequestBody::Send(_, _) => "Send",
            RequestBody::Login(_) => "Login",
            RequestBody::SetName(_) => "SetName",
            RequestBody::Logout => "Logout",
            RequestBody::Nop => "Nop",
        }
    }
}

impl TryFrom<&str> for Request {
    type Error = TcError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.trim().split(' ').collect();

        if parts.len() < 1 {
            return Err(TcError::Parse(value.to_string()));
        }

        let tag = parts[0].to_string();
        let body = match parts[1..] {
            ["Send", name, ..] => RequestBody::Send(name.to_string(), parts[3..].join(" ")),
            ["Login", name] => RequestBody::Login(name.to_string()),
            ["SetName", name] => RequestBody::SetName(name.to_string()),
            ["Logout"] => RequestBody::Logout,
            ["Nop"] => RequestBody::Nop,
            _ => return Err(TcError::Parse(value.to_string())),
        };

        Ok(Request::new(tag, body))
    }
}
