use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, Notify},
};

use crate::{TcResult, response::Response, server::connection::ConnectionWriter};
use std::{collections::VecDeque, sync::Arc};

#[derive(Debug, Clone)]
pub struct MessageQueue {
    queue: Arc<Mutex<VecDeque<Response>>>,
    notify: Arc<Notify>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn push(&self, resp: Response) {
        let mut queue = self.queue.lock().await;
        queue.push_back(resp);

        self.notify.notify_one();
    }

    pub async fn transmit<W>(&self, writer: &mut ConnectionWriter<W>) -> TcResult<()>
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

    pub async fn notified(&self) {
        self.notify.notified().await
    }
}
