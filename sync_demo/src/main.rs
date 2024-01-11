use std::{
    collections::BTreeMap,
    env::{self, Args},
    error::Error,
    mem,
};

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
    let mut args = env::args();
    args.next();
    match args.next().as_deref() {
        Some("list") => list(&agent),
        Some("add") => add(args, &topic, &agent)?,
        Some("remove") => remove(args, &topic)?,
        _ => agent.message(TodoInput::Help),
    };
    mem::drop(agent);

    // Handle the results.
    while let Some(msg) = outputs.recv() {
        match msg {
            TodoOutput::PrintLine(s) => println!("{s}"),
        }
    }
    Ok(())
}

enum TodoInput {
    Help,
    Load(u64, Option<String>),
}

#[derive(Debug, Clone)]
enum TodoOutput {
    PrintLine(String),
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
        }
    }
}

fn remove(mut args: Args, topic: &Topic<TodoMsg>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(id) = args.next().and_then(|s| s.parse::<u64>().ok()) {
        // There we use the blocking version of the remove api. The change will be persisted before return.
        topic.publish((id, None))?;
        println!("Removed task with id {id}")
    } else {
        println!("No task was provided. sync_demo remove [task]")
    }
    Ok(())
}

fn add(
    mut args: Args,
    topic: &Topic<TodoMsg>,
    agent: &Agent<Todo>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(task) = args.next() {
        let next_id = agent
            .state()
            .0
            .last_key_value()
            .map(|(id, _)| *id + 1)
            .unwrap_or(1);

        topic.publish((next_id, Some(task.clone())))?;
        println!("Added {task} with id {next_id}")
    } else {
        println!("No task was provided. sync_demo add [task]")
    }
    Ok(())
}

fn list(agent: &Agent<Todo>) {
    for (id, task) in agent.state().0.iter() {
        println!("{id}: {task}");
    }
}

fn load(topic: &Topic<TodoMsg>) -> Result<Vec<TodoInput>, Box<dyn Error + Send + Sync>> {
    topic
        .subscribe()
        .map(|msg| msg.map(|(key, value)| TodoInput::Load(key, value)))
        .collect()
}
