use std::{env, error::Error};

use crate::commands;
use vulkan_core::docker;

pub fn run() -> Result<(), Box<dyn Error>>{
    let mut args = env::args();
    args.next();

    match args.next().expect("msg").as_str(){
        "add" => commands::add::handle(&mut args),
        "remove_lang" => commands::remove_lang::handle(&mut args),
        "remove_version" => commands::remove_version::handle(&mut args),
        "list" => commands::list::handle(),
        "update_images" => docker::update_images(&commands::load_registry()?),
        _ => Err("Unknown command".into()),
    }
}