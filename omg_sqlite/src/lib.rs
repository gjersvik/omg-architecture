use std::{error::Error, path::PathBuf, sync::Arc, thread};

use omg_core::{Storage, StorageItem, StorageTopic};
use sqlite::{Connection, State};
use tokio::sync::{mpsc, oneshot};

pub fn file_blocking(path: impl Into<PathBuf>) -> Box<dyn Storage> {
    let (send, recv) = mpsc::unbounded_channel();
    let path = path.into();
    thread::spawn(move || backed(recv, path));

    Box::new(SqliteBackend { backend: send })
}

enum StorageEvent {
    Topics(oneshot::Sender<Result<Vec<StorageTopic>, Box<dyn Error + Send + Sync + 'static>>>),
    Push(
        Arc<str>,
        u64,
        Arc<str>,
        oneshot::Sender<Result<(), Box<dyn Error + Send + Sync + 'static>>>,
    ),
    ReadAll(
        Arc<str>,
        oneshot::Sender<Result<Vec<StorageItem>, Box<dyn Error + Send + Sync + 'static>>>,
    ),
}

struct SqliteBackend {
    backend: mpsc::UnboundedSender<StorageEvent>,
}

impl Storage for SqliteBackend {
    fn append_blocking(&self, topic: &str, seq: u64, data: &str) -> Result<(), Box<dyn Error>> {
        let (send, recv) = oneshot::channel();
        self.backend
            .send(StorageEvent::Push(topic.into(), seq, data.into(), send))
            .expect("Database thread is just gone");
        recv.blocking_recv()
            .expect("Database thread is just gone")
            .map_err(|e| e as Box<dyn Error>)
    }

    fn read_all_blocking(&self, topic: &str) -> Result<Vec<StorageItem>, Box<dyn Error>> {
        let (send, recv) = oneshot::channel();
        self.backend
            .send(StorageEvent::ReadAll(topic.into(), send))
            .expect("Database thread is just gone");
        recv.blocking_recv()
            .expect("Database thread is just gone")
            .map_err(|e| e as Box<dyn Error>)
    }

    fn topics(&self) -> Result<Vec<StorageTopic>, Box<dyn Error>> {
        let (send, recv) = oneshot::channel();
        self.backend
            .send(StorageEvent::Topics(send))
            .expect("Database thread is just gone");
        recv.blocking_recv()
            .expect("Database thread is just gone")
            .map_err(|e| e as Box<dyn Error>)
    }
}

fn backed(mut events: mpsc::UnboundedReceiver<StorageEvent>, path: PathBuf) {
    let db = Connection::open(path).unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS messages (topic TEXT, seq INTEGER, data TEXT)").unwrap();

    while let Some(event) = events.blocking_recv() {
        match event {
            StorageEvent::Topics(reply) => {
                let _ = reply.send(topics(&db));
            }
            StorageEvent::Push(topic, seq, data, reply) => {
                let _ = reply.send(push(&db, &topic, seq, &data));
            }
            StorageEvent::ReadAll(topic, reply) => {
                let _ = reply.send(read_all(&db, &topic));
            }
        }
    }
}

fn topics(db: &Connection) -> Result<Vec<StorageTopic>, Box<dyn Error + Send + Sync>> {
    let statement = db.prepare(
        "SELECT topic, min(seq) as first, max(seq) as last FROM messages GROUP BY topic",
    )?;
    statement
        .into_iter()
        .map(|row| {
            let row = row?;

            Ok(StorageTopic {
                name: Arc::from(row.try_read::<&str, _>("topic")?),
                first: row.try_read::<i64, _>("first")? as u64,
                last: row.try_read::<i64, _>("last")? as u64,
            })
        })
        .collect()
}

fn push(
    db: &Connection,
    topic: &str,
    seq: u64,
    data: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut statement = db.prepare("INSERT INTO messages VALUES (:topic, :seq, :data)")?;
    statement.bind((":topic", topic))?;
    statement.bind((":seq", seq as i64))?;
    statement.bind((":data", data))?;

    while statement.next()? != State::Done {}
    Ok(())
}

fn read_all(
    db: &Connection,
    topic: &str,
) -> Result<Vec<StorageItem>, Box<dyn Error + Send + Sync>> {
    let mut statement =
        db.prepare("SELECT seq, data FROM messages WHERE topic = ? ORDER BY seq ASC")?;
    statement.bind((1, topic))?;

    statement
        .into_iter()
        .map(|row| {
            let row = row?;

            Ok(StorageItem {
                seq: row.try_read::<i64, _>("seq")? as u64,
                data: row.try_read::<&str, _>("data")?.into(),
            })
        })
        .collect()
}
