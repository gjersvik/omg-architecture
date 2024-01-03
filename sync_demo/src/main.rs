use std::env::{self, Args};

use omg_core::{Agency, Agent};

fn main() {
    let storage = omg_sqlite::file("todo.db");
    let agency = Agency::new(Some(storage));
    let todo = agency.load_blocking("todo");

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
        if let Some(task) = agent.remove_blocking(&id) {
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
        let next_id = agent
            .view()
            .last_key_value()
            .map(|(id, _)| *id + 1)
            .unwrap_or(1);
        agent.insert_blocking(next_id, task.clone());
        println!("Added {task} with id {next_id}")
    } else {
        println!("No task was provided. sync_demo add [task]")
    }
}

fn list(agent: Agent<u64, String>) {
    for (id, task) in agent.view().iter() {
        println!("{id}: {task}");
    }
}
