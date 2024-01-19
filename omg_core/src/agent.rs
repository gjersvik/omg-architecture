use futures_lite::{future, Future};

use crate::{Context, Handle};

pub trait Agent
where
    Self: Sized + Send,
{
    fn tick(&mut self) -> impl Future<Output = bool> + Send;

    fn run(mut self) -> impl Future<Output = ()> + Send {
        async move { while self.tick().await {} }
    }

    fn run_blocking(self) {
        future::block_on(self.run())
    }
}

pub trait State
where
    Self: Sized + Send,
{
    type Input: Send;
    type Output: Clone + Send + Sync;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Output>;

    fn agent(self) -> (Handle<Self::Input, Self::Output>, StateAgent<Self>) {
        let (handle, context) = crate::handle(64);

        let agent: StateAgent<Self> = StateAgent {
            state: self,
            context,
        };

        (handle, agent)
    }
}

pub struct StateAgent<S: State> {
    state: S,
    context: Context<S::Input, S::Output>,
}

impl<S: State> StateAgent<S> {
    pub fn state(&self) -> &S {
        &self.state
    }
}

impl<S: State> Agent for StateAgent<S> {
    async fn tick(&mut self) -> bool {
        if let Some(msg) = self.context.pop().await {
            let output = self.state.handle(msg);
            for event in output {
                let _ = self.context.push(event).await;
            }
            true
        } else {
            false
        }
    }
}
