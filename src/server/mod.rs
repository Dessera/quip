pub mod backend;
pub mod connection;
pub mod listener;
pub mod service;

use crate::{
    QuipResult,
    server::{backend::Backend, listener::Listener},
};
use log::{info, warn};
use std::sync::Arc;

/// Server runner with any listener and backend implementation.
pub async fn run<L, B>(listener: L, backend: B) -> QuipResult<()>
where
    L: Listener,
    B: Backend + Send + Sync + 'static,
{
    match listener.address() {
        Ok(addr) => info!("Server listening on {}", addr),
        Err(_) => info!("Server listening on unknown port"),
    };

    let backend = Arc::new(backend);
    let mut handles = Vec::new();
    loop {
        let conn = match listener.accept().await {
            Ok(conn) => conn,
            Err(_) => continue,
        };

        let backend = backend.clone();
        let handle = tokio::spawn(async move {
            if let Err(err) = service::serve(&*backend, conn).await {
                warn!("Connection handler exit with error:\n  {}", err);
            }
        });

        handles.push(handle);
    }
}
