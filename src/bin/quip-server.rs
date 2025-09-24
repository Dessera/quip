use log::error;
use quip::{
    QuipResult,
    server::{self, backend::MemoryBackend, listener::tcp::TcpListener},
};

#[tokio::main]
async fn main() -> QuipResult<()> {
    env_logger::init();

    let listener = match TcpListener::bind("0.0.0.0:1145").await {
        Ok(listener) => listener,
        Err(err) => {
            error!("{}", err);
            return Err(err);
        }
    };

    let backend = MemoryBackend::new();

    if let Err(err) = server::run(listener, backend).await {
        error!("{}", err);
        return Err(err);
    }

    Ok(())
}
