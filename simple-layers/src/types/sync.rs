use async_broadcast::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Waiter{
    name: String,
    r: Receiver<()>
}

impl Waiter {
    pub(crate) fn new(name: String, r: Receiver<()>) -> Self {
        Self {
            name,
            r
        }
    }

    pub async fn wait(mut self) {
        self.r.recv().await.expect(&format!("Wait failed: [{}]", self.name));
    }
}

#[derive(Debug, Clone)]
pub struct Waker {
    name: String,
    s: Sender<()>
}

impl Waker {
    pub(crate) fn new(name: String, s: Sender<()>) -> Self {
        Self {
            name,
            s
        }
    }

    pub async fn signal(self) {
        self.s.broadcast(()).await.expect(&format!("Signal failed: [{}]", self.name));
    }
}

pub(crate) fn signal_channel(name: String) -> (Waker, Waiter) {
    let (s, r) = async_broadcast::broadcast(1);
    (Waker::new(name.clone(), s), Waiter::new(name, r))
}
