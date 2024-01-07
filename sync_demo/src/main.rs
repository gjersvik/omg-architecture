use std::{
    collections::BTreeMap,
    env::{self, Args},
    error::Error,
};

use omg_core::Agency;

fn main() -> Result<(), Box<dyn Error>> {
    // Before the main application starts we configure the Agency using crates that implements features.
    // In this case we device to use Sqlite as backed and configure it with what file to use.
    let storage = omg_sqlite::file_blocking("todo.db").unwrap();

    let agency = Agency::new(storage);

    // Get arguments
    let mut args = env::args();

    // Remove the first one don't need to know the executable
    args.next();

    // Match on the next to decide operation
    match args.next().as_deref() {
        Some("list") => list(&agency),
        Some("add") => add(args, &agency),
        Some("remove") => remove(args, &agency),
        _ => {
            help();
            Ok(())
        }
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

fn remove(mut args: Args, agency: &Agency) -> Result<(), Box<dyn Error>> {
    if let Some(id) = args.next().and_then(|s| s.parse::<u64>().ok()) {
        // There we use the blocking version of the remove api. The change will be persisted before return.
        agency.topic("todo").publish(
            serde_json::to_value((id, None::<String>))?,
        )?;
        println!("Removed task with id {id}")
    } else {
        println!("No task was provided. sync_demo remove [task]")
    }
    Ok(())
}

fn add(mut args: Args, agency: &Agency) -> Result<(), Box<dyn Error>> {
    if let Some(task) = args.next() {
        let next_id = load_tasks(agency)?
            .last_key_value()
            .map(|(id, _)| *id + 1)
            .unwrap_or(1);

        agency
            .topic("todo")
            .publish(serde_json::to_value((next_id, Some(&task)))?)?;
        println!("Added {task} with id {next_id}")
    } else {
        println!("No task was provided. sync_demo add [task]")
    }
    Ok(())
}

fn list(agency: &Agency) -> Result<(), Box<dyn Error>> {
    for (id, task) in load_tasks(agency)?.iter() {
        println!("{id}: {task}");
    }
    Ok(())
}

fn load_tasks(agency: &Agency) -> Result<BTreeMap<u64, String>, Box<dyn Error>> {
    let tasks = agency.topic("todo").subscribe()?
        .try_fold(BTreeMap::new(), |mut map, msg| {
            let event: Result<(u64, Option<String>), _> = serde_json::from_value(msg.data);
            match event {
                Ok((key, Some(value))) => {
                    map.insert(key, value);
                    Ok(map)
                }
                Ok((key, None)) => {
                    map.remove(&key);
                    Ok(map)
                }
                Err(err) => Err(err),
            }
        })?;
    Ok(tasks)
}
