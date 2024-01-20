use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_executor::{Executor, Task};
use futures_lite::Future;

use crate::{Agent, ReadHandle, WriteHandle};

pub struct Agency {
    ex: Arc<Executor<'static>>,
}

impl Agency {
    pub fn new() -> Self {
        Agency {
            ex: Arc::new(Executor::new()),
        }
    }

    pub fn add_agent(&self, agent: impl Agent + 'static) {
        self.spawn(async move { agent.run().await }).detach();
    }

    pub fn connect<In: Clone + Send + Sync + 'static, Out: Send + 'static>(
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
        .detach();
    }

    pub fn spawn<T: Send + 'static>(
        &self,
        future: impl Future<Output = T> + Send + 'static,
    ) -> Task<T> {
        self.ex.spawn(future)
    }

    pub fn tick(&self) -> bool {
        self.ex.try_tick()
    }

    pub fn tick_for(&self, duration: Duration) {
        let start = Instant::now();
        loop {
            if start.elapsed() >= duration {
                break;
            }
            if !self.tick() {
                break;
            }
        }
    }

    pub fn block(&self) {
        todo!()
    }

    pub fn block_for(&self, _duration: Duration) {
        todo!()
    }

    pub fn run_background(&self) {
        todo!()
    }

    pub fn run_background_max_cpu(&self) {
        todo!()
    }

    pub fn full_stop(&self) {
        todo!()
    }

    pub fn run_to_completion() {
        todo!()
    }
}

impl Default for Agency {
    fn default() -> Self {
        Self::new()
    }
}
