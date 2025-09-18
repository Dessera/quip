use crate::TcError;

#[derive(Debug)]
pub enum RequestBody {
    Login(String),
    SetName(String),
    Nop,
}

#[derive(Debug)]
pub struct Request {
    pub body: RequestBody,
}

impl Request {
    pub fn new(body: RequestBody) -> Self {
        Self { body }
    }

    pub fn label(&self) -> &str {
        match self.body {
            RequestBody::Login(_) => "LOGIN",
            RequestBody::SetName(_) => "SETNAME",
            RequestBody::Nop => "NOP",
        }
    }
}

impl TryFrom<&str> for Request {
    type Error = TcError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.trim().split(' ').collect();
        let body = match parts[..] {
            ["LOGIN", name] => RequestBody::Login(name.to_string()),
            ["SETNAME", name] => RequestBody::SetName(name.to_string()),
            ["NOP"] => RequestBody::Nop,
            _ => return Err(TcError::Parse(value.to_string())),
        };

        Ok(Request::new(body))
    }
}
