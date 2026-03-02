use std::process;

mod cli;
mod commands;

fn main() {
    if let Err(e) = cli::run(){
        eprint!("Error: {}", e);
        process::exit(1);
    }
}
