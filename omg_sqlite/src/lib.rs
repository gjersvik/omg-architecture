use std::{
    error::Error,
    path::PathBuf,
    sync::{mpsc::Receiver, Arc},
    thread,
};

use omg_core::{Agent, Handle, Service, StorageError, StorageInput, StorageItem, StorageTopic};
use sqlite::{Connection, State};

pub fn file(path: impl Into<PathBuf>) -> (Handle<StorageInput>, impl Agent<Output = ()>) {
    Sqlite { path: path.into() }.agent()
}

struct Sqlite {
    path: PathBuf,
}

impl Service for Sqlite {
    type Input = StorageInput;
    type Output = ();

    fn create(&mut self, channel: Receiver<Self::Input>, _: Box<dyn Fn(Self::Output) + Send>) {
        let path = self.path.clone();
        thread::spawn(move || backed(channel, path));
    }
}

fn backed(events: Receiver<StorageInput>, path: PathBuf) {
    let db = Connection::open(path).unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS messages (topic TEXT, seq INTEGER, data TEXT)")
        .unwrap();

    while let Ok(event) = events.recv() {
        match event {
            StorageInput::Topics(reply) => {
                let _ = reply.send(topics(&db).into_storage_error());
            }
            StorageInput::Push(topic, seq, data, reply) => {
                let _ = reply.send(push(&db, &topic, seq, &data).into_storage_error());
            }
            StorageInput::ReadAll(topic, reply) => {
                let _ = reply.send(read_all(&db, &topic).into_storage_error());
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

trait IntoStorageError<T> {
    fn into_storage_error(self) -> Result<T, StorageError>;
}

impl<T> IntoStorageError<T> for Result<T, Box<dyn Error + Send + Sync>> {
    fn into_storage_error(self) -> Result<T, StorageError> {
        self.map_err(Into::into)
    }
}
