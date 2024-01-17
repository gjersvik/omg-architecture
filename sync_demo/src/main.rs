use std::{collections::BTreeMap, env, error::Error, thread};

use omg_core::{Handle, State, StorageEvent, Topic, TopicCore};
use tokio::sync::oneshot;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Setup the environment
    let (storage, _) = omg_sqlite::file("todo.db");
    let events = load(&storage)?;

    // setup the agent
    let (handle, mut agent) = Todo(BTreeMap::new()).agent();
    agent.add_callback(Box::new(move |event| match event {
        TodoOutput::PrintLine(msg) => println!("{msg}"),
        TodoOutput::Publish(key, value) => publish(&storage, (key, value)),
    }));
    let join = thread::spawn(move || agent.block_until_done());

    //Publish messages
    for event in events {
        handle.send(event)?;
    }

    handle.send(inputs())?;

    // Wait for runner thread.
    let _ = join.join();
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
    storage: &Handle<StorageEvent>,
) -> Result<Vec<TodoInput>, Box<dyn Error + Send + Sync>> {
    let (send, recv) = oneshot::channel();
    storage.send(StorageEvent::Topics(send))?;
    let topics = recv.blocking_recv()??;

    let todo = topics
        .into_iter()
        .find(|topic| topic.name.as_ref() == "todo");
    let core = if let Some(topic) = todo {
        let (send, recv) = oneshot::channel();
        storage.send(StorageEvent::ReadAll(topic.name.clone(), send))?;
        let data = recv.blocking_recv()??;
        TopicCore::new(topic, data, storage.clone())
    } else {
        TopicCore::empty("todo".into(), storage.clone())
    };

    let topic = Topic::new(core);

    let events = topic
        .subscribe()
        .map(|msg| msg.map(|(key, value)| TodoInput::Load(key, value)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(events)
}

fn publish(storage: &Handle<StorageEvent>, data: (u64, Option<String>)) {
    let (send, recv) = oneshot::channel();
    storage.send(StorageEvent::Topics(send)).unwrap();
    let topics = recv.blocking_recv().unwrap().unwrap();

    let todo = topics
        .into_iter()
        .find(|topic| topic.name.as_ref() == "todo");
    let core = if let Some(topic) = todo {
        let (send, recv) = oneshot::channel();
        storage.send(StorageEvent::ReadAll(topic.name.clone(), send)).unwrap();
        let data = recv.blocking_recv().unwrap().unwrap();
        TopicCore::new(topic, data, storage.clone())
    } else {
        TopicCore::empty("todo".into(), storage.clone())
    };

    let topic = Topic::new(core);

    topic.publish(data).unwrap()
}
