use std::process;

mod cli;
mod commands;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        process::exit(0);
    }
}
