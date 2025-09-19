use crate::{TcResult, response::Response, server::connection::ConnectionWriter};
use std::{collections::VecDeque, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, Notify},
};

#[derive(Debug)]
pub struct UserData {
    pub name: String,
}

/// User handler for server.
#[derive(Debug, Clone)]
pub struct User {
    queue: Arc<Mutex<VecDeque<Response>>>,
    pub notify: Arc<Notify>,
    pub data: Arc<Mutex<UserData>>,
}

impl User {
    pub fn new(name: String) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
            data: Arc::new(Mutex::new(UserData { name })),
        }
    }

    /// Push a response into queue.
    pub async fn push_resp(&self, resp: Response) {
        let mut queue = self.queue.lock().await;
        queue.push_back(resp);

        self.notify.notify_one();
    }

    /// Write all responses to specific writer.
    ///
    /// All responses should be sended via this method (after authenticated).
    pub async fn write_all<W>(&self, writer: &mut ConnectionWriter<W>) -> TcResult<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        let mut queue = self.queue.lock().await;

        while !queue.is_empty() {
            let resp = match queue.pop_front() {
                Some(resp) => resp,
                None => continue,
            };

            writer.write_response(resp).await?;
        }

        Ok(())
    }
}
