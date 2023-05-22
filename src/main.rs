use std::env;
use std::path::Path;
use std::process;
use tree_migration::{Config, Error};

fn help() -> String {
    String::from("usage: forest-green [<configuration_file>]\n")
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

    if let Err(e) = tree_migration::run(config) {
        eprintln!("Application Error: {}", e);
        process::exit(1);
    }
}
