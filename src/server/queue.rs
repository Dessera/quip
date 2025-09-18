use std::collections::VecDeque;

pub struct MessageQueue {
    pub queue: VecDeque<String>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}
