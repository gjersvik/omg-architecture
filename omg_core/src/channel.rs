use std::{error::Error, fmt::Display, future::Future};

use tokio::sync::broadcast::{
    self,
    error::{RecvError, TryRecvError},
};

pub trait Sender {
    type Item: Clone;

    fn send(&self, value: Self::Item);
}

impl<T: Clone> Sender for broadcast::Sender<T> {
    type Item = T;
    fn send(&self, value: T) {
        let _ = self.send(value);
    }
}

pub trait Receiver {
    type Item: Clone;
    fn recv(&mut self) -> Option<Self::Item>;
    fn async_recv(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
    fn try_recv(&mut self) -> Result<Self::Item, TryError>;
}

impl<T: Clone + Send> Receiver for broadcast::Receiver<T> {
    type Item = T;

    fn recv(&mut self) -> Option<T> {
        match self.blocking_recv() {
            Ok(v) => Some(v),
            Err(RecvError::Closed) => None,
            Err(RecvError::Lagged(_)) => Receiver::recv(self),
        }
    }

    async fn async_recv(&mut self) -> Option<T> {
        loop {
            match self.recv().await {
                Ok(v) => break Some(v),
                Err(RecvError::Closed) => break None,
                Err(RecvError::Lagged(_)) => continue,
            }
        }
    }

    fn try_recv(&mut self) -> Result<T, TryError> {
        match self.try_recv() {
            Ok(v) => Ok(v),
            Err(TryRecvError::Empty) => Err(TryError::Empty),
            Err(TryRecvError::Closed) => Err(TryError::Closed),
            Err(TryRecvError::Lagged(_)) => Receiver::try_recv(self),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryError {
    Empty,
    Closed,
}

impl Error for TryError {}

impl Display for TryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "This channel is closed from sender."),
            Self::Empty => write!(f, "There is no data to return just now. (Would block.)"),
        }
    }
}

pub struct Channel<T: Clone + Send> {
    inner: broadcast::Sender<T>,
}

impl<T: Clone + Send> Channel<T> {
    pub fn new(capacity: usize) -> Self {
        Channel {
            inner: broadcast::Sender::new(capacity),
        }
    }

    pub fn publish(&mut self) -> impl Sender<Item = T> {
        self.inner.clone()
    }

    pub fn subscribe(&self) -> impl Receiver<Item = T> {
        self.inner.subscribe()
    }
}

impl<T: Clone + Send> Sender for Channel<T> {
    type Item = T;
    fn send(&self, value: T) {
        let _ = self.inner.send(value);
    }
}
