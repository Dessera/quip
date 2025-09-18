use log::error;
use tchat::{
    TcResult,
    server::{self, backend::SimpleServer},
};

#[tokio::main]
async fn main() -> TcResult<()> {
    env_logger::init();

    let backend = match SimpleServer::bind("0.0.0.0:1145").await {
        Ok(backend) => backend,
        Err(err) => {
            error!("{}", err);
            return Err(err);
        }
    };

    if let Err(err) = server::run(backend).await {
        error!("{}", err);
        return Err(err);
    }

    Ok(())
}
