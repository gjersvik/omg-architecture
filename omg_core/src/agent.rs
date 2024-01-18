use std::sync::{Arc, Mutex};

use async_channel::{Receiver, Sender};

#[derive(Debug)]
pub struct Handle<In> {
    pub input: Sender<In>,
}

pub trait Agent {
    type Output: Clone + Send + 'static;

    fn add_callback(&mut self, callback: Box<dyn Fn(Self::Output) + Send>);
}

pub trait State
where
    Self: Sized,
{
    type Input;
    type Output: Clone + Send;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Output>;

    fn agent(self) -> (Handle<Self::Input>, StateAgent<Self>) {
        let (sender, receiver) = async_channel::unbounded();
        let handle = Handle { input: sender };

        let agent: StateAgent<Self> = StateAgent {
            state: self,
            channel: receiver,
            callbacks: Vec::new(),
        };

        (handle, agent)
    }
}

pub struct StateAgent<S: State> {
    state: S,
    channel: Receiver<S::Input>,
    callbacks: Vec<Box<dyn Fn(S::Output) + Send>>,
}

impl<S: State> StateAgent<S> {
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn block_until_done(mut self) {
        while let Ok(msg) = self.channel.recv_blocking() {
            self.message(msg)
        }
    }

    fn message(&mut self, msg: S::Input) {
        let output = self.state.handle(msg);
        for event in output {
            for callback in self.callbacks.iter() {
                callback(event.clone());
            }
        }
    }

    pub fn add_callback(&mut self, callback: Box<dyn Fn(S::Output) + Send>) {
        self.callbacks.push(callback)
    }
}

type ServiceCallback<T> = Arc<Mutex<Vec<Box<dyn Fn(T) + Send>>>>;

pub trait Service
where
    Self: Sized,
{
    type Input;
    type Output: Clone + Send + 'static;

    fn create(
        &mut self,
        channel: Receiver<Self::Input>,
        callback: Box<dyn Fn(Self::Output) + Send>,
    );

    fn agent(mut self) -> (Handle<Self::Input>, ServiceAgent<Self>) {
        let (sender, receiver) = async_channel::unbounded();
        let handle = Handle { input: sender };

        let callbacks: ServiceCallback<Self::Output> = Arc::default();
        let callbacks_clone = callbacks.clone();
        let callback = move |msg: Self::Output| {
            for callback in callbacks_clone.lock().unwrap().iter() {
                callback(msg.clone());
            }
        };

        self.create(receiver, Box::new(callback));

        let agent = ServiceAgent {
            _state: self,
            _callbacks: callbacks,
        };

        (handle, agent)
    }
}

pub struct ServiceAgent<S: Service> {
    _state: S,
    _callbacks: ServiceCallback<S::Output>,
}

impl<S: Service> Agent for ServiceAgent<S> {
    type Output = S::Output;

    fn add_callback(&mut self, _callback: Box<dyn Fn(S::Output) + Send>) {
        todo!()
    }
}
