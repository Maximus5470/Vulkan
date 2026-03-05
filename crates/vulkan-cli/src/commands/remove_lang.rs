use std::{env, error::Error};

use vulkan_core::registry;

pub fn handle(args: &mut env::Args) -> Result<(), Box<dyn Error>> {
    let language = match args.next() {
        Some(language) => language,
        None => {
            eprintln!("Language not specified");
            return Err("Language not specified".into());
        }
    };

    let mut registry = registry::load_registry_from_file();
    registry.remove_runtime(&language);
    registry::save_registry(&registry)?;

    println!("Successfully removed language {}", language);
    Ok(())
}
