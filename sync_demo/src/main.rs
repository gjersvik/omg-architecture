use std::{collections::BTreeMap, env, error::Error, thread};

use omg_core::{Agent, State, StorageInput, WriteHandle};
use tokio::sync::oneshot;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Setup the environment
    let storage = omg_sqlite::file("todo.db").write_only();
    let events = load(&storage)?;

    // setup the agent
    let (handle, agent) = Todo(BTreeMap::new()).agent();
    thread::spawn(move || agent.run_blocking());

    let (mut reader, writer) = handle.split();

    //Publish messages
    for event in events {
        writer.write_blocking(event)?;
    }

    writer.write_blocking(inputs())?;
    drop(writer);

    // Handle outputs;
    while let Ok(event) = reader.read_blocking() {
        match event {
            TodoOutput::PrintLine(msg) => println!("{msg}"),
            TodoOutput::Publish(key, value) => publish(&storage, (key, value)),
        }
    }
    Ok(())
}

enum TodoInput {
    Help,
    List,
    Add(String),
    Remove(u64),
    Load(u64, Option<String>),
}

#[derive(Debug, Clone)]
enum TodoOutput {
    PrintLine(String),
    Publish(u64, Option<String>),
}

struct Todo(BTreeMap<u64, String>);

impl State for Todo {
    type Input = TodoInput;
    type Output = TodoOutput;

    fn handle(&mut self, msg: Self::Input) -> Vec<Self::Output> {
        match msg {
            TodoInput::Help => {
                vec![
                    TodoOutput::PrintLine("Help for Sync demo todo app".to_owned()),
                    TodoOutput::PrintLine("sync_demo list".to_owned()),
                    TodoOutput::PrintLine("Will list all the active todo items.".to_owned()),
                    TodoOutput::PrintLine("sync_demo add [task]".to_owned()),
                    TodoOutput::PrintLine("Adds task to the todo lists.".to_owned()),
                    TodoOutput::PrintLine("sync_demo remove [id]".to_owned()),
                    TodoOutput::PrintLine("Removes/completes the task with id: id.".to_owned()),
                ]
            }
            TodoInput::Load(key, Some(value)) => {
                self.0.insert(key, value);
                Vec::new()
            }
            TodoInput::Load(key, None) => {
                self.0.remove(&key);
                Vec::new()
            }
            TodoInput::List => self
                .0
                .iter()
                .map(|(id, task)| TodoOutput::PrintLine(format!("{id}: {task}")))
                .collect(),
            TodoInput::Add(task) => {
                let next_id = self.0.last_key_value().map(|(id, _)| *id + 1).unwrap_or(1);
                vec![
                    TodoOutput::Publish(next_id, Some(task.clone())),
                    TodoOutput::PrintLine(format!("Added {task} with id {next_id}")),
                ]
            }
            TodoInput::Remove(id) => {
                vec![
                    TodoOutput::Publish(id, None),
                    TodoOutput::PrintLine(format!("Removed task with id {id}")),
                ]
            }
        }
    }
}

fn inputs() -> TodoInput {
    let args: Vec<_> = env::args().collect();
    let args_str: Vec<_> = args.iter().map(|s| s as &str).collect();

    match args_str[..] {
        [_, "list"] => TodoInput::List,
        [_, "add", task] => TodoInput::Add(task.to_owned()),
        [_, "remove", id] if id.parse::<u64>().is_ok() => {
            TodoInput::Remove(id.parse::<u64>().unwrap())
        }
        _ => TodoInput::Help,
    }
}

fn load(
    storage: &WriteHandle<StorageInput>,
) -> Result<Vec<TodoInput>, Box<dyn Error + Send + Sync>> {
    let (send, recv) = oneshot::channel();
    storage.write_blocking(StorageInput::ReadAll("todo".into(), send))?;
    let data = recv.blocking_recv()??;
    data.into_iter()
        .map(|item| {
            let (key, value) = serde_json::from_str(&item.data)?;
            Ok(TodoInput::Load(key, value))
        })
        .collect()
}

fn publish(storage: &WriteHandle<StorageInput>, data: (u64, Option<String>)) {
    let (send, recv) = oneshot::channel();
    storage.write_blocking(StorageInput::Topics(send)).unwrap();
    let topics = recv.blocking_recv().unwrap().unwrap();

    let todo = topics
        .into_iter()
        .find(|topic| topic.name.as_ref() == "todo");
    let next_id = if let Some(topic) = todo {
        topic.last + 1
    } else {
        1
    };

    let (send, recv) = oneshot::channel();
    storage
        .write_blocking(StorageInput::Push(
            "todo".into(),
            next_id,
            serde_json::to_string(&data).unwrap().into(),
            send,
        ))
        .unwrap();
    recv.blocking_recv().unwrap().unwrap();
}
