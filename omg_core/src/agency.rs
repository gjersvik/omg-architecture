use std::time::Duration;

use async_executor::Task;
use futures_lite::Future;

use crate::{Agent, ReadHandle, WriteHandle};

pub struct Agency {}

impl Agency {
    pub fn new() -> Self {
        Agency {}
    }

    pub fn add_agent(&self, _agent: impl Agent) {
        todo!()
    }

    pub fn connect<In: Clone, Out>(
        _from: ReadHandle<In>,
        _to: WriteHandle<Out>,
        _filter_map: impl Fn(In) -> Option<Out>,
    ) {
        todo!()
    }

    pub fn spawn<T: Send>(&self, _future: impl Future<Output = T> + Send) -> Task<T> {
        todo!()
    }

    pub fn tick(&self) -> bool {
        todo!()
    }

    pub fn tick_for(&self, _duration: Duration) {
        todo!()
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
