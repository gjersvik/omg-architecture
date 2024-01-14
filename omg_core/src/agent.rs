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
    callbacks: Vec<Box<dyn Fn(S::Output) -> ()>>,
}

impl<S: State> Agent<S> {
    pub fn new(state: S) -> Self {
        Agent {
            state,
            callbacks: Vec::new(),
        }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn message(&mut self, msg: S::Input) {
        let output = self.state.handle(msg);
        for event in output {
            for callback in self.callbacks.iter() {
                callback(event.clone());
            }
        }
    }

    pub fn add_callback(&mut self, callback: Box<dyn Fn(S::Output) -> ()>) {
        self.callbacks.push(callback)
    }
}
