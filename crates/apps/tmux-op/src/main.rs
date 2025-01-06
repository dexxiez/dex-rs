mod config;
mod languages;
mod new_project;
mod project_finder;
mod ui;

use std::{env, time::Instant};

use config::Config;
use new_project::create_project;
use project_finder::find_project_files;

fn print_help() {
    println!("Usage: tmux-op [command]");
    println!();
    println!("Commands:");
    println!("  mk    Create a new project");
    println!();
    println!("Options:");
    println!("  --debug    Print debug information");
}

fn main() -> anyhow::Result<()> {
    dirs::home_dir().expect("Failed to get home directory");
    let args: Vec<String> = env::args().collect();
    let debug = args.iter().any(|arg| arg == "--debug");

    if args.len() > 1 {
        match args[1].as_str() {
            "mk" => {
                return create_project();
            }
            "help" => {
                print_help();
                std::process::exit(0);
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--debug" => {
                // Skip the debug argument
            }
            _ => {
                print_help();
                std::process::exit(1);
            }
        }
    }

    let config_start = Instant::now();
    let config = Config::load()?;
    let config_duration = config_start.elapsed();

    if debug {
        eprintln!("Config load took: {}ms", config_duration.as_millis());
    }

    let search_start = Instant::now();
    let projects = find_project_files(&config.search_paths)?;
    let search_duration = search_start.elapsed();

    if debug {
        eprintln!("Project search took: {}ms", search_duration.as_millis());
    }

    let _ = ui::main(projects);
    Ok(())
}
