use tokio::{io::AsyncWriteExt, sync::Mutex};

use crate::{TcResult, response::Response, server::connection::ConnectionWriter};
use std::{collections::VecDeque, sync::Arc};

#[derive(Debug, Clone)]
pub struct MessageQueue(Arc<Mutex<VecDeque<Response>>>);

impl MessageQueue {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(VecDeque::new())))
    }

    pub async fn push(&self, resp: Response) {
        let mut queue = self.0.lock().await;
        queue.push_back(resp);
    }

    pub async fn transmit<W>(&self, writer: &mut ConnectionWriter<W>) -> TcResult<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        let mut queue = self.0.lock().await;

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
