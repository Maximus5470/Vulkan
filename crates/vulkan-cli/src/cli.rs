use std::{env, error::Error};

use crate::commands;

pub fn run() -> Result<(), Box<dyn Error>>{
    let mut args = env::args();
    args.next();

    match args.next().expect("msg").as_str(){
        "add" => commands::add::handle(&mut args),
        "remove" => commands::remove::handle(&mut args),
        "list" => commands::list::handle(),
        _ => Err("Unknown command".into()),
    }
}