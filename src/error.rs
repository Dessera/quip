#[derive(thiserror::Error, Debug)]
pub enum QuipError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSL/TLS error: {0}")]
    Tls(#[from] native_tls::Error),

    #[error("Client has disconnected")]
    Disconnect,

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Serialize error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Duplicate error: {0}")]
    Duplicate(String),

    #[error("NotFound error: {0}")]
    NotFound(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type QuipResult<T> = Result<T, QuipError>;
