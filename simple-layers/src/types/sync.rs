use async_broadcast::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Waiter(Receiver<()>);

impl Waiter {
    pub(crate) fn new(r: Receiver<()>) -> Self {
        Self(r)
    }

    pub async fn wait(mut self) {
        self.0.recv().await.unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct Waker(Sender<()>);

impl Waker {
    pub(crate) fn new(s: Sender<()>) -> Self {
        Self(s)
    }

    pub async fn signal(self) {
        self.0.broadcast(()).await.unwrap();
    }
}

pub(crate) fn signal_channel() -> (Waker, Waiter) {
    let (s, r) = async_broadcast::broadcast(1);
    (Waker::new(s), Waiter::new(r))
}
