use std::env::{self, Args};

use omg_core::Agency;

fn main() {
    let storage = omg_sqlite::file("todo.db");
    let agency = Agency::new(Some(storage));
    let _todo = agency.load_blocking("todo");

    // Get arguments
    let mut args = env::args();

    // Remove the first one don't need to know the executable
    args.next();

    // Match on the next to decide operation
    match args.next().as_deref() {
        Some("list") => list(),
        Some("add") => add(args),
        Some("remove") => remove(args),
        _ => help(),
    }
}

fn help() {
    todo!("Todo: Implement help")
}

fn remove(_args: Args) {
    todo!("Todo: Implement remove")
}

fn add(_args: Args) {
    todo!("Todo: Implement add")
}

fn list() {
    todo!("Todo: Implement list")
}
