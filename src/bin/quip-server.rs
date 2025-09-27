use log::error;
use quip::{
    QuipResult,
    data::{BackendData, User},
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

    let users = vec![
        User {
            name: "Dessera".into(),
            password: "Pass".into(),
        },
        User {
            name: "Scarlet".into(),
            password: "Pass".into(),
        },
    ];

    let groups = vec![];

    let data = BackendData::new(users, groups);

    let backend = match MemoryBackend::from_data(data) {
        Ok(backend) => backend,
        Err(err) => {
            error!("{}", err);
            return Err(err);
        }
    };

    if let Err(err) = server::run(listener, backend).await {
        error!("{}", err);
        return Err(err);
    }

    Ok(())
}
