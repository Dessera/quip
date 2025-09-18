#[derive(thiserror::Error, Debug)]
pub enum TcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Client has disconnected")]
    Disconnect,

    #[error("Parse error: Invalid command '{0}'")]
    Parse(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type TcResult<T> = Result<T, TcError>;
