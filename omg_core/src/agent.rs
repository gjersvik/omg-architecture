use std::sync::mpsc::{self, Receiver, SendError, Sender};

pub trait State {
    type Input;
    type Output: Clone + Send;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Output>;
}

impl<T: State> ActorTypes for T {
    type Input = T::Input;
    type Output = T::Output;
}

pub trait ActorTypes {
    type Input;
    type Output: Clone + Send;
}

pub struct Agent<S: ActorTypes> {
    state: S,
    channel: Receiver<S::Input>,
    callbacks: Vec<Box<dyn Fn(S::Output) -> () + Send>>,
}

impl<S: State> Agent<S> {
    pub fn new(state: S) -> (Handle<S::Input>, Self) {
        let (sender, receiver) = mpsc::channel();
        let handle = Handle { channel: sender };

        let agent = Agent {
            state,
            channel: receiver,
            callbacks: Vec::new(),
        };

        (handle, agent)
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn block_until_done(mut self) {
        while let Ok(msg) = self.channel.recv() {
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

    pub fn add_callback(&mut self, callback: Box<dyn Fn(S::Output) -> () + Send>) {
        self.callbacks.push(callback)
    }
}

#[derive(Debug, Clone)]
pub struct Handle<T> {
    channel: Sender<T>,
}

impl<T> Handle<T> {
    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.channel.send(message)
    }
}
