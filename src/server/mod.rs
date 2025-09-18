pub mod backend;
pub mod connection;
pub mod queue;

use crate::server::backend::Backend;
use log::warn;
use std::sync::Arc;

pub async fn run<B: Backend + Send + Sync + 'static>(server: B) -> ! {
    let server = Arc::new(server);
    loop {
        let conn = match server.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                warn!("Failed to accept connection: {}", err);
                continue;
            }
        };

        let server_rc = server.clone();
        tokio::spawn(async move { server_rc.serve(conn).await });
    }
}
