use futures_lite::future;

use crate::{Context, Handle};

pub trait State
where
    Self: Sized,
{
    type Input;
    type Output: Clone + Send;

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

    pub fn block_until_done(mut self) {
        future::block_on(async {
            while let Some(msg) = self.context.pop().await {
                self.message(msg).await
            }
        });
    }

    async fn message(&mut self, msg: S::Input) {
        let output = self.state.handle(msg);
        for event in output {
            let _ = self.context.push(event).await;
        }
    }
}
