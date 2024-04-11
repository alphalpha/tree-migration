use async_std::task;
use std::env;
use std::path::Path;
use std::process;
use tree_migration::{Config, Error};

fn help() -> String {
    String::from("usage: forest-green <configuration_file> [--forest_green]\n")
}

fn main() {
    let mut args = env::args();
    args.next();
    let config = args
        .next()
        .ok_or(Error::Custom(String::from("Cannot parse config file path")))
        .map(|p| Path::new(&p).to_path_buf())
        .and_then(|config_path| Config::from(&config_path))
        .unwrap_or_else(|e| {
            eprintln!("Problem parsing arguments. {}\n\n{}", e, help());
            process::exit(1);
        });

    let forest_green = args
        .next()
        .map(|p| p == "--forest_green")
        .unwrap_or_else(|| false);

    let task = task::spawn(async move {
        if let Err(e) = tree_migration::run(config, forest_green).await {
            eprintln!("Application Error: {}", e);
            process::exit(1);
        }
    });
    task::block_on(task);
    println!("Done");
}
