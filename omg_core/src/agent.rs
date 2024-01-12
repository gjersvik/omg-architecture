use crate::Sender;

pub trait State {
    type Input;
    type Output: Clone + Send;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Output>;
}

pub struct Agent<S: State> {
    state: S,
    senders: Vec<Box<dyn Sender<Item = S::Output>>>,
}

impl<S: State> Agent<S> {
    pub fn new(state: S) -> Self {
        Agent {
            state,
            senders: Vec::new(),
        }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn message(&mut self, msg: S::Input) {
        let output = self.state.handle(msg);
        for event in output {
            for sender in self.senders.iter() {
                sender.send(event.clone());
            }
        }
    }

    pub fn on_output(&mut self, sender: Box<dyn Sender<Item = S::Output>>) {
        self.senders.push(sender)
    }
}
