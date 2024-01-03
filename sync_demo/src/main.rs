use std::env::{self, Args};

use omg_core::{Agency, Agent};

fn main() {
    // Before the main application starts we configure the Agency using crates that implements features.
    // In this case we device to use Sqlite as backed and configure it with what file to use. 
    let storage = omg_sqlite::file("todo.db").unwrap();

    // Once we have configured all the parts we create the Agency we will use for the rest of application.
    let agency = Agency::new(Some(storage));

    // Get or create an agent we will use for this todo app. 
    let todo = agency.get("todo");

    // Get arguments
    let mut args = env::args();

    // Remove the first one don't need to know the executable
    args.next();

    // Match on the next to decide operation
    match args.next().as_deref() {
        Some("list") => list(todo),
        Some("add") => add(args, todo),
        Some("remove") => remove(args, todo),
        _ => help(),
    }
}

fn help() {
    // Just printing som help text nothing relevant to toolbox

    println!("Help for Sync demo todo app");
    println!("sync_demo list");
    println!("Will list all the active todo items.");
    println!("sync_demo add [task]");
    println!("Adds task to the todo lists.");
    println!("sync_demo remove [id]");
    println!("Removes/completes the task with id: id.");
}

fn remove(mut args: Args, mut agent: Agent<u64, String>) {
    if let Some(id) = args.next().and_then(|s| s.parse::<u64>().ok()) {

        // There we use the blocking version of the remove api. The change will be persisted before return.
        if let Some(task) = agent.remove_blocking(&id).unwrap() {
            println!("Removed {task} with id {id}")
        } else {
            println!("Task with id {id} not found. Nothing to remove.")
        }
    } else {
        println!("No task was provided. sync_demo remove [task]")
    }
}

fn add(mut args: Args, mut agent: Agent<u64, String>) {
    if let Some(task) = args.next() {
        // Here we get the one more than the last id in a sorted map.
        let next_id = agent
            .load_blocking().unwrap()
            .last_key_value()
            .map(|(id, _)| *id + 1)
            .unwrap_or(1);

        // The blocking version of insertion. The change will be persisted before return.
        // If there we more than one Todo Agent the data would be synced between them.
        agent.insert_blocking(next_id, task.clone()).unwrap();
        println!("Added {task} with id {next_id}")
    } else {
        println!("No task was provided. sync_demo add [task]")
    }
}

fn list(agent: Agent<u64, String>) {
    // Load will make sure that materialized view is up to date. We use the blocking version.
    // Then return a read only reference to that view so that you can use all the rust apis you need.
    // Load may fail if the underlying storage fails or the types persisted is incompatible with the type asked for. Serialization error.
    for (id, task) in agent.load_blocking().unwrap().iter() {
        println!("{id}: {task}");
    }
}
