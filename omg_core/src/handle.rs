use futures_lite::future;

pub use async_channel::{RecvError, SendError, TryRecvError, TrySendError};

pub struct Handle<In, Out: Clone> {
    reader: ReadHandle<Out>,
    writer: WriteHandle<In>,
}

impl<In, Out: Clone> Handle<In, Out> {
    pub fn new_handle(&self) -> Handle<In, Out> {
        Handle {
            reader: self.reader.new_reader(),
            writer: self.writer.new_writer(),
        }
    }

    pub fn merge(reader: ReadHandle<Out>, writer: WriteHandle<In>) -> Self {
        Handle { reader, writer }
    }

    pub fn split(self) -> (ReadHandle<Out>, WriteHandle<In>) {
        let Self { reader, writer } = self;
        (reader, writer)
    }

    pub fn read_only(self) -> ReadHandle<Out> {
        self.reader
    }

    pub fn write_only(self) -> WriteHandle<In> {
        self.writer
    }
}

pub struct ReadHandle<Out: Clone>(async_broadcast::Receiver<Out>);

impl<Out: Clone> ReadHandle<Out> {
    pub fn new_reader(&self) -> Self {
        ReadHandle(self.0.new_receiver())
    }

    pub async fn read(&mut self) -> Result<Out, RecvError> {
        self.0.recv_direct().await.map_err(|err| match err {
            async_broadcast::RecvError::Overflowed(_) => {
                panic!("Bug: Handle should not overflow. The agent must wait.")
            }
            async_broadcast::RecvError::Closed => RecvError,
        })
    }

    pub fn try_read(&mut self) -> Result<Out, TryRecvError> {
        self.0.try_recv().map_err(|err| match err {
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
}

pub struct WriteHandle<In>(async_channel::Sender<In>);

impl<In> WriteHandle<In> {
    pub fn new_writer(&self) -> Self {
        WriteHandle(self.0.clone())
    }

    pub async fn write(&self, msg: In) -> Result<(), SendError<In>> {
        self.0.send(msg).await
    }

    pub fn try_write(&self, msg: In) -> Result<(), TrySendError<In>> {
        self.0.try_send(msg)
    }

    pub fn write_blocking(&self, msg: In) -> Result<(), SendError<In>> {
        self.0.send_blocking(msg)
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
            reader: ReadHandle(out_r),
            writer: WriteHandle(in_s),
        },
        Context {
            input: in_r,
            output: out_s,
        },
    )
}
