use std::{env, error::Error};

use crate::commands;
use vulkan_core::docker;
use vulkan_core::registry;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();

    let command = args
        .next()
        .ok_or("No command specified. Available: add-language, add, remove_lang, remove_version, list, update_images")?;

    match command.as_str() {
        "add-language" | "add" => commands::add::handle(&mut args),
        "remove-lang" | "remove-language" => commands::remove_lang::handle(&mut args),
        "remove_version" => commands::remove_version::handle(&mut args),
        "add_version" => commands::add_version::handle(&mut args),
        "list" => commands::list::handle(),
        "update_images" => docker::update_images(&registry::load_registry_from_file()),
        _ => Err(format!(
            "Unknown command '{}'. Available: add-language, add, remove_lang, remove_version, list, update_images",
            command
        )
        .into()),
    }
}
