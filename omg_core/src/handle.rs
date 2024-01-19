use futures_lite::future;

pub use async_channel::{RecvError, SendError, TryRecvError, TrySendError};

#[derive(Debug)]
pub struct Handle<In, Out: Clone> {
    input: async_channel::Sender<In>,
    inactive: async_broadcast::InactiveReceiver<Out>,
    output: Option<async_broadcast::Receiver<Out>>,
}

impl<In, Out: Clone> Handle<In, Out> {
    pub fn new_handle(&self) -> Handle<In, Out> {
        Handle {
            input: self.input.clone(),
            inactive: self.inactive.clone(),
            output: None,
        }
    }

    pub async fn write(&self, msg: In) -> Result<(), SendError<In>> {
        self.input.send(msg).await
    }

    pub fn try_write(&self, msg: In) -> Result<(), TrySendError<In>> {
        self.input.try_send(msg)
    }

    pub fn write_blocking(&self, msg: In) -> Result<(), SendError<In>> {
        self.input.send_blocking(msg)
    }

    pub async fn read(&mut self) -> Result<Out, RecvError> {
        self.output().recv_direct().await.map_err(|err| match err {
            async_broadcast::RecvError::Overflowed(_) => {
                panic!("Bug: Handle should not overflow. The agent must wait.")
            }
            async_broadcast::RecvError::Closed => RecvError,
        })
    }

    pub fn try_read(&mut self) -> Result<Out, TryRecvError> {
        self.output().try_recv().map_err(|err| match err {
            async_broadcast::TryRecvError::Overflowed(_) => {
                panic!("Bug: Handle should not overflow. The agent must wait.")
            }
            async_broadcast::TryRecvError::Empty => TryRecvError::Empty,
            async_broadcast::TryRecvError::Closed => TryRecvError::Closed,
        })
    }

    pub fn read_blocking(&mut self) -> Result<Out, RecvError> {
        future::block_on(self.read())
    }

    fn output(&mut self) -> &mut async_broadcast::Receiver<Out> {
        if self.output.is_none() {
            self.output = Some(self.inactive.activate_cloned());
        }
        self.output
            .as_mut()
            .expect("Bug: It should have been impassible for self.option to be None here.")
    }
}

pub struct Context<In, Out: Clone> {
    input: async_channel::Receiver<In>,
    output: async_broadcast::Sender<Out>,
}

impl<In, Out: Clone> Context<In, Out> {
    pub async fn push(&self, value: Out) -> bool {
        self.output.broadcast_direct(value).await.is_err()
    }

    pub async fn pop(&self) -> Option<In> {
        self.input.recv().await.ok()
    }
}

pub fn handle<In, Out: Clone>(cap: usize) -> (Handle<In, Out>, Context<In, Out>) {
    let (in_s, in_r) = async_channel::bounded(cap);
    let (out_s, out_r) = async_broadcast::broadcast(cap);

    (
        Handle {
            input: in_s,
            inactive: out_r.deactivate(),
            output: None,
        },
        Context {
            input: in_r,
            output: out_s,
        },
    )
}