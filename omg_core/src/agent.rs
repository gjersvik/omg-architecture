use crate::{Receiver, Sender};

pub trait State {
    type Input;
    type Output: Clone;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Output>;
}

pub struct Agent<S: State> {
    state: S,
    sender: Sender<S::Output>,
}

impl<S: State> Agent<S> {
    pub fn new(state: S) -> Self {
        Agent {
            state,
            sender: Sender::new(64),
        }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn message(&mut self, msg: S::Input) {
        let output = self.state.handle(msg);
        for event in output {
            self.sender.send(event);
        }
    }

    pub fn subscribe(&self) -> Receiver<S::Output> {
        self.sender.subscribe()
    }
}
