pub trait State {
    type Input;
    type Output;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Input>;
}

pub struct Agent<S:State>{
    state: S,
}

impl<S: State> Agent<S> {
    pub fn new(state: S) -> Self {
        Agent { state }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn message(&mut self, msg: S::Input){
        self.state.handle(msg);
    }
}
