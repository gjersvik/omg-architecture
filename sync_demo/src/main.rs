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

fn remove(_args: Args, _agent: Agent) {
    todo!("Todo: Implement remove")
}

fn add(mut args: Args, _agent: Agent) {
    if let Some(_task) = args.next() {
        todo!("Todo: Implement add")
    } else {
        println!("No task was provided. sync_demo add [task]")
    }
}

fn list(_agent: Agent) {
    todo!("Todo: Implement list")
}
