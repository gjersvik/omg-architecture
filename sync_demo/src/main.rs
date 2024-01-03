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
    todo!("Todo: Implement help")
}

fn remove(_args: Args, _agent: Agent) {
    todo!("Todo: Implement remove")
}

fn add(_args: Args, _agent: Agent) {
    todo!("Todo: Implement add")
}

fn list(_agent: Agent) {
    todo!("Todo: Implement list")
}
