use std::future::Future;

use crate::{Agent, ReadHandle, WriteHandle};

pub trait Agency {
    fn spawn(
        &self,
        future: impl Future<Output = ()> + Send + 'static,
    );

    fn add_agent(&self, agent: impl Agent + 'static) {
        self.spawn(async move { agent.run().await });
    }

    fn connect<In: Clone + Send + Sync + 'static, Out: Send + 'static>(
        &self,
        mut from: ReadHandle<In>,
        to: WriteHandle<Out>,
        filter_map: impl Fn(In) -> Option<Out> + Send + 'static,
    ) {
        self.spawn(async move {
            while let Ok(msg) = from.read().await {
                let msg = filter_map(msg);
                if let Some(msg) = msg {
                    let _ = to.write(msg).await;
                }
            }
        })
    }
}

