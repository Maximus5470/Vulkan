use std::{env, error::Error};

use crate::commands;
use vulkan_core::docker;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next(); // skip binary name

    let command = args
        .next()
        .ok_or("No command specified. Available: add-language, add, remove_lang, remove_version, list, update_images")?;

    match command.as_str() {
        "add-language" | "add" => commands::add::handle(&mut args),
        "remove_lang" => commands::remove_lang::handle(&mut args),
        "remove_version" => commands::remove_version::handle(&mut args),
        "list" => commands::list::handle(),
        "update_images" => docker::update_images(&commands::load_registry()?),
        _ => Err(format!(
            "Unknown command '{}'. Available: add-language, remove_lang, remove_version, list, update_images",
            command
        )
        .into()),
    }
}
