use tchat::{
    TcResult,
    server::{self, backend::SimpleServer},
};

#[tokio::main]
async fn main() -> TcResult<()> {
    env_logger::init();

    server::run(SimpleServer::bind("0.0.0.0:1145").await?).await
}
