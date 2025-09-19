pub mod backend;
pub mod connection;
pub mod service;
pub mod user;

use crate::{TcResult, server::backend::Backend};
use log::{info, warn};
use std::sync::Arc;

pub async fn run<B: Backend + Send + Sync + 'static>(server: B) -> TcResult<()> {
    match server.address() {
        Ok(addr) => info!("Tchat server listening on {}", addr),
        Err(_) => info!("Tchat server listening"),
    };

    let server = Arc::new(server);
    let mut handles = Vec::new();
    loop {
        let conn = match server.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                warn!("Failed to accept connection: {}", err);
                continue;
            }
        };

        let server = server.clone();
        let handle = tokio::spawn(async move { server.serve(conn).await });

        handles.push(handle);
    }
}
