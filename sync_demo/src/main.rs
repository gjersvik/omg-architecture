use std::{collections::BTreeMap, env, error::Error, mem};

use omg_core::{Agency, Agent, State, Topic};

type TodoMsg = (u64, Option<String>);

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Setup the environment
    let storage = omg_sqlite::file("todo.db");

    let mut agency = Agency::load(storage)?;
    let topic = agency.create_topic("todo");
    let events = load(&topic)?;

    let mut agent = Agent::new(Todo(BTreeMap::new()));
    let mut outputs = agent.subscribe();
    for event in events {
        agent.message(event);
    }

    // Run the current command
    agent.message(inputs());
    mem::drop(agent);

    // Handle the results.
    while let Some(msg) = outputs.recv() {
        match msg {
            TodoOutput::PrintLine(s) => println!("{s}"),
            TodoOutput::Publish(key, value) => topic.publish((key, value))?,
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

fn load(topic: &Topic<TodoMsg>) -> Result<Vec<TodoInput>, Box<dyn Error + Send + Sync>> {
    topic
        .subscribe()
        .map(|msg| msg.map(|(key, value)| TodoInput::Load(key, value)))
        .collect()
}
