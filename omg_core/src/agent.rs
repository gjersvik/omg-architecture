use std::sync::mpsc::{self, Receiver, SendError, Sender};

pub struct Handle<T> {
    channel: Sender<T>,
}

impl<T> Handle<T> {
    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.channel.send(message)
    }
}

pub trait State where Self: Sized {
    type Input;
    type Output: Clone + Send;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Output>;

    fn agent(self) -> (Handle<Self::Input>, StateAgent<Self>) {
        let (sender, receiver) = mpsc::channel();
        let handle = Handle { channel: sender };

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

    pub fn add_callback(&mut self, callback: Box<dyn Fn(S::Output) + Send>) {
        self.callbacks.push(callback)
    }
}

pub trait Service {
    type Input;
    type Output: Clone + Send;

    fn create(&mut self, channel: Receiver<Self::Input>, callback: Box<dyn Fn(Self::Output) + Send> );
}
