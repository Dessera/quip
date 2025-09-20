use crate::{QuipResult, response::Response, server::stream::QuipBufWriter};
use std::{collections::VecDeque, sync::Arc};
use tokio::{
    io::AsyncWrite,
    sync::{Mutex, Notify},
};

/// User status to cache message before login.
#[derive(Debug, PartialEq, Eq)]
pub enum UserStatus {
    Cache,
    Auth,
}

#[derive(Debug)]
pub struct UserData {
    pub name: String,
    pub status: UserStatus,
}

/// User handler for server.
#[derive(Debug, Clone)]
pub struct User {
    queue: Arc<Mutex<VecDeque<Response>>>,
    pub notify: Arc<Notify>,
    pub data: Arc<Mutex<UserData>>,
}

impl User {
    pub fn new(name: impl Into<String>, status: UserStatus) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
            data: Arc::new(Mutex::new(UserData {
                name: name.into(),
                status,
            })),
        }
    }

    /// Push a response into queue.
    pub async fn push_resp(&self, resp: Response) {
        let mut queue = self.queue.lock().await;
        let user_data = self.data.lock().await;
        queue.push_back(resp);

        if user_data.status != UserStatus::Cache {
            self.notify.notify_one();
        }
    }

    /// Write all responses to specific writer.
    ///
    /// All responses should be sended via this method (after authenticated).
    pub async fn write_all<W>(&self, writer: &mut QuipBufWriter<W>) -> QuipResult<()>
    where
        W: AsyncWrite + Unpin,
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
