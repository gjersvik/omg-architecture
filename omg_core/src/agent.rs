use futures_lite::future;

#[derive(Debug, Clone)]
pub struct Handle<In, Out> {
    pub input: async_channel::Sender<In>,
    pub output: async_broadcast::Receiver<Out>,
}

pub struct Context<In, Out: Clone> {
    pub input: async_channel::Receiver<In>,
    output: async_broadcast::Sender<Out>,
}

impl<In, Out: Clone> Context<In, Out> {
    pub async fn push(&self, value: Out) -> bool {
        self.output.broadcast_direct(value).await.is_err()
    } 
}

pub fn handle<In, Out: Clone>(cap: usize) -> (Handle<In, Out>, Context<In, Out>) {
    let (in_s, in_r) = async_channel::bounded(cap);
    let (out_s, out_r) = async_broadcast::broadcast(cap);

    (
        Handle {
            input: in_s,
            output: out_r,
        },
        Context {
            input: in_r,
            output: out_s,
        },
    )
}

pub trait State
where
    Self: Sized,
{
    type Input;
    type Output: Clone + Send;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Output>;

    fn agent(self) -> (Handle<Self::Input, Self::Output>, StateAgent<Self>) {
        let (handle, context) = handle(64);

        let agent: StateAgent<Self> = StateAgent {
            state: self,
            context
        };

        (handle, agent)
    }
}

pub struct StateAgent<S: State> {
    state: S,
    context: Context<S::Input, S::Output>
}

impl<S: State> StateAgent<S> {
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn block_until_done(mut self) {
        while let Ok(msg) = self.context.input.recv_blocking() {
            self.message(msg)
        }
    }

    fn message(&mut self, msg: S::Input) {
        let output = self.state.handle(msg);
        for event in output {
            let _ = future::block_on(self.context.push(event));
        }
    }
}
